use lib_core::model::test_result::{
	TestResultBmc, TestResultForCreate, TestResultForUpdate,
};
use lib_rest_core::prelude::*;

// Case-scoped CRUD functions:
// - create_test_result
// - get_test_result
// - list_test_results
// - update_test_result
// - delete_test_result
generate_case_rest_fns! {
	Bmc: TestResultBmc,
	Entity: lib_core::model::test_result::TestResult,
	ForCreate: TestResultForCreate,
	ForUpdate: TestResultForUpdate,
	Suffix: test_result
}
