// Drug sub-resources REST endpoints (G.k.2.3.r, G.k.4.r, G.k.6.r)

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use lib_core::model::acs::{
	DRUG_DOSAGE_CREATE, DRUG_DOSAGE_DELETE, DRUG_DOSAGE_LIST, DRUG_DOSAGE_READ,
	DRUG_DOSAGE_UPDATE, DRUG_INDICATION_CREATE, DRUG_INDICATION_DELETE,
	DRUG_INDICATION_LIST, DRUG_INDICATION_READ, DRUG_INDICATION_UPDATE,
	DRUG_SUBSTANCE_CREATE, DRUG_SUBSTANCE_DELETE, DRUG_SUBSTANCE_LIST,
	DRUG_SUBSTANCE_READ, DRUG_SUBSTANCE_UPDATE,
};
use lib_core::model::drug::{
	DosageInformation, DosageInformationBmc, DosageInformationFilter,
	DosageInformationForCreate, DosageInformationForUpdate, DrugActiveSubstance,
	DrugActiveSubstanceBmc, DrugActiveSubstanceFilter, DrugActiveSubstanceForCreate,
	DrugActiveSubstanceForUpdate, DrugIndication, DrugIndicationBmc,
	DrugIndicationFilter, DrugIndicationForCreate, DrugIndicationForUpdate,
};
use lib_core::model::{self, ModelManager};
use lib_rest_core::rest_params::{ParamsForCreate, ParamsForUpdate};
use lib_rest_core::rest_result::DataRestResult;
use lib_rest_core::{require_permission, Result};
use lib_web::middleware::mw_auth::CtxW;
use modql::filter::{ListOptions, OpValValue, OpValsValue};
use serde_json::json;
use uuid::Uuid;

fn ensure_drug_scope(
	path_drug_id: Uuid,
	entity_drug_id: Uuid,
	entity_id: Uuid,
	entity: &'static str,
) -> Result<()> {
	if path_drug_id != entity_drug_id {
		return Err(model::Error::EntityUuidNotFound {
			entity,
			id: entity_id,
		}
		.into());
	}
	Ok(())
}

// -- Drug Active Substances (G.k.2.3.r)

/// POST /api/cases/{case_id}/drugs/{drug_id}/active-substances
pub async fn create_drug_active_substance(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, drug_id)): Path<(Uuid, Uuid)>,
	Json(params): Json<ParamsForCreate<DrugActiveSubstanceForCreate>>,
) -> Result<(StatusCode, Json<DataRestResult<DrugActiveSubstance>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, DRUG_SUBSTANCE_CREATE)?;
	let ParamsForCreate { data } = params;
	let mut data = data;
	data.drug_id = drug_id;

	let id = DrugActiveSubstanceBmc::create(&ctx, &mm, data).await?;
	let entity = DrugActiveSubstanceBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::CREATED, Json(DataRestResult { data: entity })))
}

/// GET /api/cases/{case_id}/drugs/{drug_id}/active-substances
pub async fn list_drug_active_substances(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, drug_id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, Json<DataRestResult<Vec<DrugActiveSubstance>>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, DRUG_SUBSTANCE_LIST)?;
	let filter = DrugActiveSubstanceFilter {
		drug_id: Some(OpValsValue::from(vec![OpValValue::Eq(json!(
			drug_id.to_string()
		))])),
		..Default::default()
	};
	let entities = DrugActiveSubstanceBmc::list(
		&ctx,
		&mm,
		Some(vec![filter]),
		Some(ListOptions::default()),
	)
	.await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entities })))
}

