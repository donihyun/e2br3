use lib_core::model::narrative::{
	NarrativeInformationBmc, NarrativeInformationForCreate, NarrativeInformationForUpdate,
};
use lib_rest_core::prelude::*;

// Case-scoped single narrative CRUD:
// - create_narrative_information
// - get_narrative_information
// - update_narrative_information
// - delete_narrative_information
generate_case_single_rest_fns! {
	Bmc: NarrativeInformationBmc,
	Entity: lib_core::model::narrative::NarrativeInformation,
	ForCreate: NarrativeInformationForCreate,
	ForUpdate: NarrativeInformationForUpdate,
	Suffix: narrative_information
}
