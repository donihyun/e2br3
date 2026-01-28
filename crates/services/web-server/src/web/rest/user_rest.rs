// User REST endpoints with RBAC permission checks

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use lib_core::model::acs::{
	has_permission, USER_CREATE, USER_DELETE, USER_LIST, USER_READ, USER_UPDATE,
};
use lib_core::model::user::{
	User, UserBmc, UserFilter, UserForCreate, UserForUpdate,
};
use lib_core::model::ModelManager;
use lib_rest_core::rest_params::{ParamsForCreate, ParamsForUpdate, ParamsList};
use lib_rest_core::rest_result::DataRestResult;
use lib_web::middleware::mw_auth::CtxW;
use lib_web::{Error as WebError, Result};
use uuid::Uuid;

/// POST /api/users
/// Create a new user
/// **Requires User.Create permission (admin only)**
pub async fn create_user(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Json(params): Json<ParamsForCreate<UserForCreate>>,
) -> Result<(StatusCode, Json<DataRestResult<User>>)> {
	let ctx = ctx_w.0;
	tracing::debug!("{:<12} - rest create_user", "HANDLER");

	// Check permission
	if !has_permission(ctx.role(), USER_CREATE) {
		return Err(WebError::PermissionDenied {
			required_permission: "User.Create".to_string(),
		});
	}

	let ParamsForCreate { data } = params;
	let id = UserBmc::create(&ctx, &mm, data)
		.await
		.map_err(WebError::Model)?;
	let entity = UserBmc::get(&ctx, &mm, id).await.map_err(WebError::Model)?;

	Ok((StatusCode::CREATED, Json(DataRestResult { data: entity })))
}

/// GET /api/users/:id
/// Get a user by ID
/// **Requires User.Read permission (all authenticated users)**
pub async fn get_user(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<DataRestResult<User>>)> {
	let ctx = ctx_w.0;
	tracing::debug!("{:<12} - rest get_user id={}", "HANDLER", id);

	// Check permission
	if !has_permission(ctx.role(), USER_READ) {
		return Err(WebError::PermissionDenied {
			required_permission: "User.Read".to_string(),
		});
	}

	// Non-admin users can only view users in their organization
	// (RLS will enforce this at the database level)
	let entity = UserBmc::get(&ctx, &mm, id).await.map_err(WebError::Model)?;

	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// GET /api/users
/// List all users with optional filtering
/// **Requires User.List permission (all authenticated users can list users in their org)**
pub async fn list_users(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Query(params): Query<ParamsList<UserFilter>>,
) -> Result<(StatusCode, Json<DataRestResult<Vec<User>>>)> {
	let ctx = ctx_w.0;
	tracing::debug!("{:<12} - rest list_users", "HANDLER");

	// Check permission
	if !has_permission(ctx.role(), USER_LIST) {
		return Err(WebError::PermissionDenied {
			required_permission: "User.List".to_string(),
		});
	}

	// RLS will filter to users in the same organization (unless admin)
	let entities = UserBmc::list(&ctx, &mm, params.filters, params.list_options)
		.await
		.map_err(WebError::Model)?;

	Ok((StatusCode::OK, Json(DataRestResult { data: entities })))
}

/// PUT /api/users/:id
/// Update a user
/// **Requires User.Update permission (admin only)**
pub async fn update_user(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(id): Path<Uuid>,
	Json(params): Json<ParamsForUpdate<UserForUpdate>>,
) -> Result<(StatusCode, Json<DataRestResult<User>>)> {
	let ctx = ctx_w.0;
	tracing::debug!("{:<12} - rest update_user id={}", "HANDLER", id);

	// Check permission
	if !has_permission(ctx.role(), USER_UPDATE) {
		return Err(WebError::PermissionDenied {
			required_permission: "User.Update".to_string(),
		});
	}

	let ParamsForUpdate { data } = params;
	UserBmc::update(&ctx, &mm, id, data)
		.await
		.map_err(WebError::Model)?;
	let entity = UserBmc::get(&ctx, &mm, id).await.map_err(WebError::Model)?;

	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// DELETE /api/users/:id
/// Delete a user
/// **Requires User.Delete permission (admin only)**
pub async fn delete_user(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(id): Path<Uuid>,
) -> Result<StatusCode> {
	let ctx = ctx_w.0;
	tracing::debug!("{:<12} - rest delete_user id={}", "HANDLER", id);

	// Check permission
	if !has_permission(ctx.role(), USER_DELETE) {
		return Err(WebError::PermissionDenied {
			required_permission: "User.Delete".to_string(),
		});
	}

	// Prevent users from deleting themselves
	if id == ctx.user_id() {
		return Err(WebError::AccessDenied {
			required_role: "Cannot delete yourself".to_string(),
		});
	}

	UserBmc::delete(&ctx, &mm, id)
		.await
		.map_err(WebError::Model)?;

	Ok(StatusCode::NO_CONTENT)
}

/// GET /api/users/me
/// Get current user's profile
/// **Any authenticated user**
pub async fn get_current_user(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
) -> Result<(StatusCode, Json<DataRestResult<User>>)> {
	let ctx = ctx_w.0;
	tracing::debug!("{:<12} - rest get_current_user", "HANDLER");

	let entity = UserBmc::get(&ctx, &mm, ctx.user_id())
		.await
		.map_err(WebError::Model)?;

	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}
