use crate::error::{Error, Result};
use crate::utils::token;
use axum::extract::State;
use axum::Json;
use lib_auth::pwd::{self, ContentToHash, SchemeStatus};
use lib_core::ctx::Ctx;
use lib_core::model::store::set_full_context_dbx;
use lib_core::model::user::{UserBmc, UserForAuth, UserForLogin};
use lib_core::model::ModelManager;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::query;
use tower_cookies::Cookies;
use tracing::debug;

// region:    --- Login
pub async fn api_login_handler(
	State(mm): State<ModelManager>,
	cookies: Cookies,
	Json(payload): Json<LoginPayload>,
) -> Result<Json<Value>> {
	debug!("{:<12} - api_login_handler", "HANDLER");

	let LoginPayload {
		email,
		pwd: pwd_clear,
	} = payload;
	let root_ctx = Ctx::root_ctx();

	// -- Get the user (set auth email in-session to satisfy RLS).
	let user: UserForLogin = {
		let mm_txn = mm.new_with_txn().map_err(Error::Model)?;
		mm_txn
			.dbx()
			.begin_txn()
			.await
			.map_err(|err| Error::Model(err.into()))?;
		let auth_email_query =
			query("SELECT set_config('app.auth_email', $1, true)").bind(&email);
		if let Err(err) = mm_txn.dbx().execute(auth_email_query).await {
			let _ = mm_txn.dbx().rollback_txn().await;
			return Err(Error::Model(err.into()));
		}
		let user = match UserBmc::first_by_email(&root_ctx, &mm_txn, &email)
			.await
		{
			Ok(user) => user,
			Err(err) => {
				let _ = mm_txn.dbx().rollback_txn().await;
				return Err(Error::Model(err));
			}
		};
		if let Err(err) = mm_txn.dbx().commit_txn().await {
			let _ = mm_txn.dbx().rollback_txn().await;
			return Err(Error::Model(err.into()));
		}
		user
	}
	.ok_or(Error::LoginFailEmailNotFound)?;
	let user_id = user.id;
	let user_ctx = Ctx::new(user.id, user.organization_id, user.role.clone())
		.map_err(|_| Error::LoginFailUserCtxCreate { user_id })?;

	// -- Validate the password.
	let Some(pwd) = user.pwd else {
		return Err(Error::LoginFailUserHasNoPwd { user_id });
	};

	let scheme_status = pwd::validate_pwd(
		ContentToHash {
			salt: user.pwd_salt,
			content: pwd_clear.clone(),
		},
		pwd,
	)
	.await
	.map_err(|_| Error::LoginFailPwdNotMatching { user_id })?;

	// -- Update password scheme if needed
	if let SchemeStatus::Outdated = scheme_status {
		debug!("pwd encrypt scheme outdated, upgrading.");
		UserBmc::update_pwd(&user_ctx, &mm, user.id, &pwd_clear).await?;
	}

	// -- Set web token.
	token::set_token_cookie(&cookies, &user.email, user.token_salt)?;

	// Create the success body.
	let body = Json(json!({
		"result": {
			"success": true
		}
	}));

	Ok(body)
}

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
	email: String,
	pwd: String,
}
// endregion: --- Login

// region:    --- Logoff
pub async fn api_logoff_handler(
	cookies: Cookies,
	Json(payload): Json<LogoffPayload>,
) -> Result<Json<Value>> {
	debug!("{:<12} - api_logoff_handler", "HANDLER");
	let should_logoff = payload.logoff;

	if should_logoff {
		token::remove_token_cookie(&cookies)?;
	}

	// Create the success body.
	let body = Json(json!({
		"result": {
			"logged_off": should_logoff
		}
	}));

	Ok(body)
}

#[derive(Debug, Deserialize)]
pub struct LogoffPayload {
	logoff: bool,
}
// endregion: --- Logoff

// region:    --- Token Refresh
pub async fn api_refresh_handler(
	State(mm): State<ModelManager>,
	ctx_w: crate::middleware::mw_auth::CtxW,
	cookies: Cookies,
) -> Result<Json<Value>> {
	debug!("{:<12} - api_refresh_handler", "HANDLER");

	let ctx = ctx_w.0;
	let user_id = ctx.user_id();

	// Ensure RLS/audit context is set on the same connection used for the query.
	let dbx = mm.dbx();
	dbx.begin_txn()
		.await
		.map_err(|err| Error::Model(err.into()))?;
	if let Err(err) = set_full_context_dbx(
		dbx,
		ctx.user_id(),
		ctx.organization_id(),
		ctx.role(),
	)
	.await
	{
		let _ = dbx.rollback_txn().await;
		return Err(Error::Model(err));
	}

	// Get the user to refresh token
	let user: UserForAuth = match UserBmc::get(&ctx, &mm, user_id).await {
		Ok(user) => user,
		Err(err) => {
			let _ = dbx.rollback_txn().await;
			return Err(Error::Model(err));
		}
	};
	if let Err(err) = dbx.commit_txn().await {
		let _ = dbx.rollback_txn().await;
		return Err(Error::Model(err.into()));
	}

	// Set new web token
	token::set_token_cookie(&cookies, &user.email, user.token_salt)?;

	// Calculate expiration time (15 minutes from now)
	let expires_at = time::OffsetDateTime::now_utc() + time::Duration::minutes(15);

	// Create the success body with expiration info
	let body = Json(json!({
		"data": {
			"expiresAt": expires_at.format(&time::format_description::well_known::Rfc3339).unwrap_or_default()
		}
	}));

	Ok(body)
}
// endregion: --- Token Refresh
