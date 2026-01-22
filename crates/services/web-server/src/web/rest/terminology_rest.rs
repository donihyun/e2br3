// Terminology REST endpoints for MedDRA, WHODrug, ISO Countries, E2B Code Lists

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use lib_core::model::terminology::{
	E2bCodeList, E2bCodeListBmc, IsoCountry, IsoCountryBmc, MeddraTerm,
	MeddraTermBmc, WhodrugProduct, WhodrugProductBmc,
};
use lib_core::model::ModelManager;
use lib_rest_core::rest_result::DataRestResult;
use lib_rest_core::Result;
use lib_web::middleware::mw_auth::CtxW;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TerminologySearchParams {
	pub q: String,
	#[serde(default = "default_limit")]
	pub limit: i64,
	pub version: Option<String>,
}

fn default_limit() -> i64 {
	20
}

#[derive(Deserialize)]
pub struct CodeListParams {
	pub list_name: String,
}

/// GET /api/terminology/meddra?q={term}&limit={count}&version={version}
/// Search MedDRA terms by name
pub async fn search_meddra(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Query(params): Query<TerminologySearchParams>,
) -> Result<(StatusCode, Json<DataRestResult<Vec<MeddraTerm>>>)> {
	let ctx = ctx_w.0;
	tracing::debug!(
		"{:<12} - rest search_meddra q={} limit={}",
		"HANDLER",
		params.q,
		params.limit
	);

	let terms = MeddraTermBmc::search(
		&ctx,
		&mm,
		&params.q,
		params.version.as_deref(),
		params.limit,
	)
	.await?;

	Ok((StatusCode::OK, Json(DataRestResult { data: terms })))
}

/// GET /api/terminology/whodrug?q={term}&limit={count}
/// Search WHODrug products by name
pub async fn search_whodrug(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Query(params): Query<TerminologySearchParams>,
) -> Result<(StatusCode, Json<DataRestResult<Vec<WhodrugProduct>>>)> {
	let ctx = ctx_w.0;
	tracing::debug!(
		"{:<12} - rest search_whodrug q={} limit={}",
		"HANDLER",
		params.q,
		params.limit
	);

	let products =
		WhodrugProductBmc::search(&ctx, &mm, &params.q, params.limit).await?;

	Ok((StatusCode::OK, Json(DataRestResult { data: products })))
}

/// GET /api/terminology/countries
/// List all active ISO countries
pub async fn list_countries(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
) -> Result<(StatusCode, Json<DataRestResult<Vec<IsoCountry>>>)> {
	let ctx = ctx_w.0;
	tracing::debug!("{:<12} - rest list_countries", "HANDLER");

	let countries = IsoCountryBmc::list_all(&ctx, &mm).await?;

	Ok((StatusCode::OK, Json(DataRestResult { data: countries })))
}

/// GET /api/terminology/code-lists?list_name={name}
/// Get E2B code list values by list name
pub async fn get_code_list(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Query(params): Query<CodeListParams>,
) -> Result<(StatusCode, Json<DataRestResult<Vec<E2bCodeList>>>)> {
	let ctx = ctx_w.0;
	tracing::debug!(
		"{:<12} - rest get_code_list list_name={}",
		"HANDLER",
		params.list_name
	);

	let codes =
		E2bCodeListBmc::get_by_list_name(&ctx, &mm, &params.list_name).await?;

	Ok((StatusCode::OK, Json(DataRestResult { data: codes })))
}
