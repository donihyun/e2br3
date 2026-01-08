use lib_core::model::terminology::{
	E2bCodeListBmc, E2bCodeListFilter, E2bCodeListForCreate, MeddraTermBmc, MeddraTermFilter,
	MeddraTermForCreate, WhodrugProductBmc, WhodrugProductFilter, WhodrugProductForCreate,
};
use lib_rest_core::prelude::*;

// MedDRA terminology CRUD functions
// - create_meddra_term
// - get_meddra_term
// - list_meddra_terms
// - delete_meddra_term
generate_common_rest_fns! {
	Bmc: MeddraTermBmc,
	Entity: lib_core::model::terminology::MeddraTerm,
	ForCreate: MeddraTermForCreate,
	Filter: MeddraTermFilter,
	Suffix: meddra_term,
	Id: i64
}

// WHODrug product CRUD functions
// - create_whodrug_product
// - get_whodrug_product
// - list_whodrug_products
// - delete_whodrug_product
generate_common_rest_fns! {
	Bmc: WhodrugProductBmc,
	Entity: lib_core::model::terminology::WhodrugProduct,
	ForCreate: WhodrugProductForCreate,
	Filter: WhodrugProductFilter,
	Suffix: whodrug_product,
	Id: i64
}

// E2B code list CRUD functions
// - create_e2b_code_list
// - get_e2b_code_list
// - list_e2b_code_lists
// - delete_e2b_code_list
generate_common_rest_fns! {
	Bmc: E2bCodeListBmc,
	Entity: lib_core::model::terminology::E2bCodeList,
	ForCreate: E2bCodeListForCreate,
	Filter: E2bCodeListFilter,
	Suffix: e2b_code_list,
	Id: i32
}
