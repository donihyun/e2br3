mod common;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use common::{cookie_header, init_test_mm, seed_org_with_users, Result};
use lib_auth::token::generate_web_token;
use serial_test::serial;
use serde_json::Value;
use tower::ServiceExt;

#[serial]
#[tokio::test]
async fn test_import_then_export_xml() -> Result<()> {
	let Some(examples_dir) = std::env::var("E2BR3_EXAMPLES_DIR").ok() else {
		eprintln!("E2BR3_EXAMPLES_DIR not set; skipping xml import/export test");
		return Ok(());
	};
	if std::env::var("E2BR3_XSD_PATH").is_err() {
		eprintln!("E2BR3_XSD_PATH not set; skipping xml import/export test");
		return Ok(());
	}

	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "admin_pwd", "viewer_pwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);

	let xml_path = std::path::Path::new(&examples_dir)
		.join("1-1_ExampleCase_literature_initial_v1_0.xml");
	let mut xml = std::fs::read_to_string(xml_path)?;
	let unique_safety_report_id =
		format!("DSJP-TEST-{}", uuid::Uuid::new_v4());
	let marker = "extension=\"DSJP-B0123456-Lit1\"";
	if xml.contains(marker) {
		xml = xml.replace(marker, &format!("extension=\"{unique_safety_report_id}\""));
	} else {
		return Err("failed to locate safety_report_id marker in example XML".into());
	}

	let boundary = "X-BOUNDARY-XML-IMPORT";
	let body = format!(
		"--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"case.xml\"\r\nContent-Type: application/xml\r\n\r\n{xml}\r\n--{boundary}--\r\n"
	);

	let req = Request::builder()
		.method("POST")
		.uri("/api/import/xml")
		.header("content-type", format!("multipart/form-data; boundary={boundary}"))
		.header("cookie", cookie.clone())
		.body(Body::from(body))?;

	let res = app.clone().oneshot(req).await?;
	let status = res.status();
	let body = to_bytes(res.into_body(), usize::MAX).await?;
	if status != StatusCode::OK {
		return Err(format!(
			"import status {} body {}",
			status,
			String::from_utf8_lossy(&body)
		)
		.into());
	}
	let value: Value = serde_json::from_slice(&body)?;
	let case_id = value
		.get("data")
		.and_then(|v| v.get("case_id"))
		.and_then(|v| v.as_str())
		.ok_or("missing case_id in import response")?;

	let update_body = serde_json::json!({
		"data": {
			"status": "validated"
		}
	});
	let req = Request::builder()
		.method("PUT")
		.uri(format!("/api/cases/{case_id}"))
		.header("content-type", "application/json")
		.header("cookie", cookie.clone())
		.body(Body::from(update_body.to_string()))?;
	let res = app.clone().oneshot(req).await?;
	let status = res.status();
	let body = to_bytes(res.into_body(), usize::MAX).await?;
	if status != StatusCode::OK {
		return Err(format!(
			"update status {} body {}",
			status,
			String::from_utf8_lossy(&body)
		)
		.into());
	}

	let safety_body = serde_json::json!({
		"data": {
			"case_id": case_id,
			"transmission_date": [2024, 15],
			"report_type": "1",
			"date_first_received_from_source": [2024, 10],
			"date_of_most_recent_information": [2024, 15],
			"fulfil_expedited_criteria": true
		}
	});
	let req = Request::builder()
		.method("POST")
		.uri(format!("/api/cases/{case_id}/safety-report"))
		.header("content-type", "application/json")
		.header("cookie", cookie.clone())
		.body(Body::from(safety_body.to_string()))?;
	let res = app.clone().oneshot(req).await?;
	let status = res.status();
	let body = to_bytes(res.into_body(), usize::MAX).await?;
	if status != StatusCode::OK && status != StatusCode::CREATED {
		return Err(format!(
			"safety report status {} body {}",
			status,
			String::from_utf8_lossy(&body)
		)
		.into());
	}

	let req = Request::builder()
		.method("GET")
		.uri(format!("/api/cases/{case_id}/export/xml"))
		.header("cookie", cookie)
		.body(Body::empty())?;

	let res = app.oneshot(req).await?;
	let status = res.status();
	let body = to_bytes(res.into_body(), usize::MAX).await?;
	if status != StatusCode::OK {
		return Err(format!(
			"export status {} body {}",
			status,
			String::from_utf8_lossy(&body)
		)
		.into());
	}
	let xml = String::from_utf8_lossy(&body);
	assert!(xml.contains("<MCCI_IN200100UV01"));

	Ok(())
}
