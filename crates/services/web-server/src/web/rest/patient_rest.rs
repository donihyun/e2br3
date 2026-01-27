use lib_core::model::acs::{
	PATIENT_CREATE, PATIENT_DELETE, PATIENT_READ, PATIENT_UPDATE,
};
use lib_core::model::patient::{
	PatientInformationBmc, PatientInformationForCreate, PatientInformationForUpdate,
};
use lib_rest_core::prelude::*;

// Case-scoped single patient CRUD:
// - create_patient
// - get_patient
// - update_patient
// - delete_patient
generate_case_single_rest_fns! {
	Bmc: PatientInformationBmc,
	Entity: lib_core::model::patient::PatientInformation,
	ForCreate: PatientInformationForCreate,
	ForUpdate: PatientInformationForUpdate,
	Suffix: patient,
	PermCreate: PATIENT_CREATE,
	PermRead: PATIENT_READ,
	PermUpdate: PATIENT_UPDATE,
	PermDelete: PATIENT_DELETE
}
