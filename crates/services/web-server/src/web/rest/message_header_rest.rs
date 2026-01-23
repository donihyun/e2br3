use lib_core::model::message_header::{
	MessageHeaderBmc, MessageHeaderForCreate, MessageHeaderForUpdate,
};
use lib_rest_core::prelude::*;

// Case-scoped single message header CRUD:
// - create_message_header
// - get_message_header
// - update_message_header
// - delete_message_header
generate_case_single_rest_fns! {
	Bmc: MessageHeaderBmc,
	Entity: lib_core::model::message_header::MessageHeader,
	ForCreate: MessageHeaderForCreate,
	ForUpdate: MessageHeaderForUpdate,
	Suffix: message_header
}
