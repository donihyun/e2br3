mod common;

use common::{
	create_case_fixture, demo_org_id, demo_user_id, init_test_mm,
	set_current_user, Result,
};
use lib_core::ctx::Ctx;
use lib_core::model::case::CaseBmc;
use lib_core::model::narrative::{
	NarrativeInformationBmc, NarrativeInformationForCreate,
	NarrativeInformationForUpdate,
};
use serial_test::serial;

#[serial]
#[tokio::test]
async fn test_narrative_information_crud() -> Result<()> {
	let mm = init_test_mm().await;
	let ctx = Ctx::root_ctx();

	set_current_user(&mm, demo_user_id()).await?;
	let case_id = create_case_fixture(&mm, demo_org_id(), demo_user_id()).await?;

	let narrative_c = NarrativeInformationForCreate {
		case_id,
		case_narrative: "Initial narrative".to_string(),
	};
	let narrative_id = NarrativeInformationBmc::create(&ctx, &mm, narrative_c).await?;
	let narrative = NarrativeInformationBmc::get_by_case(&ctx, &mm, case_id).await?;
	assert_eq!(narrative.id, narrative_id);

	let narrative_u = NarrativeInformationForUpdate {
		case_narrative: Some("Updated narrative".to_string()),
		reporter_comments: None,
		sender_comments: None,
	};
	NarrativeInformationBmc::update_by_case(&ctx, &mm, case_id, narrative_u)
		.await?;
	let narrative = NarrativeInformationBmc::get_by_case(&ctx, &mm, case_id).await?;
	assert_eq!(narrative.case_narrative, "Updated narrative");

	NarrativeInformationBmc::delete_by_case(&ctx, &mm, case_id).await?;
	CaseBmc::delete(&ctx, &mm, case_id).await?;
	Ok(())
}
