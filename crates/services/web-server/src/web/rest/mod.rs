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

// Newly enabled modules
pub mod audit_rest;
pub mod terminology_rest;
pub mod receiver_rest;
pub mod drug_reaction_assessment_rest;
pub mod drug_recurrence_rest;
pub mod case_identifiers_rest;
pub mod parent_history_rest;

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
		// Patient (singleton per case)
		.route(
			"/cases/{case_id}/patient",
			get(patient_rest::get_patient)
				.post(patient_rest::create_patient)
				.put(patient_rest::update_patient)
				.delete(patient_rest::delete_patient),
		)
		// Reactions (collection per case)
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
		// Drugs (collection per case)
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
		// Drug-Reaction Assessments (collection per drug) - G.k.9.i
		.route(
			"/cases/{case_id}/drugs/{drug_id}/reaction-assessments",
			get(drug_reaction_assessment_rest::list_drug_reaction_assessments)
				.post(drug_reaction_assessment_rest::create_drug_reaction_assessment),
		)
		.route(
			"/cases/{case_id}/drugs/{drug_id}/reaction-assessments/{id}",
			get(drug_reaction_assessment_rest::get_drug_reaction_assessment)
				.put(drug_reaction_assessment_rest::update_drug_reaction_assessment)
				.delete(drug_reaction_assessment_rest::delete_drug_reaction_assessment),
		)
		// Drug Recurrences (collection per drug) - G.k.8.r
		.route(
			"/cases/{case_id}/drugs/{drug_id}/recurrences",
			get(drug_recurrence_rest::list_drug_recurrences)
				.post(drug_recurrence_rest::create_drug_recurrence),
		)
		.route(
			"/cases/{case_id}/drugs/{drug_id}/recurrences/{id}",
			get(drug_recurrence_rest::get_drug_recurrence)
				.put(drug_recurrence_rest::update_drug_recurrence)
				.delete(drug_recurrence_rest::delete_drug_recurrence),
		)
		// Test Results (collection per case)
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
		// Narrative (singleton per case)
		.route(
			"/cases/{case_id}/narrative",
			get(narrative_rest::get_narrative_information)
				.post(narrative_rest::create_narrative_information)
				.put(narrative_rest::update_narrative_information)
				.delete(narrative_rest::delete_narrative_information),
		)
		// Message Header (singleton per case)
		.route(
			"/cases/{case_id}/message-header",
			get(message_header_rest::get_message_header)
				.post(message_header_rest::create_message_header)
				.put(message_header_rest::update_message_header)
				.delete(message_header_rest::delete_message_header),
		)
		// Safety Report (singleton per case)
		.route(
			"/cases/{case_id}/safety-report",
			get(safety_report_rest::get_safety_report_identification)
				.post(safety_report_rest::create_safety_report_identification)
				.put(safety_report_rest::update_safety_report_identification)
				.delete(safety_report_rest::delete_safety_report_identification),
		)
		// Receiver (singleton per case) - Section A
		.route(
			"/cases/{case_id}/receiver",
			get(receiver_rest::get_receiver)
				.post(receiver_rest::create_receiver)
				.put(receiver_rest::update_receiver)
				.delete(receiver_rest::delete_receiver),
		)
		// Other Case Identifiers (collection per case) - C.1.9.r
		.route(
			"/cases/{case_id}/other-identifiers",
			get(case_identifiers_rest::list_other_case_identifiers)
				.post(case_identifiers_rest::create_other_case_identifier),
		)
		.route(
			"/cases/{case_id}/other-identifiers/{id}",
			get(case_identifiers_rest::get_other_case_identifier)
				.put(case_identifiers_rest::update_other_case_identifier)
				.delete(case_identifiers_rest::delete_other_case_identifier),
		)
		// Linked Report Numbers (collection per case) - C.1.10.r
		.route(
			"/cases/{case_id}/linked-reports",
			get(case_identifiers_rest::list_linked_report_numbers)
				.post(case_identifiers_rest::create_linked_report_number),
		)
		.route(
			"/cases/{case_id}/linked-reports/{id}",
			get(case_identifiers_rest::get_linked_report_number)
				.put(case_identifiers_rest::update_linked_report_number)
				.delete(case_identifiers_rest::delete_linked_report_number),
		)
		// Parent Medical History (collection per parent) - D.10.7.1.r
		.route(
			"/cases/{case_id}/patient/parent/{parent_id}/medical-history",
			get(parent_history_rest::list_parent_medical_history)
				.post(parent_history_rest::create_parent_medical_history),
		)
		.route(
			"/cases/{case_id}/patient/parent/{parent_id}/medical-history/{id}",
			get(parent_history_rest::get_parent_medical_history)
				.put(parent_history_rest::update_parent_medical_history)
				.delete(parent_history_rest::delete_parent_medical_history),
		)
		// Parent Past Drug History (collection per parent) - D.10.8.r
		.route(
			"/cases/{case_id}/patient/parent/{parent_id}/past-drugs",
			get(parent_history_rest::list_parent_past_drug_history)
				.post(parent_history_rest::create_parent_past_drug_history),
		)
		.route(
			"/cases/{case_id}/patient/parent/{parent_id}/past-drugs/{id}",
			get(parent_history_rest::get_parent_past_drug_history)
				.put(parent_history_rest::update_parent_past_drug_history)
				.delete(parent_history_rest::delete_parent_past_drug_history),
		)
		// Case Versions (read-only collection per case)
		.route(
			"/cases/{case_id}/versions",
			get(audit_rest::list_case_versions),
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

/// Routes for /api/terminology
pub fn routes_terminology(mm: ModelManager) -> Router {
	Router::new()
		.route("/terminology/meddra", get(terminology_rest::search_meddra))
		.route("/terminology/whodrug", get(terminology_rest::search_whodrug))
		.route("/terminology/countries", get(terminology_rest::list_countries))
		.route("/terminology/code-lists", get(terminology_rest::get_code_list))
		.with_state(mm)
}

/// Routes for /api/audit-logs
pub fn routes_audit(mm: ModelManager) -> Router {
	Router::new()
		.route("/audit-logs", get(audit_rest::list_audit_logs))
		.route(
			"/audit-logs/by-record/{table_name}/{record_id}",
			get(audit_rest::list_audit_logs_by_record),
		)
		.with_state(mm)
}
