use lib_core::model::receiver::{
	ReceiverInformationBmc, ReceiverInformationForCreate, ReceiverInformationForUpdate,
};
use lib_rest_core::prelude::*;

// Case-scoped single receiver CRUD:
// - create_receiver
// - get_receiver
// - update_receiver
// - delete_receiver
generate_case_single_rest_fns! {
	Bmc: ReceiverInformationBmc,
	Entity: lib_core::model::receiver::ReceiverInformation,
	ForCreate: ReceiverInformationForCreate,
	ForUpdate: ReceiverInformationForUpdate,
	Suffix: receiver
}
