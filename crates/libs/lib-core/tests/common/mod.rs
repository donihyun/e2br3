use lib_core::_dev_utils;
use lib_core::ctx::Ctx;
use lib_core::model::ModelManager;
use sqlx::types::Uuid;
use std::env;

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

/// Default demo role for testing (admin has full access)
#[allow(dead_code)]
pub const DEMO_ROLE: &str = "admin";

#[allow(dead_code)]
pub async fn init_test_mm() -> ModelManager {
	env::set_var("E2BR3_DEVDB_SKIP_ROLE_SETUP", "0");
	env::set_var("E2BR3_TEST_CURRENT_USER_ID", demo_user_id().to_string());
	_dev_utils::init_dev().await;
	ModelManager::new().await.unwrap()
}

#[allow(dead_code)]
pub fn unique_suffix() -> String {
	Uuid::new_v4().to_string()
}

#[allow(dead_code)]
pub fn demo_org_id() -> Uuid {
	// NOTE: Seeded by sql/dev_initial/00-recreate-db.sql
	uuid::uuid!("00000000-0000-0000-0000-000000000001")
}

#[allow(dead_code)]
pub fn demo_user_id() -> Uuid {
	// NOTE: Seeded by sql/dev_initial/13-e2br3-seed.sql
	uuid::uuid!("11111111-1111-1111-1111-111111111111")
}

/// Creates a demo context for testing with admin role
#[allow(dead_code)]
pub fn demo_ctx() -> Ctx {
	Ctx::new(demo_user_id(), demo_org_id(), DEMO_ROLE.to_string())
		.expect("Failed to create demo context")
}

#[allow(dead_code)]
pub async fn set_current_user(mm: &ModelManager, user_id: Uuid) -> Result<()> {
	sqlx::query("SELECT set_config('app.current_user_id', $1, false)")
		.bind(user_id.to_string())
		.execute(mm.dbx().db())
		.await?;
	Ok(())
}

#[allow(dead_code)]
pub async fn create_case_fixture(
	mm: &ModelManager,
	org_id: Uuid,
	user_id: Uuid,
) -> Result<Uuid> {
	let case_id = Uuid::new_v4();
	let safety_report_id = format!("SR-TEST-{case_id}");

	let mut tx = mm.dbx().db().begin().await?;
	sqlx::query("SELECT set_current_user_context($1)")
		.bind(user_id)
		.execute(&mut *tx)
		.await?;
	sqlx::query(
		"INSERT INTO cases (id, organization_id, safety_report_id, version, status, created_by, updated_by, submitted_by, submitted_at, created_at, updated_at)
		 VALUES ($1, $2, $3, 1, 'draft', $4, $4, $4, NOW(), NOW(), NOW())",
	)
	.bind(case_id)
	.bind(org_id)
	.bind(safety_report_id)
	.bind(user_id)
	.execute(&mut *tx)
	.await?;
	tx.commit().await?;

	Ok(case_id)
}

#[allow(dead_code)]
pub async fn delete_case_fixture(mm: &ModelManager, case_id: Uuid) -> Result<()> {
	sqlx::query("DELETE FROM cases WHERE id = $1")
		.bind(case_id)
		.execute(mm.dbx().db())
		.await?;
	Ok(())
}

#[allow(dead_code)]
pub async fn audit_log_count(
	mm: &ModelManager,
	table_name: &str,
	record_id: Uuid,
	action: &str,
) -> Result<i64> {
	let count: i64 = sqlx::query_scalar(
		"SELECT COUNT(*) FROM audit_logs WHERE table_name = $1 AND record_id = $2 AND action = $3",
	)
	.bind(table_name)
	.bind(record_id)
	.bind(action)
	.fetch_one(mm.dbx().db())
	.await?;
	Ok(count)
}
