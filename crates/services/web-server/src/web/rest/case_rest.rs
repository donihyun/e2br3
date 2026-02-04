use lib_core::model::acs::{
	CASE_CREATE, CASE_DELETE, CASE_LIST, CASE_READ, CASE_UPDATE, XML_EXPORT,
};
use lib_core::model::case::{CaseBmc, CaseFilter, CaseForCreate, CaseForUpdate};
use lib_core::xml::export_case_xml;
use axum::http::header;
use axum::response::Response;
use lib_web::middleware::mw_auth::CtxW;
use lib_rest_core::prelude::*;
use lib_rest_core::Error;
use tokio::runtime::Handle;
use tokio::task;

// This macro generates all 5 CRUD functions:
// - create_case
// - get_case
// - list_cases
// - update_case
// - delete_case
generate_common_rest_fns! {
	Bmc: CaseBmc,
	Entity: lib_core::model::case::Case,
	ForCreate: CaseForCreate,
	ForUpdate: CaseForUpdate,
	Filter: CaseFilter,
	Suffix: case,
	PermCreate: CASE_CREATE,
	PermRead: CASE_READ,
	PermUpdate: CASE_UPDATE,
	PermDelete: CASE_DELETE,
	PermList: CASE_LIST
}

pub async fn export_case(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(id): Path<Uuid>,
) -> Result<Response> {
	let ctx = ctx_w.0;
	require_permission(&ctx, XML_EXPORT)?;
	let ctx_clone = ctx.clone();
	let mm_clone = mm.clone();
	let xml = task::spawn_blocking(move || {
		Handle::current().block_on(export_case_xml(&ctx_clone, &mm_clone, id))
	})
	.await
	.map_err(|err| Error::BadRequest {
		message: format!("export task failed: {err}"),
	})??;
	let mut response = (StatusCode::OK, xml).into_response();
	response.headers_mut().insert(
		header::CONTENT_TYPE,
		header::HeaderValue::from_static("application/xml"),
	);
	Ok(response)
}
