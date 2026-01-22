mod common;

use common::{
	create_case_fixture, demo_org_id, demo_user_id, init_test_mm, set_current_user,
	Result,
};
use lib_core::ctx::Ctx;
use lib_core::model::case::{CaseBmc, CaseForUpdate};
use serial_test::serial;

#[serial]
#[tokio::test]
async fn test_case_crud() -> Result<()> {
	let mm = init_test_mm().await;
	let ctx = Ctx::root_ctx();

	set_current_user(&mm, demo_user_id()).await?;
	let case_id = create_case_fixture(&mm, demo_org_id(), demo_user_id()).await?;

	let case = CaseBmc::get(&ctx, &mm, case_id).await?;
	assert_eq!(case.id, case_id);

	let cases = CaseBmc::list(&ctx, &mm, None, None).await?;
	assert!(cases.iter().any(|c| c.id == case_id));

	let case_u = CaseForUpdate {
		safety_report_id: None,
		status: Some("validated".to_string()),
		submitted_by: None,
		submitted_at: None,
	};
	CaseBmc::update(&ctx, &mm, case_id, case_u).await?;
	let case = CaseBmc::get(&ctx, &mm, case_id).await?;
	assert_eq!(case.status, "validated");

	CaseBmc::delete(&ctx, &mm, case_id).await?;
	Ok(())
}
