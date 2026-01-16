use lib_core::model::user::{UserBmc, UserFilter, UserForCreate, UserForUpdate};
use lib_rest_core::prelude::*;

// This macro generates all 5 CRUD functions:
// - create_user
// - get_user
// - list_users
// - update_user
// - delete_user
generate_common_rest_fns! {
	Bmc: UserBmc,
	Entity: lib_core::model::user::User,
	ForCreate: UserForCreate,
	ForUpdate: UserForUpdate,
	Filter: UserFilter,
	Suffix: user
}
