// Declare handler modules
pub mod case_rest;
pub mod organization_rest;
pub mod patient_rest;
pub mod user_rest;

pub mod reaction_rest;
pub mod drug_rest;
pub mod test_result_rest;
pub mod narrative_rest;
pub mod message_header_rest;
pub mod safety_report_rest;

// TODO: Enable these when BMC methods are implemented
// pub mod audit_rest;
// pub mod terminology_rest;

use axum::routing::get;
use axum::Router;
use lib_core::model::ModelManager;
use lib_web::handlers::handlers_rest::rest_collection_item_routes;

/// Routes for /api/cases and nested subresources
pub fn routes_cases(mm: ModelManager) -> Router {
	rest_collection_item_routes(
		"/cases",
		"/cases/{id}",
		get(case_rest::list_cases).post(case_rest::create_case),
		get(case_rest::get_case)
			.put(case_rest::update_case)
			.delete(case_rest::delete_case),
	)
		.route(
			"/cases/{case_id}/patient",
			get(patient_rest::get_patient)
				.post(patient_rest::create_patient)
				.put(patient_rest::update_patient)
				.delete(patient_rest::delete_patient),
		)
		.route(
			"/cases/{case_id}/reactions",
			get(reaction_rest::list_reactions).post(reaction_rest::create_reaction),
		)
		.route(
			"/cases/{case_id}/reactions/{id}",
			get(reaction_rest::get_reaction)
				.put(reaction_rest::update_reaction)
				.delete(reaction_rest::delete_reaction),
		)
		.route(
			"/cases/{case_id}/drugs",
			get(drug_rest::list_drug_informations).post(drug_rest::create_drug_information),
		)
		.route(
			"/cases/{case_id}/drugs/{id}",
			get(drug_rest::get_drug_information)
				.put(drug_rest::update_drug_information)
				.delete(drug_rest::delete_drug_information),
		)
		.route(
			"/cases/{case_id}/test-results",
			get(test_result_rest::list_test_results)
				.post(test_result_rest::create_test_result),
		)
		.route(
			"/cases/{case_id}/test-results/{id}",
			get(test_result_rest::get_test_result)
				.put(test_result_rest::update_test_result)
				.delete(test_result_rest::delete_test_result),
		)
		.route(
			"/cases/{case_id}/narrative",
			get(narrative_rest::get_narrative_information)
				.post(narrative_rest::create_narrative_information)
				.put(narrative_rest::update_narrative_information)
				.delete(narrative_rest::delete_narrative_information),
		)
		.route(
			"/cases/{case_id}/message-header",
			get(message_header_rest::get_message_header)
				.post(message_header_rest::create_message_header)
				.put(message_header_rest::update_message_header)
				.delete(message_header_rest::delete_message_header),
		)
		.route(
			"/cases/{case_id}/safety-report",
			get(safety_report_rest::get_safety_report_identification)
				.post(safety_report_rest::create_safety_report_identification)
				.put(safety_report_rest::update_safety_report_identification)
				.delete(safety_report_rest::delete_safety_report_identification),
		)
		.with_state(mm)
}

/// Routes for /api/organizations
pub fn routes_organizations(mm: ModelManager) -> Router {
	rest_collection_item_routes(
		"/organizations",
		"/organizations/{id}",
		get(organization_rest::list_organizations)
			.post(organization_rest::create_organization),
		get(organization_rest::get_organization)
			.put(organization_rest::update_organization)
			.delete(organization_rest::delete_organization),
	)
	.with_state(mm)
}

/// Routes for /api/users
pub fn routes_users(mm: ModelManager) -> Router {
	rest_collection_item_routes(
		"/users",
		"/users/{id}",
		get(user_rest::list_users).post(user_rest::create_user),
		get(user_rest::get_user)
			.put(user_rest::update_user)
			.delete(user_rest::delete_user),
	)
	.with_state(mm)
}
