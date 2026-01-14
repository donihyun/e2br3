mod common;

use common::{
	create_case_fixture, demo_org_id, demo_user_id, init_test_mm,
	set_current_user, Result,
};
use lib_core::ctx::Ctx;
use lib_core::model::case::CaseBmc;
use lib_core::model::drug::{
	DrugInformationBmc, DrugInformationForCreate, DrugInformationForUpdate,
};
use serial_test::serial;

#[serial]
#[tokio::test]
async fn test_drug_information_crud() -> Result<()> {
	let mm = init_test_mm().await;
	let ctx = Ctx::root_ctx();

	set_current_user(&mm, demo_user_id()).await?;
	let case_id = create_case_fixture(&mm, demo_org_id(), demo_user_id()).await?;

	let drug_c = DrugInformationForCreate {
		case_id,
		sequence_number: 1,
		drug_characterization: "1".to_string(),
		medicinal_product: "Demo Drug".to_string(),
	};
	let drug_id = DrugInformationBmc::create(&ctx, &mm, drug_c).await?;
	let drug = DrugInformationBmc::get(&ctx, &mm, drug_id).await?;
	assert_eq!(drug.medicinal_product, "Demo Drug");

	let drug_u = DrugInformationForUpdate {
		medicinal_product: Some("Updated Drug".to_string()),
		drug_characterization: None,
		brand_name: None,
		manufacturer_name: None,
		batch_lot_number: None,
		action_taken: Some("1".to_string()),
	};
	DrugInformationBmc::update(&ctx, &mm, drug_id, drug_u).await?;
	let drug = DrugInformationBmc::get(&ctx, &mm, drug_id).await?;
	assert_eq!(drug.medicinal_product, "Updated Drug");

	DrugInformationBmc::delete(&ctx, &mm, drug_id).await?;
	CaseBmc::delete(&ctx, &mm, case_id).await?;
	Ok(())
}
