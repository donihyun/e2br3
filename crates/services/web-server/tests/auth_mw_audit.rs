use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use lib_auth::token::generate_web_token;
use lib_core::_dev_utils;
use lib_core::ctx::{ROLE_ADMIN, ROLE_VIEWER, SYSTEM_ORG_ID, SYSTEM_USER_ID};
use lib_core::model::store::set_org_context_dbx;
use lib_core::model::user::UserBmc;
use lib_core::model::ModelManager;
use serde_json::{json, Value};
use serial_test::serial;
use tower::ServiceExt;
use uuid::Uuid;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

async fn enable_rls(mm: &ModelManager) -> Result<()> {
	let query = sqlx::query("SET ROLE e2br3_app_role");
	mm.dbx().execute(query).await?;
	let query = sqlx::query("SET row_security = on");
	mm.dbx().execute(query).await?;
	Ok(())
}

fn system_user_id() -> Uuid {
	Uuid::parse_str(SYSTEM_USER_ID).expect("system user id")
}

fn system_org_id() -> Uuid {
	Uuid::parse_str(SYSTEM_ORG_ID).expect("system org id")
}

async fn insert_org(mm: &ModelManager, created_by: Uuid) -> Result<Uuid> {
	let org_id = Uuid::new_v4();
	let query = sqlx::query(
		"INSERT INTO organizations (id, name, org_type, address, contact_email, created_by, updated_by)
		 VALUES ($1, $2, $3, $4, $5, $6, $6)",
	)
	.bind(org_id)
	.bind(format!("RLS Org {org_id}"))
	.bind("internal")
	.bind("123 RLS St")
	.bind(format!("rls-org-{org_id}@example.com"))
	.bind(created_by);
	mm.dbx().execute(query).await?;
	Ok(org_id)
}

async fn insert_user(
	mm: &ModelManager,
	org_id: Uuid,
	role: &str,
	created_by: Uuid,
) -> Result<(Uuid, String, Uuid)> {
	let user_id = Uuid::new_v4();
	let token_salt = Uuid::new_v4();
	let pwd_salt = Uuid::new_v4();
	let email = format!("rls-user-{user_id}@example.com");
	let username = format!("rls_user_{user_id}");
	let query = sqlx::query(
		"INSERT INTO users (id, organization_id, email, username, pwd_salt, token_salt, role, active, created_by, updated_by)
		 VALUES ($1, $2, $3, $4, $5, $6, $7, true, $8, $8)",
	)
	.bind(user_id)
	.bind(org_id)
	.bind(&email)
	.bind(username)
	.bind(pwd_salt)
	.bind(token_salt)
	.bind(role)
	.bind(created_by);
	mm.dbx().execute(query).await?;
	Ok((user_id, email, token_salt))
}

async fn setup_env() {
	std::env::set_var("SERVICE_DB_URL", "postgres://app_user:dev_only_pwd@localhost/app_db");
	std::env::set_var("SERVICE_WEB_FOLDER", "web-folder");
	std::env::set_var("SERVICE_PWD_KEY", "ZmFrZV9rZXk");
	std::env::set_var("SERVICE_TOKEN_KEY", "ZmFrZV9rZXk");
	std::env::set_var("SERVICE_TOKEN_DURATION_SEC", "3600");
	std::env::set_var("E2BR3_DEVDB_SKIP_ROLE_SETUP", "0");
	std::env::set_var("E2BR3_DB_ROLE", "e2br3_app_role");
}

