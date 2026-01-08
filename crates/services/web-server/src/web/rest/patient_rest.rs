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
	Suffix: patient
}
