use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use lib_auth::token::generate_web_token;
use lib_core::_dev_utils;
use lib_core::ctx::{ROLE_ADMIN, ROLE_VIEWER, SYSTEM_ORG_ID, SYSTEM_USER_ID};
use lib_core::model::store::set_org_context_dbx;
use lib_core::model::ModelManager;
use lib_web::Error as WebError;
use serde_json::Value;
use serial_test::serial;
use tower::ServiceExt;
use uuid::Uuid;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

async fn enable_rls(mm: &ModelManager) -> Result<()> {
	let query = sqlx::query("SET ROLE e2br3_app_role");
	mm.dbx().execute(query).await?;
	let query = sqlx::query("SET row_security = on");
	mm.dbx().execute(query).await?;
	Ok(())
}

fn system_user_id() -> Uuid {
	Uuid::parse_str(SYSTEM_USER_ID).expect("system user id")
}

fn system_org_id() -> Uuid {
	Uuid::parse_str(SYSTEM_ORG_ID).expect("system org id")
}

async fn insert_org(mm: &ModelManager, created_by: Uuid) -> Result<Uuid> {
	let org_id = Uuid::new_v4();
	let query = sqlx::query(
		"INSERT INTO organizations (id, name, org_type, address, contact_email, created_by, updated_by)
		 VALUES ($1, $2, $3, $4, $5, $6, $6)",
	)
	.bind(org_id)
	.bind(format!("RLS Org {org_id}"))
	.bind("internal")
	.bind("123 RLS St")
	.bind(format!("rls-org-{org_id}@example.com"))
	.bind(created_by);
	mm.dbx().execute(query).await?;
	Ok(org_id)
}

async fn insert_user(
	mm: &ModelManager,
	org_id: Uuid,
	created_by: Uuid,
) -> Result<(Uuid, String, Uuid)> {
	let user_id = Uuid::new_v4();
	let token_salt = Uuid::new_v4();
	let pwd_salt = Uuid::new_v4();
	let email = format!("rls-user-{user_id}@example.com");
	let username = format!("rls_user_{user_id}");
	let query = sqlx::query(
		"INSERT INTO users (id, organization_id, email, username, pwd_salt, token_salt, role, active, created_by, updated_by)
		 VALUES ($1, $2, $3, $4, $5, $6, $7, true, $8, $8)",
	)
	.bind(user_id)
	.bind(org_id)
	.bind(&email)
	.bind(username)
	.bind(pwd_salt)
	.bind(token_salt)
	.bind(ROLE_VIEWER)
	.bind(created_by);
	mm.dbx().execute(query).await?;
	Ok((user_id, email, token_salt))
}

async fn insert_case(
	mm: &ModelManager,
	org_id: Uuid,
	created_by: Uuid,
) -> Result<Uuid> {
	let case_id = Uuid::new_v4();
	let query = sqlx::query(
		"INSERT INTO cases (id, organization_id, safety_report_id, created_by, updated_by)
		 VALUES ($1, $2, $3, $4, $4)",
	)
	.bind(case_id)
	.bind(org_id)
	.bind(format!("SR-TEST-{case_id}"))
	.bind(created_by);
	mm.dbx().execute(query).await?;
	Ok(case_id)
}

#[serial]
#[tokio::test]
async fn test_rls_list_users_filters_org() -> Result<()> {
	std::env::set_var("SERVICE_DB_URL", "postgres://app_user:dev_only_pwd@localhost/app_db");
	std::env::set_var("SERVICE_WEB_FOLDER", "web-folder");
	std::env::set_var("SERVICE_PWD_KEY", "ZmFrZV9rZXk");
	std::env::set_var("SERVICE_TOKEN_KEY", "ZmFrZV9rZXk");
	std::env::set_var("SERVICE_TOKEN_DURATION_SEC", "3600");
	std::env::set_var("E2BR3_DEVDB_SKIP_ROLE_SETUP", "0");
	std::env::set_var("E2BR3_DB_ROLE", "e2br3_app_role");

	_dev_utils::init_dev().await;
	let mm = ModelManager::new().await?;

	let dbx = mm.dbx();
	dbx.begin_txn().await?;
	enable_rls(&mm).await?;
	set_org_context_dbx(dbx, system_org_id(), ROLE_ADMIN).await?;
	let query = sqlx::query("SELECT set_current_user_context($1)")
		.bind(system_user_id());
	mm.dbx().execute(query).await?;
	let org1_id = insert_org(&mm, system_user_id()).await?;
	let org2_id = insert_org(&mm, system_user_id()).await?;

	let (user1_id, user1_email, user1_salt) =
		insert_user(&mm, org1_id, system_user_id()).await?;
	let (user2_id, _, _) = insert_user(&mm, org2_id, system_user_id()).await?;
	dbx.commit_txn().await?;

	let token = generate_web_token(&user1_email, user1_salt)?;
	let cookie = format!("auth-token={}", token);

	let app = web_server::app(mm);
	let req = Request::builder()
		.method("GET")
		.uri("/api/users")
		.header("cookie", &cookie)
		.body(Body::empty())?;
	let res = app.clone().oneshot(req).await?;
	assert_eq!(res.status(), StatusCode::OK);

	let body = to_bytes(res.into_body(), usize::MAX).await?;
	let value: Value = serde_json::from_slice(&body)?;
	let users = value
		.get("data")
		.and_then(|v| v.as_array())
		.ok_or("missing data array")?;

	let user1_id = user1_id.to_string();
	let user2_id = user2_id.to_string();
	assert!(users.iter().any(|u| u.get("id").and_then(|v| v.as_str()) == Some(&user1_id)));
	assert!(!users.iter().any(|u| u.get("id").and_then(|v| v.as_str()) == Some(&user2_id)));

	let req = Request::builder()
		.method("GET")
		.uri(format!("/api/users/{user2_id}"))
		.header("cookie", &cookie)
		.body(Body::empty())?;
	let res = app.oneshot(req).await?;
	let status = res.status();
	if status != StatusCode::BAD_REQUEST {
		let err = res
			.extensions()
			.get::<std::sync::Arc<WebError>>()
			.map(|e| format!("{e:?}"));
		let body = to_bytes(res.into_body(), usize::MAX).await?;
		return Err(format!(
			"user get status {} body {} err {:?}",
			status,
			String::from_utf8_lossy(&body),
			err
		)
		.into());
	}

	Ok(())
}

