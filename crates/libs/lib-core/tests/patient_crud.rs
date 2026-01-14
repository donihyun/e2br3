mod common;

use common::{
	create_case_fixture, demo_org_id, demo_user_id, init_test_mm,
	set_current_user, Result,
};
use lib_core::ctx::Ctx;
use lib_core::model::case::CaseBmc;
use lib_core::model::patient::{
	PatientInformationBmc, PatientInformationForCreate,
	PatientInformationForUpdate,
};
use serial_test::serial;

#[serial]
#[tokio::test]
async fn test_patient_information_crud() -> Result<()> {
	let mm = init_test_mm().await;
	let ctx = Ctx::root_ctx();

	set_current_user(&mm, demo_user_id()).await?;
	let case_id = create_case_fixture(&mm, demo_org_id(), demo_user_id()).await?;

	let patient_c = PatientInformationForCreate {
		case_id,
		patient_initials: Some("TU".to_string()),
		sex: Some("1".to_string()),
	};
	let patient_id = PatientInformationBmc::create(&ctx, &mm, patient_c).await?;
	let patient = PatientInformationBmc::get_by_case(&ctx, &mm, case_id).await?;
	assert_eq!(patient.id, patient_id);

	let patient_by_id = PatientInformationBmc::get(&ctx, &mm, patient_id).await?;
	assert_eq!(patient_by_id.id, patient_id);

	let patients = PatientInformationBmc::list(&ctx, &mm, None, None).await?;
	assert!(patients.iter().any(|p| p.id == patient_id));

	let patient_u = PatientInformationForUpdate {
		patient_initials: Some("UP".to_string()),
		patient_given_name: None,
		patient_family_name: None,
		birth_date: None,
		age_at_time_of_onset: None,
		age_unit: None,
		weight_kg: None,
		height_cm: None,
		sex: None,
		medical_history_text: Some("Updated history".to_string()),
	};
	PatientInformationBmc::update_by_case(&ctx, &mm, case_id, patient_u).await?;
	let patient = PatientInformationBmc::get_by_case(&ctx, &mm, case_id).await?;
	assert_eq!(patient.patient_initials.as_deref(), Some("UP"));

	PatientInformationBmc::delete_by_case(&ctx, &mm, case_id).await?;
	CaseBmc::delete(&ctx, &mm, case_id).await?;
	Ok(())
}
