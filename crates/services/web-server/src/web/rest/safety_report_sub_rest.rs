// Safety Report sub-resources REST endpoints (C.2.r, C.3.x, C.4.r, C.5)

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use lib_core::model::acs::{
	LITERATURE_REFERENCE_CREATE, LITERATURE_REFERENCE_DELETE, LITERATURE_REFERENCE_LIST,
	LITERATURE_REFERENCE_READ, LITERATURE_REFERENCE_UPDATE, PRIMARY_SOURCE_CREATE,
	PRIMARY_SOURCE_DELETE, PRIMARY_SOURCE_LIST, PRIMARY_SOURCE_READ, PRIMARY_SOURCE_UPDATE,
	SENDER_INFORMATION_CREATE, SENDER_INFORMATION_DELETE, SENDER_INFORMATION_LIST,
	SENDER_INFORMATION_READ, SENDER_INFORMATION_UPDATE, STUDY_INFORMATION_CREATE,
	STUDY_INFORMATION_DELETE, STUDY_INFORMATION_LIST, STUDY_INFORMATION_READ,
	STUDY_INFORMATION_UPDATE, STUDY_REGISTRATION_CREATE, STUDY_REGISTRATION_DELETE,
	STUDY_REGISTRATION_LIST, STUDY_REGISTRATION_READ, STUDY_REGISTRATION_UPDATE,
};
use lib_core::model::safety_report::{
	LiteratureReference, LiteratureReferenceBmc, LiteratureReferenceFilter,
	LiteratureReferenceForCreate, LiteratureReferenceForUpdate, PrimarySource,
	PrimarySourceBmc, PrimarySourceFilter, PrimarySourceForCreate,
	PrimarySourceForUpdate, SenderInformation, SenderInformationBmc,
	SenderInformationFilter, SenderInformationForCreate, SenderInformationForUpdate,
	StudyInformation, StudyInformationBmc, StudyInformationFilter,
	StudyInformationForCreate, StudyInformationForUpdate, StudyRegistrationNumber,
	StudyRegistrationNumberBmc, StudyRegistrationNumberFilter,
	StudyRegistrationNumberForCreate, StudyRegistrationNumberForUpdate,
};
use lib_core::model::ModelManager;
use lib_rest_core::rest_params::{ParamsForCreate, ParamsForUpdate};
use lib_rest_core::rest_result::DataRestResult;
use lib_rest_core::{require_permission, Result};
use lib_web::middleware::mw_auth::CtxW;
use modql::filter::{ListOptions, OpValValue, OpValsValue};
use serde_json::json;
use uuid::Uuid;

// -- Sender Information (C.3.x)

/// POST /api/cases/{case_id}/safety-report/senders
pub async fn create_sender_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
	Json(params): Json<ParamsForCreate<SenderInformationForCreate>>,
) -> Result<(StatusCode, Json<DataRestResult<SenderInformation>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, SENDER_INFORMATION_CREATE)?;
	let ParamsForCreate { data } = params;
	let mut data = data;
	data.case_id = case_id;

	let id = SenderInformationBmc::create(&ctx, &mm, data).await?;
	let entity = SenderInformationBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::CREATED, Json(DataRestResult { data: entity })))
}

/// GET /api/cases/{case_id}/safety-report/senders
pub async fn list_sender_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
) -> Result<(StatusCode, Json<DataRestResult<Vec<SenderInformation>>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, SENDER_INFORMATION_LIST)?;
	let filter = SenderInformationFilter {
		case_id: Some(OpValsValue::from(vec![OpValValue::Eq(json!(
			case_id.to_string()
		))])),
		..Default::default()
	};
	let entities = SenderInformationBmc::list(
		&ctx,
		&mm,
		Some(vec![filter]),
		Some(ListOptions::default()),
	)
	.await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entities })))
}

/// GET /api/cases/{case_id}/safety-report/senders/{id}
pub async fn get_sender_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, Json<DataRestResult<SenderInformation>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, SENDER_INFORMATION_READ)?;
	let entity = SenderInformationBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// PUT /api/cases/{case_id}/safety-report/senders/{id}
pub async fn update_sender_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, id)): Path<(Uuid, Uuid)>,
	Json(params): Json<ParamsForUpdate<SenderInformationForUpdate>>,
) -> Result<(StatusCode, Json<DataRestResult<SenderInformation>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, SENDER_INFORMATION_UPDATE)?;
	let ParamsForUpdate { data } = params;
	SenderInformationBmc::update(&ctx, &mm, id, data).await?;
	let entity = SenderInformationBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// DELETE /api/cases/{case_id}/safety-report/senders/{id}
pub async fn delete_sender_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {
	let ctx = ctx_w.0;
	require_permission(&ctx, SENDER_INFORMATION_DELETE)?;
	SenderInformationBmc::delete(&ctx, &mm, id).await?;
	Ok(StatusCode::NO_CONTENT)
}

// -- Primary Sources (C.2.r)

