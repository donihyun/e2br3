mod common;

use common::{
	audit_log_count, create_case_fixture, demo_org_id, demo_user_id,
	delete_case_fixture, init_test_mm, set_current_user, Result,
};
use lib_core::ctx::Ctx;
use lib_core::model::audit::AuditLogBmc;
use lib_core::model::case::{CaseBmc, CaseForUpdate};
use serial_test::serial;

#[serial]
#[tokio::test]
async fn test_audit_trail_cases() -> Result<()> {
	let mm = init_test_mm().await;
	let ctx = Ctx::root_ctx();

	set_current_user(&mm, demo_user_id()).await?;
	let case_id = create_case_fixture(&mm, demo_org_id(), demo_user_id()).await?;

	assert_eq!(audit_log_count(&mm, "cases", case_id, "CREATE").await?, 1);

	let case_u = CaseForUpdate {
		safety_report_id: None,
		status: Some("validated".to_string()),
		updated_by: Some(demo_user_id()),
		submitted_by: None,
		submitted_at: None,
	};
	CaseBmc::update(&ctx, &mm, case_id, case_u).await?;

	assert_eq!(audit_log_count(&mm, "cases", case_id, "UPDATE").await?, 1);

	CaseBmc::delete(&ctx, &mm, case_id).await?;
	assert_eq!(audit_log_count(&mm, "cases", case_id, "DELETE").await?, 1);

	let logs = AuditLogBmc::list_by_record(&ctx, &mm, "cases", case_id).await?;
	assert!(logs.iter().any(|l| l.action == "CREATE"));
	assert!(logs.iter().any(|l| l.action == "UPDATE"));
	assert!(logs.iter().any(|l| l.action == "DELETE"));

	delete_case_fixture(&mm, case_id).await.ok();
	Ok(())
}
