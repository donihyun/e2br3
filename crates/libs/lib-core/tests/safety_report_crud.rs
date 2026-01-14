mod common;

use common::{
	create_case_fixture, demo_org_id, demo_user_id, init_test_mm,
	set_current_user, Result,
};
use lib_core::ctx::Ctx;
use lib_core::model::case::CaseBmc;
use lib_core::model::safety_report::{
	SafetyReportIdentificationBmc, SafetyReportIdentificationForCreate,
	SafetyReportIdentificationForUpdate,
};
use serial_test::serial;
use sqlx::types::time::{Date};
use time::Month; 

#[serial]
#[tokio::test]
async fn test_safety_report_identification_crud() -> Result<()> {
	let mm = init_test_mm().await;
	let ctx = Ctx::root_ctx();

	set_current_user(&mm, demo_user_id()).await?;
	let case_id = create_case_fixture(&mm, demo_org_id(), demo_user_id()).await?;

	let report_c = SafetyReportIdentificationForCreate {
		case_id,
		transmission_date: Date::from_calendar_date(2024, Month::January, 1)?,
		report_type: "1".to_string(),
		date_first_received_from_source: Date::from_calendar_date(2024, Month::January, 1)?,
		date_of_most_recent_information: Date::from_calendar_date(2024, Month::January, 1)?,
		fulfil_expedited_criteria: true,
	};
	let report_id = SafetyReportIdentificationBmc::create(&ctx, &mm, report_c).await?;
	let report = SafetyReportIdentificationBmc::get_by_case(&ctx, &mm, case_id).await?;
	assert_eq!(report.id, report_id);

	let report_u = SafetyReportIdentificationForUpdate {
		transmission_date: None,
		report_type: Some("2".to_string()),
		worldwide_unique_id: Some("WUID-1".to_string()),
		nullification_reason: None,
		receiver_organization: Some("Receiver".to_string()),
	};
	SafetyReportIdentificationBmc::update_by_case(&ctx, &mm, case_id, report_u)
		.await?;
	let report = SafetyReportIdentificationBmc::get_by_case(&ctx, &mm, case_id).await?;
	assert_eq!(report.report_type, "2");

	SafetyReportIdentificationBmc::delete_by_case(&ctx, &mm, case_id).await?;
	CaseBmc::delete(&ctx, &mm, case_id).await?;
	Ok(())
}
