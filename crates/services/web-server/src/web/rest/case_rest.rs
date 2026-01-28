use lib_core::model::acs::{
	CASE_CREATE, CASE_DELETE, CASE_LIST, CASE_READ, CASE_UPDATE,
};
use lib_core::model::case::{CaseBmc, CaseFilter, CaseForCreate, CaseForUpdate};
use lib_rest_core::prelude::*;

// This macro generates all 5 CRUD functions:
// - create_case
// - get_case
// - list_cases
// - update_case
// - delete_case
generate_common_rest_fns! {
	Bmc: CaseBmc,
	Entity: lib_core::model::case::Case,
	ForCreate: CaseForCreate,
	ForUpdate: CaseForUpdate,
	Filter: CaseFilter,
	Suffix: case,
	PermCreate: CASE_CREATE,
	PermRead: CASE_READ,
	PermUpdate: CASE_UPDATE,
	PermDelete: CASE_DELETE,
	PermList: CASE_LIST
}
