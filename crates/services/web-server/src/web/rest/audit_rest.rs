// Audit Log REST endpoints

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use lib_core::model::audit::{AuditLog, AuditLogBmc, AuditLogFilter, CaseVersion, CaseVersionBmc};
use lib_core::model::user::{User, UserBmc};
use lib_core::model::ModelManager;
use lib_rest_core::rest_params::ParamsList;
use lib_rest_core::rest_result::DataRestResult;
use lib_web::{Error as WebError, Result};
use lib_web::middleware::mw_auth::CtxW;
use uuid::Uuid;

/// Verifies that the current user has admin role
async fn require_admin_role(ctx: &lib_core::ctx::Ctx, mm: &ModelManager) -> Result<()> {
	let user: User = UserBmc::get(ctx, mm, ctx.user_id())
		.await
		.map_err(|e| WebError::Model(e))?;

	if user.role != "admin" {
		return Err(WebError::AccessDenied {
			required_role: "admin".to_string(),
		});
	}

	Ok(())
}

/// GET /api/audit-logs
/// List all audit logs with optional filtering
/// **Requires admin role**
pub async fn list_audit_logs(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Query(params): Query<ParamsList<AuditLogFilter>>,
) -> Result<(StatusCode, Json<DataRestResult<Vec<AuditLog>>>)> {
	let ctx = ctx_w.0;
	tracing::debug!("{:<12} - rest list_audit_logs", "HANDLER");

	// Verify admin role
	require_admin_role(&ctx, &mm).await?;

	let logs = AuditLogBmc::list(&ctx, &mm, params.filters, params.list_options)
		.await
		.map_err(|e| WebError::Model(e))?;

	Ok((StatusCode::OK, Json(DataRestResult { data: logs })))
}

/// GET /api/audit-logs/by-record/{table_name}/{record_id}
/// List audit logs for a specific record
/// **Requires admin role**
pub async fn list_audit_logs_by_record(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((table_name, record_id)): Path<(String, Uuid)>,
) -> Result<(StatusCode, Json<DataRestResult<Vec<AuditLog>>>)> {
	let ctx = ctx_w.0;
	tracing::debug!(
		"{:<12} - rest list_audit_logs_by_record table={} id={}",
		"HANDLER",
		table_name,
		record_id
	);

	// Verify admin role
	require_admin_role(&ctx, &mm).await?;

	let logs = AuditLogBmc::list_by_record(&ctx, &mm, &table_name, record_id)
		.await
		.map_err(|e| WebError::Model(e))?;

	Ok((StatusCode::OK, Json(DataRestResult { data: logs })))
}

/// GET /api/cases/{case_id}/versions
/// List all versions for a specific case
/// **Requires admin role**
pub async fn list_case_versions(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
) -> Result<(StatusCode, Json<DataRestResult<Vec<CaseVersion>>>)> {
	let ctx = ctx_w.0;
	tracing::debug!(
		"{:<12} - rest list_case_versions case_id={}",
		"HANDLER",
		case_id
	);

	// Verify admin role
	require_admin_role(&ctx, &mm).await?;

	let versions = CaseVersionBmc::list_by_case(&ctx, &mm, case_id)
		.await
		.map_err(|e| WebError::Model(e))?;

	Ok((StatusCode::OK, Json(DataRestResult { data: versions })))
}
