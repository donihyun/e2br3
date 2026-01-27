mod common;

use common::{demo_org_id, demo_user_id, init_test_mm, unique_suffix, DEMO_ROLE, Result};
use lib_core::model::organization::{
	OrganizationBmc, OrganizationForCreate, OrganizationForUpdate,
};
use lib_core::model::store::set_full_context_dbx;
use serial_test::serial;

use crate::common::demo_ctx;

#[serial]
#[tokio::test]
async fn test_organization_crud() -> Result<()> {
	let mm = init_test_mm().await;
	set_full_context_dbx(mm.dbx(), demo_user_id(), demo_org_id(), DEMO_ROLE).await?;
	let ctx = demo_ctx();
	let suffix = unique_suffix();
	let org_c = OrganizationForCreate {
		name: format!("Test Org {suffix}"),
		org_type: Some("internal".to_string()),
		address: Some("123 Test St".to_string()),
		contact_email: Some(format!("test-org-{suffix}@example.com")),
	};

	let org_id = OrganizationBmc::create(&ctx, &mm, org_c).await?;
	let org = OrganizationBmc::get(&ctx, &mm, org_id).await?;
	assert_eq!(org.name, format!("Test Org {suffix}"));

	let orgs = OrganizationBmc::list(&ctx, &mm, None, None).await?;
	assert!(orgs.iter().any(|o| o.id == org_id));

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
