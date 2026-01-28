use lib_core::model::acs::{
	REACTION_CREATE, REACTION_DELETE, REACTION_LIST, REACTION_READ, REACTION_UPDATE,
};
use lib_core::model::reaction::{ReactionBmc, ReactionForCreate, ReactionForUpdate};
use lib_rest_core::prelude::*;

// Case-scoped CRUD functions:
// - create_reaction
// - get_reaction
// - list_reactions
// - update_reaction
// - delete_reaction
generate_case_rest_fns! {
	Bmc: ReactionBmc,
	Entity: lib_core::model::reaction::Reaction,
	ForCreate: ReactionForCreate,
	ForUpdate: ReactionForUpdate,
	Suffix: reaction,
	PermCreate: REACTION_CREATE,
	PermRead: REACTION_READ,
	PermUpdate: REACTION_UPDATE,
	PermDelete: REACTION_DELETE,
	PermList: REACTION_LIST
}
