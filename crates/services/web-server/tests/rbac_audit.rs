mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{cookie_header, init_test_mm, seed_org_with_users, Result};
use lib_auth::token::generate_web_token;
use serial_test::serial;
use tower::ServiceExt;

#[serial]
#[tokio::test]
async fn test_admin_can_list_audit_logs() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let app = web_server::app(mm);

	let req = Request::builder()
		.method("GET")
		.uri("/api/audit-logs")
		.header("cookie", cookie_header(&token.to_string()))
		.body(Body::empty())?;
	let res = app.oneshot(req).await?;
	assert_eq!(res.status(), StatusCode::OK);
	Ok(())
}

#[serial]
#[tokio::test]
async fn test_viewer_cannot_list_audit_logs() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.viewer.email, seed.viewer.token_salt)?;
	let app = web_server::app(mm);

	let req = Request::builder()
		.method("GET")
		.uri("/api/audit-logs")
		.header("cookie", cookie_header(&token.to_string()))
		.body(Body::empty())?;
	let res = app.oneshot(req).await?;
	assert_eq!(res.status(), StatusCode::FORBIDDEN);
	Ok(())
}
