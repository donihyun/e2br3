mod common;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use common::{cookie_header, init_test_mm, seed_org_with_users, Result};
use lib_auth::token::generate_web_token;
use serde_json::{json, Value};
use serial_test::serial;
use tower::ServiceExt;
use uuid::Uuid;

async fn create_case(
	app: &axum::Router,
	cookie: &str,
	org_id: Uuid,
) -> Result<Uuid> {
	let body = json!({
		"data": {
			"organization_id": org_id,
			"safety_report_id": format!("SR-{}", Uuid::new_v4()),
			"status": "draft"
		}
	});
	let req = Request::builder()
		.method("POST")
		.uri("/api/cases")
		.header("cookie", cookie)
		.header("content-type", "application/json")
		.body(Body::from(body.to_string()))?;
	let res = app.clone().oneshot(req).await?;
	let status = res.status();
	let body = to_bytes(res.into_body(), usize::MAX).await?;
	if status != StatusCode::CREATED {
		return Err(format!(
			"create case status {} body {}",
			status,
			String::from_utf8_lossy(&body)
		)
		.into());
	}
	let value: Value = serde_json::from_slice(&body)?;
	let id = value
		.get("data")
		.and_then(|v| v.get("id"))
		.and_then(|v| v.as_str())
		.ok_or("missing data.id")?;
	Ok(Uuid::parse_str(id)?)
}

async fn create_safety_report(
	app: &axum::Router,
	cookie: &str,
	case_id: Uuid,
) -> Result<()> {
	let body = json!({
		"data": {
			"case_id": case_id,
			"transmission_date": [2024, 1],
			"report_type": "1",
			"date_first_received_from_source": [2024, 1],
			"date_of_most_recent_information": [2024, 1],
			"fulfil_expedited_criteria": false
		}
	});
	let req = Request::builder()
		.method("POST")
		.uri(format!("/api/cases/{case_id}/safety-report"))
		.header("cookie", cookie)
		.header("content-type", "application/json")
		.body(Body::from(body.to_string()))?;
	let res = app.clone().oneshot(req).await?;
	let status = res.status();
	let body = to_bytes(res.into_body(), usize::MAX).await?;
	if status != StatusCode::CREATED {
		return Err(format!(
			"create safety report status {} body {}",
			status,
			String::from_utf8_lossy(&body)
		)
		.into());
	}
	Ok(())
}

async fn create_sender(
	app: &axum::Router,
	cookie: &str,
	case_id: Uuid,
	sender_type: &str,
) -> Result<()> {
	let body = json!({
		"data": {
			"case_id": case_id,
			"sender_type": sender_type,
			"organization_name": "Test Sender Org"
		}
	});
	let req = Request::builder()
		.method("POST")
		.uri(format!("/api/cases/{case_id}/safety-report/senders"))
		.header("cookie", cookie)
		.header("content-type", "application/json")
		.body(Body::from(body.to_string()))?;
	let res = app.clone().oneshot(req).await?;
	let status = res.status();
	let body = to_bytes(res.into_body(), usize::MAX).await?;
	if status != StatusCode::CREATED {
		return Err(format!(
			"create sender status {} body {}",
			status,
			String::from_utf8_lossy(&body)
		)
		.into());
	}
	Ok(())
}

async fn create_message_header(
	app: &axum::Router,
	cookie: &str,
	case_id: Uuid,
) -> Result<()> {
	let body = json!({
		"data": {
			"case_id": case_id,
			"message_number": format!("MSG-{case_id}"),
			"message_sender_identifier": "SENDER01",
			"message_receiver_identifier": "RECEIVER01",
			"message_date": "20240201010101"
		}
	});
	let req = Request::builder()
		.method("POST")
		.uri(format!("/api/cases/{case_id}/message-header"))
		.header("cookie", cookie)
		.header("content-type", "application/json")
		.body(Body::from(body.to_string()))?;
	let res = app.clone().oneshot(req).await?;
	let status = res.status();
	let body = to_bytes(res.into_body(), usize::MAX).await?;
	if status != StatusCode::CREATED {
		return Err(format!(
			"create message header status {} body {}",
			status,
			String::from_utf8_lossy(&body)
		)
		.into());
	}
	Ok(())
}

