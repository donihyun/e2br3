mod common;

use common::{init_test_mm, Result};
use lib_core::ctx::Ctx;
use lib_core::model::organization::{
	OrganizationBmc, OrganizationForCreate, OrganizationForUpdate,
};
use serial_test::serial;

#[serial]
#[tokio::test]
async fn test_organization_crud() -> Result<()> {
	let mm = init_test_mm().await;
	let ctx = Ctx::root_ctx();
	let org_c = OrganizationForCreate {
		name: "Test Org".to_string(),
		org_type: Some("internal".to_string()),
		address: Some("123 Test St".to_string()),
		contact_email: Some("test-org@example.com".to_string()),
	};

	let org_id = OrganizationBmc::create(&ctx, &mm, org_c).await?;
	let org = OrganizationBmc::get(&ctx, &mm, org_id).await?;
	assert_eq!(org.name, "Test Org");

	let org_u = OrganizationForUpdate {
		name: Some("Updated Org".to_string()),
		org_type: None,
		address: None,
		city: None,
		state: None,
		postcode: None,
		country_code: None,
		contact_email: None,
		contact_phone: None,
		active: Some(false),
	};
	OrganizationBmc::update(&ctx, &mm, org_id, org_u).await?;
	let org = OrganizationBmc::get(&ctx, &mm, org_id).await?;
	assert_eq!(org.name, "Updated Org");
	assert!(!org.active);

	OrganizationBmc::delete(&ctx, &mm, org_id).await?;
	Ok(())
}
