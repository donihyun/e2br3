mod common;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use common::{cookie_header, init_test_mm, seed_org_with_users, Result};
use lib_auth::token::generate_web_token;
use serial_test::serial;
use tower::ServiceExt;

#[serial]
#[tokio::test]
async fn test_admin_can_access_terminology_endpoints() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());

	let app = web_server::app(mm);

	let req = Request::builder()
		.method("GET")
		.uri("/api/terminology/meddra?q=test&limit=5")
		.header("cookie", cookie.clone())
		.body(Body::empty())?;
	let res = app.clone().oneshot(req).await?;
	if res.status() != StatusCode::OK {
		let status = res.status();
		let body = to_bytes(res.into_body(), usize::MAX).await?;
		return Err(format!(
			"meddra status {} body {}",
			status,
			String::from_utf8_lossy(&body)
		)
		.into());
	}

	let req = Request::builder()
		.method("GET")
		.uri("/api/terminology/whodrug?q=test&limit=5")
		.header("cookie", cookie.clone())
		.body(Body::empty())?;
	let res = app.clone().oneshot(req).await?;
	if res.status() != StatusCode::OK {
		let status = res.status();
		let body = to_bytes(res.into_body(), usize::MAX).await?;
		return Err(format!(
			"whodrug status {} body {}",
			status,
			String::from_utf8_lossy(&body)
		)
		.into());
	}

	let req = Request::builder()
		.method("GET")
		.uri("/api/terminology/countries")
		.header("cookie", cookie.clone())
		.body(Body::empty())?;
	let res = app.clone().oneshot(req).await?;
	if res.status() != StatusCode::OK {
		let status = res.status();
		let body = to_bytes(res.into_body(), usize::MAX).await?;
		return Err(format!(
			"countries status {} body {}",
			status,
			String::from_utf8_lossy(&body)
		)
		.into());
	}

	let req = Request::builder()
		.method("GET")
		.uri("/api/terminology/code-lists?list_name=report_type")
		.header("cookie", cookie)
		.body(Body::empty())?;
	let res = app.oneshot(req).await?;
	if res.status() != StatusCode::OK {
		let status = res.status();
		let body = to_bytes(res.into_body(), usize::MAX).await?;
		return Err(format!(
			"code-lists status {} body {}",
			status,
			String::from_utf8_lossy(&body)
		)
		.into());
	}

	Ok(())
}

#[serial]
#[tokio::test]
async fn test_viewer_cannot_access_terminology_endpoints() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.viewer.email, seed.viewer.token_salt)?;
	let cookie = cookie_header(&token.to_string());

	let app = web_server::app(mm);

	let req = Request::builder()
		.method("GET")
		.uri("/api/terminology/meddra?q=test&limit=5")
		.header("cookie", cookie)
		.body(Body::empty())?;
	let res = app.oneshot(req).await?;
	assert_eq!(res.status(), StatusCode::FORBIDDEN);

	Ok(())
}