async fn update_message_header_receiver(
	app: &axum::Router,
	cookie: &str,
	case_id: Uuid,
	batch_receiver_identifier: &str,
) -> Result<()> {
	let body = json!({
		"data": {
			"batch_receiver_identifier": batch_receiver_identifier
		}
	});
	let req = Request::builder()
		.method("PUT")
		.uri(format!("/api/cases/{case_id}/message-header"))
		.header("cookie", cookie)
		.header("content-type", "application/json")
		.body(Body::from(body.to_string()))?;
	let res = app.clone().oneshot(req).await?;
	let status = res.status();
	let body = to_bytes(res.into_body(), usize::MAX).await?;
	if status != StatusCode::OK {
		return Err(format!(
			"update message header status {} body {}",
			status,
			String::from_utf8_lossy(&body)
		)
		.into());
	}
	Ok(())
}

async fn get_validation(
	app: &axum::Router,
	cookie: &str,
	uri: &str,
) -> Result<(StatusCode, Value)> {
	let req = Request::builder()
		.method("GET")
		.uri(uri)
		.header("cookie", cookie)
		.body(Body::empty())?;
	let res = app.clone().oneshot(req).await?;
	let status = res.status();
	let body = to_bytes(res.into_body(), usize::MAX).await?;
	let value = serde_json::from_slice::<Value>(&body)?;
	Ok((status, value))
}

#[serial]
#[tokio::test]
async fn test_validation_defaults_to_fda_profile() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);

	let case_id = create_case(&app, &cookie, seed.org_id).await?;
	let (status, body) =
		get_validation(&app, &cookie, &format!("/api/cases/{case_id}/validation"))
			.await?;

	assert_eq!(status, StatusCode::OK);
	assert_eq!(body["data"]["profile"], "fda");
	Ok(())
}

#[serial]
#[tokio::test]
async fn test_validation_supports_mfds_profile() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);

	let case_id = create_case(&app, &cookie, seed.org_id).await?;
	create_safety_report(&app, &cookie, case_id).await?;
	create_sender(&app, &cookie, case_id, "3").await?;

	let (status, body) = get_validation(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/validation?profile=mfds"),
	)
	.await?;

	assert_eq!(status, StatusCode::OK);
	assert_eq!(body["data"]["profile"], "mfds");
	assert!(
		body["data"]["issues"]
			.as_array()
			.map(|items| {
				items
					.iter()
					.any(|issue| issue["code"] == "MFDS.C.3.1.KR.1.REQUIRED")
			})
			.unwrap_or(false),
		"expected MFDS sender KR issue, body={body}"
	);
	Ok(())
}

#[serial]
#[tokio::test]
async fn test_validation_rejects_unknown_profile() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);

	let case_id = create_case(&app, &cookie, seed.org_id).await?;
	let (status, body) = get_validation(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/validation?profile=unknown"),
	)
	.await?;

	assert_eq!(status, StatusCode::BAD_REQUEST);
	assert!(
		body.to_string().contains("invalid validation profile"),
		"unexpected body={body}"
	);
	Ok(())
}

#[serial]
#[tokio::test]
async fn test_validation_infers_mfds_profile_from_batch_receiver() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);

	let case_id = create_case(&app, &cookie, seed.org_id).await?;
	create_safety_report(&app, &cookie, case_id).await?;
	create_message_header(&app, &cookie, case_id).await?;
	update_message_header_receiver(&app, &cookie, case_id, "ZZMFDS").await?;
	create_sender(&app, &cookie, case_id, "3").await?;

	let (status, body) =
		get_validation(&app, &cookie, &format!("/api/cases/{case_id}/validation"))
			.await?;

	assert_eq!(status, StatusCode::OK);
	assert_eq!(body["data"]["profile"], "mfds");
	assert!(
		body["data"]["issues"]
			.as_array()
			.map(|items| {
				items
					.iter()
					.any(|issue| issue["code"] == "MFDS.C.3.1.KR.1.REQUIRED")
			})
			.unwrap_or(false),
		"expected MFDS issue from inferred profile, body={body}"
	);
	Ok(())
}