/// POST /api/cases/{case_id}/safety-report/primary-sources
pub async fn create_primary_source(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
	Json(params): Json<ParamsForCreate<PrimarySourceForCreate>>,
) -> Result<(StatusCode, Json<DataRestResult<PrimarySource>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, PRIMARY_SOURCE_CREATE)?;
	let ParamsForCreate { data } = params;
	let mut data = data;
	data.case_id = case_id;

	let id = PrimarySourceBmc::create(&ctx, &mm, data).await?;
	let entity = PrimarySourceBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::CREATED, Json(DataRestResult { data: entity })))
}

/// GET /api/cases/{case_id}/safety-report/primary-sources
pub async fn list_primary_sources(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
) -> Result<(StatusCode, Json<DataRestResult<Vec<PrimarySource>>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, PRIMARY_SOURCE_LIST)?;
	let filter = PrimarySourceFilter {
		case_id: Some(OpValsValue::from(vec![OpValValue::Eq(json!(
			case_id.to_string()
		))])),
		..Default::default()
	};
	let entities = PrimarySourceBmc::list(
		&ctx,
		&mm,
		Some(vec![filter]),
		Some(ListOptions::default()),
	)
	.await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entities })))
}

/// GET /api/cases/{case_id}/safety-report/primary-sources/{id}
pub async fn get_primary_source(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, Json<DataRestResult<PrimarySource>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, PRIMARY_SOURCE_READ)?;
	let entity = PrimarySourceBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// PUT /api/cases/{case_id}/safety-report/primary-sources/{id}
pub async fn update_primary_source(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, id)): Path<(Uuid, Uuid)>,
	Json(params): Json<ParamsForUpdate<PrimarySourceForUpdate>>,
) -> Result<(StatusCode, Json<DataRestResult<PrimarySource>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, PRIMARY_SOURCE_UPDATE)?;
	let ParamsForUpdate { data } = params;
	PrimarySourceBmc::update(&ctx, &mm, id, data).await?;
	let entity = PrimarySourceBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// DELETE /api/cases/{case_id}/safety-report/primary-sources/{id}
pub async fn delete_primary_source(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {
	let ctx = ctx_w.0;
	require_permission(&ctx, PRIMARY_SOURCE_DELETE)?;
	PrimarySourceBmc::delete(&ctx, &mm, id).await?;
	Ok(StatusCode::NO_CONTENT)
}

// -- Literature References (C.4.r)

/// POST /api/cases/{case_id}/safety-report/literature
pub async fn create_literature_reference(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
	Json(params): Json<ParamsForCreate<LiteratureReferenceForCreate>>,
) -> Result<(StatusCode, Json<DataRestResult<LiteratureReference>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, LITERATURE_REFERENCE_CREATE)?;
	let ParamsForCreate { data } = params;
	let mut data = data;
	data.case_id = case_id;

	let id = LiteratureReferenceBmc::create(&ctx, &mm, data).await?;
	let entity = LiteratureReferenceBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::CREATED, Json(DataRestResult { data: entity })))
}

/// GET /api/cases/{case_id}/safety-report/literature
pub async fn list_literature_references(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
) -> Result<(StatusCode, Json<DataRestResult<Vec<LiteratureReference>>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, LITERATURE_REFERENCE_LIST)?;
	let filter = LiteratureReferenceFilter {
		case_id: Some(OpValsValue::from(vec![OpValValue::Eq(json!(
			case_id.to_string()
		))])),
		..Default::default()
	};
	let entities = LiteratureReferenceBmc::list(
		&ctx,
		&mm,
		Some(vec![filter]),
		Some(ListOptions::default()),
	)
	.await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entities })))
}

/// GET /api/cases/{case_id}/safety-report/literature/{id}
pub async fn get_literature_reference(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, Json<DataRestResult<LiteratureReference>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, LITERATURE_REFERENCE_READ)?;
	let entity = LiteratureReferenceBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// PUT /api/cases/{case_id}/safety-report/literature/{id}
pub async fn update_literature_reference(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, id)): Path<(Uuid, Uuid)>,
	Json(params): Json<ParamsForUpdate<LiteratureReferenceForUpdate>>,
) -> Result<(StatusCode, Json<DataRestResult<LiteratureReference>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, LITERATURE_REFERENCE_UPDATE)?;
	let ParamsForUpdate { data } = params;
	LiteratureReferenceBmc::update(&ctx, &mm, id, data).await?;
	let entity = LiteratureReferenceBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// DELETE /api/cases/{case_id}/safety-report/literature/{id}
pub async fn delete_literature_reference(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {
	let ctx = ctx_w.0;
	require_permission(&ctx, LITERATURE_REFERENCE_DELETE)?;
	LiteratureReferenceBmc::delete(&ctx, &mm, id).await?;
	Ok(StatusCode::NO_CONTENT)
}

// -- Study Information (C.5)

/// POST /api/cases/{case_id}/safety-report/studies
pub async fn create_study_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
	Json(params): Json<ParamsForCreate<StudyInformationForCreate>>,
) -> Result<(StatusCode, Json<DataRestResult<StudyInformation>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, STUDY_INFORMATION_CREATE)?;
	let ParamsForCreate { data } = params;
	let mut data = data;
	data.case_id = case_id;

	let id = StudyInformationBmc::create(&ctx, &mm, data).await?;
	let entity = StudyInformationBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::CREATED, Json(DataRestResult { data: entity })))
}

