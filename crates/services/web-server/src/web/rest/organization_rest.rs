use lib_core::model::organization::{
	OrganizationBmc, OrganizationFilter, OrganizationForCreate,
	OrganizationForUpdate,
};
use lib_rest_core::prelude::*;

// This macro generates all 5 CRUD functions:
// - create_organization
// - get_organization
// - list_organizations
// - update_organization
// - delete_organization
generate_common_rest_fns! {
	Bmc: OrganizationBmc,
	Entity: lib_core::model::organization::Organization,
	ForCreate: OrganizationForCreate,
	ForUpdate: OrganizationForUpdate,
	Filter: OrganizationFilter,
	Suffix: organization
}
