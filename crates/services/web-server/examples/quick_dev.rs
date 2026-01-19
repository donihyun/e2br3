#![allow(unused)] // For example code.

//! Quick development testing script for the E2B(R3) REST API.
//!
//! This script:
//! 1. Initializes the dev database (runs all SQL files in sql/dev_initial/)
//! 2. Sets up the demo user password
//! 3. Demonstrates and tests all major REST API endpoints
//!
//! ## Prerequisites
//! 1. PostgreSQL running locally (postgres://postgres:welcome@localhost/postgres)
//! 2. Start the web server in a separate terminal:
//!    ```bash
//!    cargo run --bin web-server
//!    ```
//!
//! ## Usage
//! ```bash
//! cargo run --example quick_dev
//! ```
//!
//! ## Demo User Credentials
//! - Email: demo.user@example.com
//! - Password: welcome
//! - Role: admin
//!
//! ## API Endpoints Demonstrated
//!
//! ### Authentication
//! - POST /auth/v1/login - Login with email/password
//! - POST /auth/v1/logoff - Logout
//! - POST /auth/v1/refresh - Refresh authentication token
//!
//! ### Organizations (CRUD)
//! - GET    /api/organizations - List all organizations
//! - POST   /api/organizations - Create organization
//! - GET    /api/organizations/{id} - Get organization
//! - PUT    /api/organizations/{id} - Update organization
//! - DELETE /api/organizations/{id} - Delete organization
//!
//! ### Users (CRUD)
//! - GET    /api/users - List all users
//! - POST   /api/users - Create user
//! - GET    /api/users/{id} - Get user
//! - PUT    /api/users/{id} - Update user
//! - DELETE /api/users/{id} - Delete user
//!
//! ### Cases (CRUD with nested resources)
//! - GET    /api/cases - List all cases
//! - POST   /api/cases - Create case
//! - GET    /api/cases/{id} - Get case
//! - PUT    /api/cases/{id} - Update case
//! - DELETE /api/cases/{id} - Delete case
//!
//! ### Case Sub-resources (per case)
//! - /api/cases/{case_id}/patient - Patient info (singleton)
//! - /api/cases/{case_id}/reactions - Reactions (collection)
//! - /api/cases/{case_id}/drugs - Drug info (collection)
//! - /api/cases/{case_id}/test-results - Test results (collection)
//! - /api/cases/{case_id}/narrative - Narrative (singleton)
//! - /api/cases/{case_id}/message-header - Message header (singleton)
//! - /api/cases/{case_id}/safety-report - Safety report ID (singleton)
//! - /api/cases/{case_id}/receiver - Receiver info (singleton)
//! - /api/cases/{case_id}/other-identifiers - Other IDs (collection)
//! - /api/cases/{case_id}/linked-reports - Linked reports (collection)
//! - /api/cases/{case_id}/versions - Case versions (read-only, admin)
//!
//! ### Audit Logs (Admin Only)
//! - GET /api/audit-logs - List all audit logs
//! - GET /api/audit-logs/by-record/{table}/{id} - Logs for specific record
//!
//! ### Terminology (Read-only)
//! - GET /api/terminology/countries - List countries
//! - GET /api/terminology/meddra?q={query} - Search MedDRA
//! - GET /api/terminology/whodrug?q={query} - Search WHODrug
//! - GET /api/terminology/code-lists?list_name={name} - Get code list

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For examples.

use lib_core::_dev_utils;
use lib_core::ctx::Ctx;
use lib_core::model::user::UserBmc;
use lib_core::model::ModelManager;
use serde_json::{json, Value};
use uuid::Uuid;

const BASE_URL: &str = "http://localhost:8080";

// Demo user credentials (from seed data)
const DEMO_USER_ID: &str = "11111111-1111-1111-1111-111111111111";
const DEMO_EMAIL: &str = "demo.user@example.com";

const DEMO_PWD: &str = "welcome";

// Demo organization ID from seed data
const DEMO_ORG_ID: &str = "00000000-0000-0000-0000-000000000001";