/// GET /api/cases/{case_id}/safety-report/studies
pub async fn list_study_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
) -> Result<(StatusCode, Json<DataRestResult<Vec<StudyInformation>>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, STUDY_INFORMATION_LIST)?;
	let filter = StudyInformationFilter {
		case_id: Some(OpValsValue::from(vec![OpValValue::Eq(json!(
			case_id.to_string()
		))])),
		..Default::default()
	};
	let entities = StudyInformationBmc::list(
		&ctx,
		&mm,
		Some(vec![filter]),
		Some(ListOptions::default()),
	)
	.await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entities })))
}

/// GET /api/cases/{case_id}/safety-report/studies/{id}
pub async fn get_study_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, Json<DataRestResult<StudyInformation>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, STUDY_INFORMATION_READ)?;
	let entity = StudyInformationBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// PUT /api/cases/{case_id}/safety-report/studies/{id}
pub async fn update_study_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, id)): Path<(Uuid, Uuid)>,
	Json(params): Json<ParamsForUpdate<StudyInformationForUpdate>>,
) -> Result<(StatusCode, Json<DataRestResult<StudyInformation>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, STUDY_INFORMATION_UPDATE)?;
	let ParamsForUpdate { data } = params;
	StudyInformationBmc::update(&ctx, &mm, id, data).await?;
	let entity = StudyInformationBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// DELETE /api/cases/{case_id}/safety-report/studies/{id}
pub async fn delete_study_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {
	let ctx = ctx_w.0;
	require_permission(&ctx, STUDY_INFORMATION_DELETE)?;
	StudyInformationBmc::delete(&ctx, &mm, id).await?;
	Ok(StatusCode::NO_CONTENT)
}

// -- Study Registration Numbers (C.5.1.r)

/// POST /api/cases/{case_id}/safety-report/studies/{study_id}/registrations
pub async fn create_study_registration_number(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, study_id)): Path<(Uuid, Uuid)>,
	Json(params): Json<ParamsForCreate<StudyRegistrationNumberForCreate>>,
) -> Result<(StatusCode, Json<DataRestResult<StudyRegistrationNumber>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, STUDY_REGISTRATION_CREATE)?;
	let ParamsForCreate { data } = params;
	let mut data = data;
	data.study_information_id = study_id;

	let id = StudyRegistrationNumberBmc::create(&ctx, &mm, data).await?;
	let entity = StudyRegistrationNumberBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::CREATED, Json(DataRestResult { data: entity })))
}

/// GET /api/cases/{case_id}/safety-report/studies/{study_id}/registrations
pub async fn list_study_registration_numbers(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, study_id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, Json<DataRestResult<Vec<StudyRegistrationNumber>>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, STUDY_REGISTRATION_LIST)?;
	let filter = StudyRegistrationNumberFilter {
		study_information_id: Some(OpValsValue::from(vec![OpValValue::Eq(json!(
			study_id.to_string()
		))])),
		..Default::default()
	};
	let entities = StudyRegistrationNumberBmc::list(
		&ctx,
		&mm,
		Some(vec![filter]),
		Some(ListOptions::default()),
	)
	.await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entities })))
}

/// GET /api/cases/{case_id}/safety-report/studies/{study_id}/registrations/{id}
pub async fn get_study_registration_number(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, _study_id, id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<(StatusCode, Json<DataRestResult<StudyRegistrationNumber>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, STUDY_REGISTRATION_READ)?;
	let entity = StudyRegistrationNumberBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// PUT /api/cases/{case_id}/safety-report/studies/{study_id}/registrations/{id}
pub async fn update_study_registration_number(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, _study_id, id)): Path<(Uuid, Uuid, Uuid)>,
	Json(params): Json<ParamsForUpdate<StudyRegistrationNumberForUpdate>>,
) -> Result<(StatusCode, Json<DataRestResult<StudyRegistrationNumber>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, STUDY_REGISTRATION_UPDATE)?;
	let ParamsForUpdate { data } = params;
	StudyRegistrationNumberBmc::update(&ctx, &mm, id, data).await?;
	let entity = StudyRegistrationNumberBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// DELETE /api/cases/{case_id}/safety-report/studies/{study_id}/registrations/{id}
pub async fn delete_study_registration_number(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, _study_id, id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode> {
	let ctx = ctx_w.0;
	require_permission(&ctx, STUDY_REGISTRATION_DELETE)?;
	StudyRegistrationNumberBmc::delete(&ctx, &mm, id).await?;
	Ok(StatusCode::NO_CONTENT)
}