async fn setup_users() -> Result<(ModelManager, String, Uuid, Uuid, String, Uuid)> {
	setup_env().await;
	_dev_utils::init_dev().await;
	let mm = ModelManager::new().await?;

	let dbx = mm.dbx();
	dbx.begin_txn().await?;
	enable_rls(&mm).await?;
	set_org_context_dbx(dbx, system_org_id(), ROLE_ADMIN).await?;
	let query = sqlx::query("SELECT set_current_user_context($1)")
		.bind(system_user_id());
	mm.dbx().execute(query).await?;

	let org_id = insert_org(&mm, system_user_id()).await?;
	let (admin_id, admin_email, admin_salt) =
		insert_user(&mm, org_id, ROLE_ADMIN, system_user_id()).await?;
	let (viewer_id, viewer_email, viewer_salt) =
		insert_user(&mm, org_id, ROLE_VIEWER, system_user_id()).await?;
	dbx.commit_txn().await?;

	let admin_ctx =
		lib_core::ctx::Ctx::new(system_user_id(), system_org_id(), ROLE_ADMIN.to_string())?;
	let dbx = mm.dbx();
	dbx.begin_txn().await?;
	enable_rls(&mm).await?;
	set_org_context_dbx(dbx, system_org_id(), ROLE_ADMIN).await?;
	let query = sqlx::query("SELECT set_current_user_context($1)")
		.bind(system_user_id());
	mm.dbx().execute(query).await?;
	UserBmc::update_pwd(&admin_ctx, &mm, admin_id, "adminpwd").await?;
	UserBmc::update_pwd(&admin_ctx, &mm, viewer_id, "viewpwd").await?;
	dbx.commit_txn().await?;

	Ok((
		mm,
		admin_email,
		admin_salt,
		admin_id,
		viewer_email,
		viewer_salt,
	))
}

fn cookie_header(token: &str) -> String {
	format!("auth-token={token}")
}