#[tokio::main]
async fn main() -> Result<()> {
	// =====================================================================
	// INITIALIZE DEV DATABASE
	// =====================================================================

	println!(">> Setting up demo user password...");
	let mm = ModelManager::new().await?;
	let root_ctx = Ctx::root_ctx();
	let user_id = Uuid::parse_str(DEMO_USER_ID)?;
	UserBmc::update_pwd(&root_ctx, &mm, user_id, DEMO_PWD).await?;
	println!(">> Demo user password set successfully!");

	// =====================================================================
	// START API TESTING
	// =====================================================================
	let hc = httpc_test::new_client(BASE_URL)?;

	println!("\n========== E2B(R3) REST API Quick Dev Testing ==========\n");
	println!("NOTE: Make sure the web server is running: cargo run --bin web-server\n");

	// =====================================================================
	// AUTHENTICATION
	// =====================================================================
	println!("--- AUTHENTICATION ---\n");

	// -- Login
	println!(">> Logging in as demo user...");
	let req_login = hc.do_post(
		"/auth/v1/login",
		json!({
			"email": DEMO_EMAIL,
			"pwd": DEMO_PWD
		}),
	);
	req_login.await?.print().await?;

	// =====================================================================
	// ORGANIZATIONS
	// =====================================================================
	println!("\n--- ORGANIZATIONS ---\n");

	// -- List Organizations
	println!(">> Listing all organizations...");
	hc.do_get("/api/organizations").await?.print().await?;

	// -- Create Organization
	println!(">> Creating a new organization...");
	let create_org_res = hc
		.do_post(
			"/api/organizations",
			json!({
				"data": {
					"name": "Test Pharma Corp",
					"type": "external",
					"address": "456 Test Ave",
					"contact_email": "test@pharma.com"
				}
			}),
		)
		.await?;
	create_org_res.print().await?;
	let new_org_id = create_org_res
		.json_value::<String>("/data/id")
		.unwrap_or_default();

	// -- Get Organization
	if !new_org_id.is_empty() {
		println!(">> Getting organization {}...", new_org_id);
		hc.do_get(&format!("/api/organizations/{}", new_org_id))
			.await?
			.print()
			.await?;

		// -- Update Organization
		println!(">> Updating organization...");
		hc.do_put(
			&format!("/api/organizations/{}", new_org_id),
			json!({
				"data": {
					"name": "Test Pharma Corp Updated",
					"city": "San Francisco",
					"state": "CA"
				}
			}),
		)
		.await?
		.print()
		.await?;
	}

	// =====================================================================
	// USERS
	// =====================================================================
	println!("\n--- USERS ---\n");

	// -- List Users
	println!(">> Listing all users...");
	hc.do_get("/api/users").await?.print().await?;

	// -- Create User
	println!(">> Creating a new user...");
	let create_user_res = hc
		.do_post(
			"/api/users",
			json!({
				"data": {
					"organization_id": DEMO_ORG_ID,
					"email": "new.user@example.com",
					"username": "new_user",
					"pwd_clear": "password123",
					"role": "user",
					"first_name": "New",
					"last_name": "User"
				}
			}),
		)
		.await?;
	create_user_res.print().await?;
	let new_user_id = create_user_res
		.json_value::<String>("/data/id")
		.unwrap_or_default();

	// =====================================================================
	// CASES
	// =====================================================================
	println!("\n--- CASES ---\n");

	// -- List Cases
	println!(">> Listing all cases...");
	hc.do_get("/api/cases").await?.print().await?;

	// -- Create Case
	println!(">> Creating a new case...");
	let create_case_res = hc
		.do_post(
			"/api/cases",
			json!({
				"data": {
					"organization_id": DEMO_ORG_ID,
					"safety_report_id": "SR-TEST-001",
					"status": "draft"
				}
			}),
		)
		.await?;
	create_case_res.print().await?;
	let case_id = create_case_res
		.json_value::<String>("/data/id")
		.unwrap_or_default();

	if !case_id.is_empty() {
		// -- Get Case
		println!(">> Getting case {}...", case_id);
		hc.do_get(&format!("/api/cases/{}", case_id))
			.await?
			.print()
			.await?;

		// -- Update Case
		println!(">> Updating case status...");
		hc.do_put(
			&format!("/api/cases/{}", case_id),
			json!({
				"data": {
					"status": "validated"
				}
			}),
		)
		.await?
		.print()
		.await?;

		// =====================================================================
		// PATIENT (Singleton per Case)
		// =====================================================================
		println!("\n--- PATIENT ---\n");

		// -- Create Patient
		println!(">> Creating patient for case...");
		let create_patient_res = hc
			.do_post(
				&format!("/api/cases/{}/patient", case_id),
				json!({
					"data": {
						"case_id": case_id,
						"patient_initials": "JD",
						"sex": "1"
					}
				}),
			)
			.await?;
		create_patient_res.print().await?;

		// -- Get Patient
		println!(">> Getting patient for case...");
		hc.do_get(&format!("/api/cases/{}/patient", case_id))
			.await?
			.print()
			.await?;

		// -- Update Patient
		println!(">> Updating patient information...");
		hc.do_put(
			&format!("/api/cases/{}/patient", case_id),
			json!({
				"data": {
					"patient_given_name": "John",
					"patient_family_name": "Doe",
					"sex": "1",
					"weight_kg": 75.5,
					"height_cm": 180.0
				}
			}),
		)
		.await?
		.print()
		.await?;

		// =====================================================================
		// REACTIONS (Collection per Case)
		// =====================================================================
		println!("\n--- REACTIONS ---\n");

		// -- Create Reaction
		println!(">> Creating a reaction...");
		let create_reaction_res = hc
			.do_post(
				&format!("/api/cases/{}/reactions", case_id),
				json!({
					"data": {
						"case_id": case_id,
						"sequence_number": 1,
						"primary_source_reaction": "Severe headache"
					}
				}),
			)
			.await?;
		create_reaction_res.print().await?;
		let reaction_id = create_reaction_res
			.json_value::<String>("/data/id")
			.unwrap_or_default();

		// -- List Reactions
		println!(">> Listing reactions for case...");
		hc.do_get(&format!("/api/cases/{}/reactions", case_id))
			.await?
			.print()
			.await?;

		if !reaction_id.is_empty() {
			// -- Get Reaction
			println!(">> Getting reaction {}...", reaction_id);
			hc.do_get(&format!("/api/cases/{}/reactions/{}", case_id, reaction_id))
				.await?
				.print()
				.await?;

			// -- Update Reaction
			println!(">> Updating reaction...");
			hc.do_put(
				&format!("/api/cases/{}/reactions/{}", case_id, reaction_id),
				json!({
					"data": {
						"serious": true,
						"criteria_hospitalization": true,
						"outcome": "1"
					}
				}),
			)
			.await?
			.print()
			.await?;
		}

		// =====================================================================
		// DRUGS (Collection per Case)
		// =====================================================================
		println!("\n--- DRUGS ---\n");

		// -- Create Drug
		println!(">> Creating a drug...");
		let create_drug_res = hc
			.do_post(
				&format!("/api/cases/{}/drugs", case_id),
				json!({
					"data": {
						"case_id": case_id,
						"sequence_number": 1,
						"drug_characterization": "1",
						"medicinal_product": "Aspirin"
					}
				}),
			)
			.await?;
		create_drug_res.print().await?;
		let drug_id = create_drug_res
			.json_value::<String>("/data/id")
			.unwrap_or_default();

		// -- List Drugs
		println!(">> Listing drugs for case...");
		hc.do_get(&format!("/api/cases/{}/drugs", case_id))
			.await?
			.print()
			.await?;

		if !drug_id.is_empty() {
			// -- Get Drug
			println!(">> Getting drug {}...", drug_id);
			hc.do_get(&format!("/api/cases/{}/drugs/{}", case_id, drug_id))
				.await?
				.print()
				.await?;

			// -- Update Drug
			println!(">> Updating drug...");
			hc.do_put(
				&format!("/api/cases/{}/drugs/{}", case_id, drug_id),
				json!({
					"data": {
						"action_taken": "1",
						"additional_info": "Patient stopped taking after reaction"
					}
				}),
			)
			.await?
			.print()
			.await?;
		}

		// =====================================================================
		// TEST RESULTS (Collection per Case)
		// =====================================================================
		println!("\n--- TEST RESULTS ---\n");

		// -- Create Test Result
		println!(">> Creating a test result...");
		let create_test_res = hc
			.do_post(
				&format!("/api/cases/{}/test-results", case_id),
				json!({
					"data": {
						"case_id": case_id,
						"sequence_number": 1,
						"test_name": "Complete Blood Count",
						"test_date": "2024-01-15"
					}
				}),
			)
			.await?;
		create_test_res.print().await?;
		let test_id = create_test_res
			.json_value::<String>("/data/id")
			.unwrap_or_default();

		// -- List Test Results
		println!(">> Listing test results for case...");
		hc.do_get(&format!("/api/cases/{}/test-results", case_id))
			.await?
			.print()
			.await?;

		// =====================================================================
		// NARRATIVE (Singleton per Case)
		// =====================================================================
		println!("\n--- NARRATIVE ---\n");

		// -- Create Narrative
		println!(">> Creating narrative for case...");
		let create_narrative_res = hc
			.do_post(
				&format!("/api/cases/{}/narrative", case_id),
				json!({
					"data": {
						"case_id": case_id,
						"case_narrative": "Patient experienced severe headache after taking Aspirin.",
						"reporter_comments": "First occurrence of this reaction.",
						"sender_comments": "Causality assessment pending."
					}
				}),
			)
			.await?;
		create_narrative_res.print().await?;

		// -- Get Narrative
		println!(">> Getting narrative for case...");
		hc.do_get(&format!("/api/cases/{}/narrative", case_id))
			.await?
			.print()
			.await?;

		// =====================================================================
		// MESSAGE HEADER (Singleton per Case)
		// =====================================================================
		println!("\n--- MESSAGE HEADER ---\n");

		// -- Create Message Header
		println!(">> Creating message header for case...");
		let create_header_res = hc
			.do_post(
				&format!("/api/cases/{}/message-header", case_id),
				json!({
					"data": {
						"case_id": case_id,
						"batch_number": "B-TEST-001",
						"batch_sender_identifier": "TEST-SENDER",
						"message_type": "ichicsr",
						"message_format_version": "2.1",
						"message_number": "MSG-TEST-001",
						"message_sender_identifier": "SENDER-ORG",
						"message_receiver_identifier": "RECEIVER-ORG",
						"message_date": "20240115120000"
					}
				}),
			)
			.await?;
		create_header_res.print().await?;

		// -- Get Message Header
		println!(">> Getting message header for case...");
		hc.do_get(&format!("/api/cases/{}/message-header", case_id))
			.await?
			.print()
			.await?;

		// =====================================================================
		// SAFETY REPORT (Singleton per Case)
		// =====================================================================
		println!("\n--- SAFETY REPORT ---\n");

		// -- Create Safety Report
		println!(">> Creating safety report identification for case...");
		let create_safety_res = hc
			.do_post(
				&format!("/api/cases/{}/safety-report", case_id),
				json!({
					"data": {
						"case_id": case_id,
						"transmission_date": [2024, 15],
						"report_type": "1",
						"date_first_received_from_source": [2024, 10],
						"date_of_most_recent_information": [2024, 15],
						"fulfil_expedited_criteria": true
					}
				}),
			)
			.await?;
		create_safety_res.print().await?;

		// -- Get Safety Report
		println!(">> Getting safety report for case...");
		hc.do_get(&format!("/api/cases/{}/safety-report", case_id))
			.await?
			.print()
			.await?;
	}

	// =====================================================================
	// AUDIT LOGS (Admin Only)
	// =====================================================================
	println!("\n--- AUDIT LOGS (Admin Only) ---\n");

	// -- List Audit Logs (requires admin role)
	println!(">> Listing audit logs (admin only)...");
	hc.do_get("/api/audit-logs").await?.print().await?;

	// -- List Audit Logs by Record (if case was created)
	if !case_id.is_empty() {
		println!(">> Listing audit logs for cases table...");
		hc.do_get(&format!("/api/audit-logs/by-record/cases/{}", case_id))
			.await?
			.print()
			.await?;

		// -- List Case Versions
		println!(">> Listing case versions (admin only)...");
		hc.do_get(&format!("/api/cases/{}/versions", case_id))
			.await?
			.print()
			.await?;
	}

	// =====================================================================
	// TERMINOLOGY (Read-only)
	// =====================================================================
	println!("\n--- TERMINOLOGY ---\n");

	// -- List Countries
	println!(">> Listing countries...");
	hc.do_get("/api/terminology/countries")
		.await?
		.print()
		.await?;

	// -- Search MedDRA (if available)
	println!(">> Searching MedDRA for 'headache'...");
	hc.do_get("/api/terminology/meddra?q=headache")
		.await?
		.print()
		.await?;

	// -- Get Code List
	println!(">> Getting code list for 'sex'...");
	hc.do_get("/api/terminology/code-lists?list_name=sex")
		.await?
		.print()
		.await?;

	// =====================================================================
	// TOKEN REFRESH
	// =====================================================================
	println!("\n--- TOKEN REFRESH ---\n");

	// -- Refresh Token
	println!(">> Refreshing authentication token...");
	hc.do_post("/auth/v1/refresh", json!({})).await?.print().await?;

	// =====================================================================
	// CLEANUP (Optional - uncomment to delete test data)
	// =====================================================================
	println!("\n--- CLEANUP ---\n");

	if !case_id.is_empty() {
		println!(">> Deleting test case...");
		hc.do_delete(&format!("/api/cases/{}", case_id))
			.await?
			.print()
			.await?;
	}

	if !new_org_id.is_empty() {
		println!(">> Deleting test organization...");
		hc.do_delete(&format!("/api/organizations/{}", new_org_id))
			.await?
			.print()
			.await?;
	}

	if !new_user_id.is_empty() {
		println!(">> Deleting test user...");
		hc.do_delete(&format!("/api/users/{}", new_user_id))
			.await?
			.print()
			.await?;
	}

	// =====================================================================
	// LOGOFF
	// =====================================================================
	println!("\n--- LOGOFF ---\n");

	// -- Logoff
	println!(">> Logging off...");
	let req_logoff = hc.do_post(
		"/auth/v1/logoff",
		json!({
			"logoff": true
		}),
	);
	req_logoff.await?.print().await?;

	println!("\n========== Quick Dev Testing Complete ==========\n");

	Ok(())
}