#[serial]
#[tokio::test]
async fn test_rls_list_cases_filters_org() -> Result<()> {
	std::env::set_var("SERVICE_DB_URL", "postgres://app_user:dev_only_pwd@localhost/app_db");
	std::env::set_var("SERVICE_WEB_FOLDER", "web-folder");
	std::env::set_var("SERVICE_PWD_KEY", "ZmFrZV9rZXk");
	std::env::set_var("SERVICE_TOKEN_KEY", "ZmFrZV9rZXk");
	std::env::set_var("SERVICE_TOKEN_DURATION_SEC", "3600");
	std::env::set_var("E2BR3_DEVDB_SKIP_ROLE_SETUP", "0");
	std::env::set_var("E2BR3_DB_ROLE", "e2br3_app_role");

	_dev_utils::init_dev().await;
	let mm = ModelManager::new().await?;

	let dbx = mm.dbx();
	dbx.begin_txn().await?;
	enable_rls(&mm).await?;
	set_org_context_dbx(dbx, system_org_id(), ROLE_ADMIN).await?;
	let query = sqlx::query("SELECT set_current_user_context($1)")
		.bind(system_user_id());
	mm.dbx().execute(query).await?;

	let org1_id = insert_org(&mm, system_user_id()).await?;
	let org2_id = insert_org(&mm, system_user_id()).await?;
	let (_user1_id, user1_email, user1_salt) =
		insert_user(&mm, org1_id, system_user_id()).await?;
	let _user2_id = insert_user(&mm, org2_id, system_user_id()).await?;
	let case_org1 = insert_case(&mm, org1_id, system_user_id()).await?;
	let case_org2 = insert_case(&mm, org2_id, system_user_id()).await?;
	dbx.commit_txn().await?;

	let token = generate_web_token(&user1_email, user1_salt)?;
	let cookie = format!("auth-token={}", token);

	let app = web_server::app(mm);
	let req = Request::builder()
		.method("GET")
		.uri("/api/cases")
		.header("cookie", &cookie)
		.body(Body::empty())?;
	let res = app.clone().oneshot(req).await?;
	assert_eq!(res.status(), StatusCode::OK);

	let body = to_bytes(res.into_body(), usize::MAX).await?;
	let value: Value = serde_json::from_slice(&body)?;
	let cases = value
		.get("data")
		.and_then(|v| v.as_array())
		.ok_or("missing data array")?;

	let case_org1 = case_org1.to_string();
	let case_org2 = case_org2.to_string();
	assert!(cases.iter().any(|c| c.get("id").and_then(|v| v.as_str()) == Some(&case_org1)));
	assert!(!cases.iter().any(|c| c.get("id").and_then(|v| v.as_str()) == Some(&case_org2)));

	let req = Request::builder()
		.method("GET")
		.uri(format!("/api/cases/{case_org2}"))
		.header("cookie", &cookie)
		.body(Body::empty())?;
	let res = app.oneshot(req).await?;
	let status = res.status();
	if status != StatusCode::BAD_REQUEST {
		let err = res
			.extensions()
			.get::<std::sync::Arc<WebError>>()
			.map(|e| format!("{e:?}"));
		let body = to_bytes(res.into_body(), usize::MAX).await?;
		return Err(format!(
			"case get status {} body {} err {:?}",
			status,
			String::from_utf8_lossy(&body),
			err
		)
		.into());
	}

	Ok(())
}