/// GET /api/cases/{case_id}/drugs/{drug_id}/active-substances/{id}
pub async fn get_drug_active_substance(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, drug_id, id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<(StatusCode, Json<DataRestResult<DrugActiveSubstance>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, DRUG_SUBSTANCE_READ)?;
	let entity = DrugActiveSubstanceBmc::get(&ctx, &mm, id).await?;
	ensure_drug_scope(drug_id, entity.drug_id, id, "drug_active_substances")?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// PUT /api/cases/{case_id}/drugs/{drug_id}/active-substances/{id}
pub async fn update_drug_active_substance(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, drug_id, id)): Path<(Uuid, Uuid, Uuid)>,
	Json(params): Json<ParamsForUpdate<DrugActiveSubstanceForUpdate>>,
) -> Result<(StatusCode, Json<DataRestResult<DrugActiveSubstance>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, DRUG_SUBSTANCE_UPDATE)?;
	let ParamsForUpdate { data } = params;
	let entity = DrugActiveSubstanceBmc::get(&ctx, &mm, id).await?;
	ensure_drug_scope(drug_id, entity.drug_id, id, "drug_active_substances")?;
	DrugActiveSubstanceBmc::update(&ctx, &mm, id, data).await?;
	let entity = DrugActiveSubstanceBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// DELETE /api/cases/{case_id}/drugs/{drug_id}/active-substances/{id}
pub async fn delete_drug_active_substance(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, drug_id, id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode> {
	let ctx = ctx_w.0;
	require_permission(&ctx, DRUG_SUBSTANCE_DELETE)?;
	let entity = DrugActiveSubstanceBmc::get(&ctx, &mm, id).await?;
	ensure_drug_scope(drug_id, entity.drug_id, id, "drug_active_substances")?;
	DrugActiveSubstanceBmc::delete(&ctx, &mm, id).await?;
	Ok(StatusCode::NO_CONTENT)
}

// -- Dosage Information (G.k.4.r)

/// POST /api/cases/{case_id}/drugs/{drug_id}/dosages
pub async fn create_dosage_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, drug_id)): Path<(Uuid, Uuid)>,
	Json(params): Json<ParamsForCreate<DosageInformationForCreate>>,
) -> Result<(StatusCode, Json<DataRestResult<DosageInformation>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, DRUG_DOSAGE_CREATE)?;
	let ParamsForCreate { data } = params;
	let mut data = data;
	data.drug_id = drug_id;

	let id = DosageInformationBmc::create(&ctx, &mm, data).await?;
	let entity = DosageInformationBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::CREATED, Json(DataRestResult { data: entity })))
}

/// GET /api/cases/{case_id}/drugs/{drug_id}/dosages
pub async fn list_dosage_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, drug_id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, Json<DataRestResult<Vec<DosageInformation>>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, DRUG_DOSAGE_LIST)?;
	let filter = DosageInformationFilter {
		drug_id: Some(OpValsValue::from(vec![OpValValue::Eq(json!(
			drug_id.to_string()
		))])),
		..Default::default()
	};
	let entities = DosageInformationBmc::list(
		&ctx,
		&mm,
		Some(vec![filter]),
		Some(ListOptions::default()),
	)
	.await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entities })))
}

/// GET /api/cases/{case_id}/drugs/{drug_id}/dosages/{id}
pub async fn get_dosage_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, drug_id, id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<(StatusCode, Json<DataRestResult<DosageInformation>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, DRUG_DOSAGE_READ)?;
	let entity = DosageInformationBmc::get(&ctx, &mm, id).await?;
	ensure_drug_scope(drug_id, entity.drug_id, id, "dosage_information")?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// PUT /api/cases/{case_id}/drugs/{drug_id}/dosages/{id}
pub async fn update_dosage_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, drug_id, id)): Path<(Uuid, Uuid, Uuid)>,
	Json(params): Json<ParamsForUpdate<DosageInformationForUpdate>>,
) -> Result<(StatusCode, Json<DataRestResult<DosageInformation>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, DRUG_DOSAGE_UPDATE)?;
	let ParamsForUpdate { data } = params;
	let entity = DosageInformationBmc::get(&ctx, &mm, id).await?;
	ensure_drug_scope(drug_id, entity.drug_id, id, "dosage_information")?;
	DosageInformationBmc::update(&ctx, &mm, id, data).await?;
	let entity = DosageInformationBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// DELETE /api/cases/{case_id}/drugs/{drug_id}/dosages/{id}
