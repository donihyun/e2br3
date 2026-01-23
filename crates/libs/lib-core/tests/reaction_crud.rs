mod common;

use common::{
	create_case_fixture, demo_org_id, demo_user_id, init_test_mm, set_current_user,
	Result,
};
use lib_core::ctx::Ctx;
use lib_core::model::case::CaseBmc;
use lib_core::model::reaction::{ReactionBmc, ReactionForCreate, ReactionForUpdate};
use serial_test::serial;

#[serial]
#[tokio::test]
async fn test_reaction_crud() -> Result<()> {
	let mm = init_test_mm().await;
	let ctx = Ctx::root_ctx();

	set_current_user(&mm, demo_user_id()).await?;
	let case_id = create_case_fixture(&mm, demo_org_id(), demo_user_id()).await?;

	let reaction_c = ReactionForCreate {
		case_id,
		sequence_number: 1,
		primary_source_reaction: "Headache".to_string(),
	};
	let reaction_id = ReactionBmc::create(&ctx, &mm, reaction_c).await?;
	let reaction = ReactionBmc::get(&ctx, &mm, reaction_id).await?;
	assert_eq!(reaction.primary_source_reaction, "Headache");

	let reaction_u = ReactionForUpdate {
		primary_source_reaction: Some("Updated Headache".to_string()),
		reaction_meddra_code: None,
		reaction_meddra_version: None,
		serious: Some(false),
		criteria_death: None,
		criteria_life_threatening: None,
		criteria_hospitalization: None,
		start_date: None,
		end_date: None,
		outcome: None,
	};
	ReactionBmc::update_in_case(&ctx, &mm, case_id, reaction_id, reaction_u).await?;
	let reaction = ReactionBmc::get_in_case(&ctx, &mm, case_id, reaction_id).await?;
	assert_eq!(reaction.primary_source_reaction, "Updated Headache");

	let reactions = ReactionBmc::list_by_case(&ctx, &mm, case_id).await?;
	assert!(reactions.iter().any(|r| r.id == reaction_id));

	ReactionBmc::delete(&ctx, &mm, reaction_id).await?;
	CaseBmc::delete(&ctx, &mm, case_id).await?;
	Ok(())
}
