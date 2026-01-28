use crate::error::{ClientError, Error, Result};
use crate::log::log_request;
use crate::middleware::mw_auth::CtxW;
use crate::middleware::mw_req_stamp::ReqStamp;

use axum::http::{Method, Uri};
use axum::response::{IntoResponse, Response};
use axum::Json;
use lib_core::model;
use serde_json::{json, to_value};
use std::sync::Arc;
use tracing::{debug, error};
use uuid::Uuid;
use lib_rest_core::prelude::StatusCode;

pub async fn mw_reponse_map(
	ctx: Result<CtxW>, // Axum 0.8 does not seem to support Option anymore
	uri: Uri,
	req_method: Method,
	req_stamp: ReqStamp,
	res: Response,
) -> Response {
	let ctx = ctx.map(|ctx| ctx.0).ok();

	debug!("{:<12} - mw_reponse_map", "RES_MAPPER");
	let uuid = Uuid::new_v4();

	// -- Get the eventual response error.
	let web_error = res.extensions().get::<Arc<Error>>().map(Arc::as_ref);
	let rest_error =
		res.extensions().get::<Arc<lib_rest_core::Error>>().map(Arc::as_ref);
	let debug_errors = std::env::var("E2BR3_DEBUG_ERRORS")
		.map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
		.unwrap_or(false);
	let mut debug_detail: Option<serde_json::Value> = None;
	let client_status_error = if let Some(err) = web_error {
		Some(err.client_status_and_error())
	} else if let Some(err) = rest_error {
		if debug_errors {
			debug_detail = Some(serde_json::Value::String(format!("{err:?}")));
		}
		let (status_code, client_error) = match err {
			lib_rest_core::Error::PermissionDenied { required_permission } => (
				StatusCode::FORBIDDEN,
				ClientError::PERMISSION_DENIED {
					required_permission: required_permission.clone(),
				},
			),
			lib_rest_core::Error::Model(model::Error::EntityNotFound { entity, id }) => (
				StatusCode::NOT_FOUND,
				ClientError::ENTITY_NOT_FOUND { entity, id: *id },
			),
			lib_rest_core::Error::Model(model::Error::EntityUuidNotFound { entity, id }) => (
				StatusCode::NOT_FOUND,
				ClientError::ENTITY_UUID_NOT_FOUND {
					entity,
					id: id.to_string(),
				},
			),
			lib_rest_core::Error::SerdeJson(_) => {
				(StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR)
			}
			_ => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR),
		};
		Some((status_code, client_error))
	} else {
		None
	};

	// -- If client error, build the new reponse.
	let error_response =
		client_status_error
			.as_ref()
			.map(|(status_code, client_error)| {
				let client_error = to_value(client_error).ok();
				let message = client_error.as_ref().and_then(|v| v.get("message"));
				let detail = debug_detail
					.as_ref()
					.or_else(|| client_error.as_ref().and_then(|v| v.get("detail")));

				let client_error_body = json!({
						"error": {
						"message": message, // Variant name
						"data": {
							"req_uuid": uuid.to_string(),
							"detail": detail
						},
					}
				});

				debug!("CLIENT ERROR BODY:\n{client_error_body}");

				// Build the new response from the client_error_body
				(*status_code, Json(client_error_body)).into_response()
			});

	// -- Build and log the server log line.
	let client_error = client_status_error.unzip().1;

	if let Err(err) =
		log_request(req_method, uri, req_stamp, ctx, web_error, client_error).await
	{
		error!("log_request failed: {err}");
	}

	debug!("\n");

	error_response.unwrap_or(res)
}
