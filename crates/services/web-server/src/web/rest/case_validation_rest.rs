use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use lib_core::ctx::Ctx;
use lib_core::model::acs::CASE_READ;
use lib_core::model::case::CaseBmc;
use lib_core::model::message_header::MessageHeaderBmc;
use lib_core::model::ModelManager;
use lib_core::xml::validate::{CaseValidationReport, ValidationProfile};
use lib_rest_core::rest_result::DataRestResult;
use lib_rest_core::{require_permission, Error, Result};
use lib_web::middleware::mw_auth::CtxW;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ValidationQuery {
	pub profile: Option<String>,
}

async fn resolve_profile(
	ctx: &Ctx,
	mm: &ModelManager,
	case_id: Uuid,
	profile: Option<&str>,
) -> Result<ValidationProfile> {
	if let Some(value) = profile {
		return ValidationProfile::parse(value).ok_or_else(|| Error::BadRequest {
			message: format!(
				"invalid validation profile '{value}' (expected: ich, fda or mfds)"
			),
		});
	}

	if let Ok(case) = CaseBmc::get(ctx, mm, case_id).await {
		if let Some(value) = case.validation_profile.as_deref() {
			if let Some(parsed) = ValidationProfile::parse(value) {
				return Ok(parsed);
			}
		}
	}

	let header =
		match MessageHeaderBmc::get_by_case(&Ctx::root_ctx(), mm, case_id).await {
			Ok(header) => Some(header),
			Err(lib_core::model::Error::EntityUuidNotFound { entity, id })
				if entity == "message_headers" && id == case_id =>
			{
				None
			}
			Err(err) => return Err(err.into()),
		};

	let inferred = header
		.as_ref()
		.and_then(|h| h.batch_receiver_identifier.as_deref())
		.map(|v| v.trim().to_ascii_uppercase());

	let is_mfds = inferred
		.as_deref()
		.map(|value| value.contains("MFDS"))
		.unwrap_or(false);

	Ok(if is_mfds {
		ValidationProfile::Mfds
	} else {
		ValidationProfile::Fda
	})
}

/// GET /api/cases/{case_id}/validation
/// Returns case validation issues split as blocking/non-blocking for the wizard.
pub async fn validate_case(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
	Query(query): Query<ValidationQuery>,
) -> Result<(StatusCode, Json<DataRestResult<CaseValidationReport>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, CASE_READ)?;

	let profile =
		resolve_profile(&ctx, &mm, case_id, query.profile.as_deref()).await?;

	let report = match profile {
		ValidationProfile::Ich => {
			lib_core::xml::ich::validation::validate_case(&ctx, &mm, case_id).await?
		}
		ValidationProfile::Fda => {
			lib_core::xml::fda::validation::validate_case(&ctx, &mm, case_id).await?
		}
		ValidationProfile::Mfds => {
			lib_core::xml::mfds::validation::validate_case(&ctx, &mm, case_id)
				.await?
		}
	};

	Ok((StatusCode::OK, Json(DataRestResult { data: report })))
}
