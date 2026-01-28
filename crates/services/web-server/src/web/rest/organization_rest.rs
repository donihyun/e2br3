use lib_core::model::acs::{ORG_CREATE, ORG_DELETE, ORG_LIST, ORG_READ, ORG_UPDATE};
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
	Suffix: organization,
	PermCreate: ORG_CREATE,
	PermRead: ORG_READ,
	PermUpdate: ORG_UPDATE,
	PermDelete: ORG_DELETE,
	PermList: ORG_LIST
}
