mod common;

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use common::{cookie_header, init_test_mm, seed_org_with_users, Result};
use lib_auth::token::generate_web_token;
use serde_json::Value;
use serial_test::serial;
use tower::ServiceExt;

#[serial]
#[tokio::test]
async fn test_admin_can_list_validation_rules() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());

	let app = web_server::app(mm);

	let req = Request::builder()
		.method("GET")
		.uri("/api/validation/rules")
		.header("cookie", cookie.clone())
		.body(Body::empty())?;
	let res = app.clone().oneshot(req).await?;
	let etag = res
		.headers()
		.get(header::ETAG)
		.and_then(|v| v.to_str().ok())
		.ok_or("missing ETag header")?
		.to_string();
	let status = res.status();
	let body = to_bytes(res.into_body(), usize::MAX).await?;
	if status != StatusCode::OK {
		return Err(format!(
			"validation rules status {} body {}",
			status,
			String::from_utf8_lossy(&body)
		)
		.into());
	}

	let value: Value = serde_json::from_slice(&body)?;
	let rules = value
		.get("data")
		.and_then(Value::as_array)
		.ok_or("missing data array")?;
	assert!(!rules.is_empty(), "expected non-empty validation rule list");

	let has_known = rules.iter().any(|rule| {
		rule.get("code").and_then(Value::as_str) == Some("FDA.C.1.7.1.REQUIRED")
	});
	assert!(has_known, "expected FDA.C.1.7.1.REQUIRED in catalog");

	let req = Request::builder()
		.method("GET")
		.uri("/api/validation/rules?profile=fda")
		.header("cookie", cookie)
		.body(Body::empty())?;
	let res = app.clone().oneshot(req).await?;
	let status = res.status();
	let body = to_bytes(res.into_body(), usize::MAX).await?;
	if status != StatusCode::OK {
		return Err(format!(
			"validation rules fda status {} body {}",
			status,
			String::from_utf8_lossy(&body)
		)
		.into());
	}
	let value: Value = serde_json::from_slice(&body)?;
	let rules = value
		.get("data")
		.and_then(Value::as_array)
		.ok_or("missing data array for profile query")?;
	assert!(!rules.is_empty(), "expected non-empty filtered rule list");
	let contains_mfds = rules
		.iter()
		.any(|rule| rule.get("profile").and_then(Value::as_str) == Some("mfds"));
	assert!(!contains_mfds, "profile=fda should not include mfds rules");

	let req = Request::builder()
		.method("GET")
		.uri("/api/validation/rules")
		.header("cookie", cookie_header(&token.to_string()))
		.header(header::IF_NONE_MATCH, etag)
		.body(Body::empty())?;
	let res = app.clone().oneshot(req).await?;
	assert_eq!(res.status(), StatusCode::NOT_MODIFIED);

	Ok(())
}
