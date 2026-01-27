// Narrative sub-resources REST endpoints (H.3.r, H.5.r)

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use lib_core::model::narrative::{
	CaseSummaryInformation, CaseSummaryInformationBmc, CaseSummaryInformationFilter,
	CaseSummaryInformationForCreate, CaseSummaryInformationForUpdate,
	NarrativeInformationBmc, SenderDiagnosis, SenderDiagnosisBmc,
	SenderDiagnosisFilter, SenderDiagnosisForCreate, SenderDiagnosisForUpdate,
};
use lib_core::model::ModelManager;
use lib_rest_core::rest_params::{ParamsForCreate, ParamsForUpdate};
use lib_rest_core::rest_result::DataRestResult;
use lib_rest_core::Result;
use lib_web::middleware::mw_auth::CtxW;
use modql::filter::{ListOptions, OpValValue, OpValsValue};
use serde_json::json;
use uuid::Uuid;

async fn narrative_id_for_case(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Uuid> {
	let narrative = NarrativeInformationBmc::get_by_case(ctx, mm, case_id).await?;
	Ok(narrative.id)
}

// -- Sender Diagnosis (H.3.r)

/// POST /api/cases/{case_id}/narrative/sender-diagnoses
pub async fn create_sender_diagnosis(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
	Json(params): Json<ParamsForCreate<SenderDiagnosisForCreate>>,
) -> Result<(StatusCode, Json<DataRestResult<SenderDiagnosis>>)> {
	let ctx = ctx_w.0;
	let narrative_id = narrative_id_for_case(&ctx, &mm, case_id).await?;

	let ParamsForCreate { data } = params;
	let mut data = data;
	data.narrative_id = narrative_id;

	let id = SenderDiagnosisBmc::create(&ctx, &mm, data).await?;
	let entity = SenderDiagnosisBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::CREATED, Json(DataRestResult { data: entity })))
}

/// GET /api/cases/{case_id}/narrative/sender-diagnoses
pub async fn list_sender_diagnoses(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
) -> Result<(StatusCode, Json<DataRestResult<Vec<SenderDiagnosis>>>)> {
	let ctx = ctx_w.0;
	let narrative_id = narrative_id_for_case(&ctx, &mm, case_id).await?;

	let filter = SenderDiagnosisFilter {
		narrative_id: Some(OpValsValue::from(vec![OpValValue::Eq(json!(
			narrative_id.to_string()
		))])),
		..Default::default()
	};
	let entities = SenderDiagnosisBmc::list(
		&ctx,
		&mm,
		Some(vec![filter]),
		Some(ListOptions::default()),
	)
	.await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entities })))
}

/// GET /api/cases/{case_id}/narrative/sender-diagnoses/{id}
pub async fn get_sender_diagnosis(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, Json<DataRestResult<SenderDiagnosis>>)> {
	let ctx = ctx_w.0;
	let entity = SenderDiagnosisBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// PUT /api/cases/{case_id}/narrative/sender-diagnoses/{id}
pub async fn update_sender_diagnosis(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, id)): Path<(Uuid, Uuid)>,
	Json(params): Json<ParamsForUpdate<SenderDiagnosisForUpdate>>,
) -> Result<(StatusCode, Json<DataRestResult<SenderDiagnosis>>)> {
	let ctx = ctx_w.0;
	let ParamsForUpdate { data } = params;
	SenderDiagnosisBmc::update(&ctx, &mm, id, data).await?;
	let entity = SenderDiagnosisBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// DELETE /api/cases/{case_id}/narrative/sender-diagnoses/{id}
pub async fn delete_sender_diagnosis(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {
	let ctx = ctx_w.0;
	SenderDiagnosisBmc::delete(&ctx, &mm, id).await?;
	Ok(StatusCode::NO_CONTENT)
}

// -- Case Summary Information (H.5.r)

/// POST /api/cases/{case_id}/narrative/summaries
pub async fn create_case_summary_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
	Json(params): Json<ParamsForCreate<CaseSummaryInformationForCreate>>,
) -> Result<(StatusCode, Json<DataRestResult<CaseSummaryInformation>>)> {
	let ctx = ctx_w.0;
	let narrative_id = narrative_id_for_case(&ctx, &mm, case_id).await?;

	let ParamsForCreate { data } = params;
	let mut data = data;
	data.narrative_id = narrative_id;

	let id = CaseSummaryInformationBmc::create(&ctx, &mm, data).await?;
	let entity = CaseSummaryInformationBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::CREATED, Json(DataRestResult { data: entity })))
}

/// GET /api/cases/{case_id}/narrative/summaries
pub async fn list_case_summary_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
) -> Result<(StatusCode, Json<DataRestResult<Vec<CaseSummaryInformation>>>)> {
	let ctx = ctx_w.0;
	let narrative_id = narrative_id_for_case(&ctx, &mm, case_id).await?;

	let filter = CaseSummaryInformationFilter {
		narrative_id: Some(OpValsValue::from(vec![OpValValue::Eq(json!(
			narrative_id.to_string()
		))])),
		..Default::default()
	};
	let entities = CaseSummaryInformationBmc::list(
		&ctx,
		&mm,
		Some(vec![filter]),
		Some(ListOptions::default()),
	)
	.await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entities })))
}

/// GET /api/cases/{case_id}/narrative/summaries/{id}
pub async fn get_case_summary_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, Json<DataRestResult<CaseSummaryInformation>>)> {
	let ctx = ctx_w.0;
	let entity = CaseSummaryInformationBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// PUT /api/cases/{case_id}/narrative/summaries/{id}
pub async fn update_case_summary_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, id)): Path<(Uuid, Uuid)>,
	Json(params): Json<ParamsForUpdate<CaseSummaryInformationForUpdate>>,
) -> Result<(StatusCode, Json<DataRestResult<CaseSummaryInformation>>)> {
	let ctx = ctx_w.0;
	let ParamsForUpdate { data } = params;
	CaseSummaryInformationBmc::update(&ctx, &mm, id, data).await?;
	let entity = CaseSummaryInformationBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// DELETE /api/cases/{case_id}/narrative/summaries/{id}
pub async fn delete_case_summary_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {
	let ctx = ctx_w.0;
	CaseSummaryInformationBmc::delete(&ctx, &mm, id).await?;
	Ok(StatusCode::NO_CONTENT)
}
