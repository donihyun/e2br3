use lib_core::model::acs::{
	TEST_RESULT_CREATE, TEST_RESULT_DELETE, TEST_RESULT_LIST, TEST_RESULT_READ,
	TEST_RESULT_UPDATE,
};
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
	Suffix: test_result,
	PermCreate: TEST_RESULT_CREATE,
	PermRead: TEST_RESULT_READ,
	PermUpdate: TEST_RESULT_UPDATE,
	PermDelete: TEST_RESULT_DELETE,
	PermList: TEST_RESULT_LIST
}
