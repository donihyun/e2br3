use lib_core::model::acs::{
	RECEIVER_CREATE, RECEIVER_DELETE, RECEIVER_READ, RECEIVER_UPDATE,
};
use lib_core::model::receiver::{
	ReceiverInformationBmc, ReceiverInformationForCreate,
	ReceiverInformationForUpdate,
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
	Suffix: receiver,
	PermCreate: RECEIVER_CREATE,
	PermRead: RECEIVER_READ,
	PermUpdate: RECEIVER_UPDATE,
	PermDelete: RECEIVER_DELETE
}