pub async fn delete_dosage_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, drug_id, id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode> {
	let ctx = ctx_w.0;
	require_permission(&ctx, DRUG_DOSAGE_DELETE)?;
	let entity = DosageInformationBmc::get(&ctx, &mm, id).await?;
	ensure_drug_scope(drug_id, entity.drug_id, id, "dosage_information")?;
	DosageInformationBmc::delete(&ctx, &mm, id).await?;
	Ok(StatusCode::NO_CONTENT)
}

// -- Drug Indications (G.k.6.r)

/// POST /api/cases/{case_id}/drugs/{drug_id}/indications
pub async fn create_drug_indication(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, drug_id)): Path<(Uuid, Uuid)>,
	Json(params): Json<ParamsForCreate<DrugIndicationForCreate>>,
) -> Result<(StatusCode, Json<DataRestResult<DrugIndication>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, DRUG_INDICATION_CREATE)?;
	let ParamsForCreate { data } = params;
	let mut data = data;
	data.drug_id = drug_id;

	let id = DrugIndicationBmc::create(&ctx, &mm, data).await?;
	let entity = DrugIndicationBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::CREATED, Json(DataRestResult { data: entity })))
}

/// GET /api/cases/{case_id}/drugs/{drug_id}/indications
pub async fn list_drug_indications(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, drug_id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, Json<DataRestResult<Vec<DrugIndication>>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, DRUG_INDICATION_LIST)?;
	let filter = DrugIndicationFilter {
		drug_id: Some(OpValsValue::from(vec![OpValValue::Eq(json!(
			drug_id.to_string()
		))])),
		..Default::default()
	};
	let entities = DrugIndicationBmc::list(
		&ctx,
		&mm,
		Some(vec![filter]),
		Some(ListOptions::default()),
	)
	.await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entities })))
}

/// GET /api/cases/{case_id}/drugs/{drug_id}/indications/{id}
pub async fn get_drug_indication(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, drug_id, id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<(StatusCode, Json<DataRestResult<DrugIndication>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, DRUG_INDICATION_READ)?;
	let entity = DrugIndicationBmc::get(&ctx, &mm, id).await?;
	ensure_drug_scope(drug_id, entity.drug_id, id, "drug_indications")?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// PUT /api/cases/{case_id}/drugs/{drug_id}/indications/{id}
pub async fn update_drug_indication(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, drug_id, id)): Path<(Uuid, Uuid, Uuid)>,
	Json(params): Json<ParamsForUpdate<DrugIndicationForUpdate>>,
) -> Result<(StatusCode, Json<DataRestResult<DrugIndication>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, DRUG_INDICATION_UPDATE)?;
	let ParamsForUpdate { data } = params;
	let entity = DrugIndicationBmc::get(&ctx, &mm, id).await?;
	ensure_drug_scope(drug_id, entity.drug_id, id, "drug_indications")?;
	DrugIndicationBmc::update(&ctx, &mm, id, data).await?;
	let entity = DrugIndicationBmc::get(&ctx, &mm, id).await?;
	Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
}

/// DELETE /api/cases/{case_id}/drugs/{drug_id}/indications/{id}
pub async fn delete_drug_indication(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path((_case_id, drug_id, id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode> {
	let ctx = ctx_w.0;
	require_permission(&ctx, DRUG_INDICATION_DELETE)?;
	let entity = DrugIndicationBmc::get(&ctx, &mm, id).await?;
	ensure_drug_scope(drug_id, entity.drug_id, id, "drug_indications")?;
	DrugIndicationBmc::delete(&ctx, &mm, id).await?;
	Ok(StatusCode::NO_CONTENT)
}
