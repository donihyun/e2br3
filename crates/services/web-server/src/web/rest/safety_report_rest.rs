use lib_core::model::acs::{
	SAFETY_REPORT_CREATE, SAFETY_REPORT_DELETE, SAFETY_REPORT_READ,
	SAFETY_REPORT_UPDATE,
};
use lib_core::model::safety_report::{
	SafetyReportIdentificationBmc, SafetyReportIdentificationForCreate,
	SafetyReportIdentificationForUpdate,
};
use lib_rest_core::prelude::*;

// Case-scoped single safety report identification CRUD:
// - create_safety_report_identification
// - get_safety_report_identification
// - update_safety_report_identification
// - delete_safety_report_identification
generate_case_single_rest_fns! {
	Bmc: SafetyReportIdentificationBmc,
	Entity: lib_core::model::safety_report::SafetyReportIdentification,
	ForCreate: SafetyReportIdentificationForCreate,
	ForUpdate: SafetyReportIdentificationForUpdate,
	Suffix: safety_report_identification,
	PermCreate: SAFETY_REPORT_CREATE,
	PermRead: SAFETY_REPORT_READ,
	PermUpdate: SAFETY_REPORT_UPDATE,
	PermDelete: SAFETY_REPORT_DELETE
}
