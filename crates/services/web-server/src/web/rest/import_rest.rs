use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::Json;
use lib_core::model::acs::XML_IMPORT;
use lib_core::model::ModelManager;
use lib_core::xml::xml_validation::{
	should_skip_xml_validation, validate_e2b_xml_basic,
};
use lib_core::xml::{
	import_e2b_xml, validate_e2b_xml, XmlImportRequest, XmlValidationReport,
};
use lib_rest_core::rest_result::DataRestResult;
use lib_rest_core::{require_permission, Error, Result};
use lib_web::middleware::mw_auth::CtxW;

async fn read_xml_multipart(mut multipart: Multipart) -> Result<Vec<u8>> {
	while let Some(field) =
		multipart
			.next_field()
			.await
			.map_err(|err| Error::BadRequest {
				message: format!("multipart error: {err}"),
			})? {
		let name = field.name().map(|v| v.to_string());
		if name.as_deref() == Some("file") || name.as_deref() == Some("xml") {
			let bytes = field.bytes().await.map_err(|err| Error::BadRequest {
				message: format!("multipart read error: {err}"),
			})?;
			return Ok(bytes.to_vec());
		}
	}

	Err(Error::BadRequest {
		message: "missing xml file field".to_string(),
	})
}

/// POST /api/import/xml/validate
/// Validates E2B(R3) XML payload (XSD-only for now)
pub async fn validate_xml(
	State(_mm): State<ModelManager>,
	ctx_w: CtxW,
	multipart: Multipart,
) -> Result<(StatusCode, Json<DataRestResult<XmlValidationReport>>)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, XML_IMPORT)?;

	let xml = read_xml_multipart(multipart).await?;
	let report = if should_skip_xml_validation() {
		// Keep local dev usable even when XSD files are not mounted/available.
		validate_e2b_xml_basic(&xml, None)?
	} else {
		validate_e2b_xml(&xml, None)?
	};

	Ok((StatusCode::OK, Json(DataRestResult { data: report })))
}

/// POST /api/import/xml
/// Parse + import E2B(R3) XML (pipeline WIP)
pub async fn import_xml(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	multipart: Multipart,
) -> Result<(
	StatusCode,
	Json<DataRestResult<lib_core::xml::XmlImportResult>>,
)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, XML_IMPORT)?;

	let xml = read_xml_multipart(multipart).await?;
	let result = import_e2b_xml(
		&ctx,
		&mm,
		XmlImportRequest {
			xml,
			filename: None,
		},
	)
	.await?;

	Ok((StatusCode::OK, Json(DataRestResult { data: result })))
}