#[serial]
#[tokio::test]
async fn test_auth_login_refresh_logoff() -> Result<()> {
	let (mm, admin_email, admin_salt, _admin_id, _viewer_email, _viewer_salt) =
		setup_users().await?;

	let app = web_server::app(mm.clone());
	let login_body = json!({ "email": admin_email, "pwd": "adminpwd" });
	let req = Request::builder()
		.method("POST")
		.uri("/auth/v1/login")
		.header("content-type", "application/json")
		.body(Body::from(login_body.to_string()))?;
	let res = app.clone().oneshot(req).await?;
	let status = res.status();
	if status != StatusCode::OK {
		let err = res
			.extensions()
			.get::<std::sync::Arc<lib_web::Error>>()
			.map(|e| format!("{e:?}"));
		let body = to_bytes(res.into_body(), usize::MAX).await?;
		return Err(format!(
			"login status {} body {} err {:?}",
			status,
			String::from_utf8_lossy(&body),
			err
		)
		.into());
	}

	let token = generate_web_token(&admin_email, admin_salt)?;
	let req = Request::builder()
		.method("POST")
		.uri("/auth/v1/refresh")
		.header("cookie", cookie_header(&token.to_string()))
		.body(Body::empty())?;
	let res = app.clone().oneshot(req).await?;
	let status = res.status();
	if status != StatusCode::OK {
		let err = res
			.extensions()
			.get::<std::sync::Arc<lib_web::Error>>()
			.map(|e| format!("{e:?}"));
		let body = to_bytes(res.into_body(), usize::MAX).await?;
		return Err(format!(
			"refresh status {} body {} err {:?}",
			status,
			String::from_utf8_lossy(&body),
			err
		)
		.into());
	}

	let req = Request::builder()
		.method("POST")
		.uri("/auth/v1/logoff")
		.header("cookie", cookie_header(&token.to_string()))
		.header("content-type", "application/json")
		.body(Body::from(r#"{"logoff":true}"#))?;
	let res = app.oneshot(req).await?;
	let status = res.status();
	if status != StatusCode::OK {
		let err = res
			.extensions()
			.get::<std::sync::Arc<lib_web::Error>>()
			.map(|e| format!("{e:?}"));
		let body = to_bytes(res.into_body(), usize::MAX).await?;
		return Err(format!(
			"logoff status {} body {} err {:?}",
			status,
			String::from_utf8_lossy(&body),
			err
		)
		.into());
	}

	Ok(())
}

#[serial]
#[tokio::test]
async fn test_middleware_requires_auth() -> Result<()> {
	let (mm, _admin_email, _admin_salt, _admin_id, _viewer_email, _viewer_salt) =
		setup_users().await?;
	let app = web_server::app(mm);
	let req = Request::builder()
		.method("GET")
		.uri("/api/users")
		.body(Body::empty())?;
	let res = app.oneshot(req).await?;
	assert_eq!(res.status(), StatusCode::FORBIDDEN);
	Ok(())
}

#[serial]
#[tokio::test]
async fn test_refresh_rejects_bad_token() -> Result<()> {
	let (mm, admin_email, admin_salt, _admin_id, _viewer_email, _viewer_salt) =
		setup_users().await?;
	let mut token = generate_web_token(&admin_email, admin_salt)?.to_string();
	token.pop();
	token.push('x');

	let app = web_server::app(mm);
	let req = Request::builder()
		.method("POST")
		.uri("/auth/v1/refresh")
		.header("cookie", cookie_header(&token))
		.body(Body::empty())?;
	let res = app.oneshot(req).await?;
	assert_eq!(res.status(), StatusCode::FORBIDDEN);
	Ok(())
}

#[serial]
#[tokio::test]
async fn test_permission_denied_for_viewer() -> Result<()> {
	let (mm, _admin_email, _admin_salt, _admin_id, viewer_email, viewer_salt) =
		setup_users().await?;
	let token = generate_web_token(&viewer_email, viewer_salt)?;
	let app = web_server::app(mm);
	let req = Request::builder()
		.method("POST")
		.uri("/api/users")
		.header("cookie", cookie_header(&token.to_string()))
		.header("content-type", "application/json")
		.body(Body::from(
			r#"{"data":{"organization_id":"00000000-0000-0000-0000-000000000001","email":"x@y.com","username":"x","pwd_clear":"x","role":"user"}}"#,
		))?;
	let res = app.oneshot(req).await?;
	assert_eq!(res.status(), StatusCode::FORBIDDEN);
	Ok(())
}

#[serial]
#[tokio::test]
async fn test_audit_trail_case_crud() -> Result<()> {
	let (mm, admin_email, admin_salt, admin_id, _viewer_email, _viewer_salt) =
		setup_users().await?;
	let token = generate_web_token(&admin_email, admin_salt)?;

	let app = web_server::app(mm.clone());
	let case_body = json!({
		"data": {
			"organization_id": system_org_id(),
			"safety_report_id": format!("SR-{}", Uuid::new_v4()),
			"status": "draft"
		}
	});
	let req = Request::builder()
		.method("POST")
		.uri("/api/cases")
		.header("cookie", cookie_header(&token.to_string()))
		.header("content-type", "application/json")
		.body(Body::from(case_body.to_string()))?;
	let res = app.clone().oneshot(req).await?;
	assert_eq!(res.status(), StatusCode::CREATED);

	let body = to_bytes(res.into_body(), usize::MAX).await?;
	let value: Value = serde_json::from_slice(&body)?;
	let case_id = value
		.get("data")
		.and_then(|v| v.get("id"))
		.and_then(|v| v.as_str())
		.ok_or("missing case id")?;

	let update_body = json!({
		"data": {
			"status": "submitted"
		}
	});
	let req = Request::builder()
		.method("PUT")
		.uri(format!("/api/cases/{case_id}"))
		.header("cookie", cookie_header(&token.to_string()))
		.header("content-type", "application/json")
		.body(Body::from(update_body.to_string()))?;
	let res = app.clone().oneshot(req).await?;
	assert_eq!(res.status(), StatusCode::OK);

	let req = Request::builder()
		.method("DELETE")
		.uri(format!("/api/cases/{case_id}"))
		.header("cookie", cookie_header(&token.to_string()))
		.body(Body::empty())?;
	let res = app.oneshot(req).await?;
	assert_eq!(res.status(), StatusCode::NO_CONTENT);

	let dbx = mm.dbx();
	dbx.begin_txn().await?;
	let query = sqlx::query("SET ROLE e2br3_auditor_role");
	dbx.execute(query).await?;
	let query = sqlx::query_as::<_, (i64,)>(
		"SELECT COUNT(*) FROM audit_logs WHERE table_name = 'cases' AND record_id = $1 AND user_id = $2",
	)
	.bind(Uuid::parse_str(case_id)?)
	.bind(admin_id);
	let (count,) = dbx.fetch_one(query).await?;
	dbx.rollback_txn().await?;
	assert!(count >= 3);

	Ok(())
}
