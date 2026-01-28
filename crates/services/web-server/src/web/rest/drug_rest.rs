use lib_core::model::acs::{
	DRUG_CREATE, DRUG_DELETE, DRUG_LIST, DRUG_READ, DRUG_UPDATE,
};
use lib_core::model::drug::{
	DrugInformationBmc, DrugInformationForCreate, DrugInformationForUpdate,
};
use lib_rest_core::prelude::*;

// Case-scoped CRUD functions:
// - create_drug_information
// - get_drug_information
// - list_drug_informations
// - update_drug_information
// - delete_drug_information
generate_case_rest_fns! {
	Bmc: DrugInformationBmc,
	Entity: lib_core::model::drug::DrugInformation,
	ForCreate: DrugInformationForCreate,
	ForUpdate: DrugInformationForUpdate,
	Suffix: drug_information,
	PermCreate: DRUG_CREATE,
	PermRead: DRUG_READ,
	PermUpdate: DRUG_UPDATE,
	PermDelete: DRUG_DELETE,
	PermList: DRUG_LIST
}
