#![allow(unused)] // For example code.

//! Minimal XML pipeline dev script:
//! login -> create case -> validate -> set validated -> safety report -> export

use serde_json::json;
use std::fs;
use std::path::Path;
use uuid::Uuid;

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

const BASE_URL: &str = "http://localhost:8080";

// Demo user credentials (from seed data)
const DEMO_USER_ID: &str = "11111111-1111-1111-1111-111111111111";
const DEMO_EMAIL: &str = "demo.user@example.com";
const DEMO_PWD: &str = "welcome";
const DEMO_ORG_ID: &str = "00000000-0000-0000-0000-000000000001";

#[tokio::main]
async fn main() -> Result<()> {
	// Ensure demo user password is set for login
	println!(">> Setting up demo user password...");
	let mm = lib_core::model::ModelManager::new().await?;
	let root_ctx = lib_core::ctx::Ctx::root_ctx();
	let user_id = Uuid::parse_str(DEMO_USER_ID)?;
	lib_core::model::user::UserBmc::update_pwd(&root_ctx, &mm, user_id, DEMO_PWD)
		.await?;
	println!(">> Demo user password set successfully!");

	let hc = httpc_test::new_client(BASE_URL)?;

	println!("\n========== E2B(R3) XML Generate Dev ==========" );

	// --- AUTH ---
	println!("\n--- AUTH ---\n");
	let login_res = hc
		.do_post(
			"/auth/v1/login",
			json!({
				"email": DEMO_EMAIL,
				"pwd": DEMO_PWD
			}),
		)
		.await?;
	login_res.print().await?;

	// --- XML VALIDATION (optional) ---
	println!("\n--- XML VALIDATION ---\n");
	if let Ok(examples_dir) = std::env::var("E2BR3_EXAMPLES_DIR") {
		let xml_path = Path::new(&examples_dir)
			.join("1-1_ExampleCase_literature_initial_v1_0.xml");
		let xml = fs::read_to_string(&xml_path)?;
		let boundary = "X-BOUNDARY-XML-VALIDATE";
		let body = format!(
			"--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"case.xml\"\r\nContent-Type: application/xml\r\n\r\n{xml}\r\n--{boundary}--\r\n"
		);
		let res = hc
			.reqwest_client()
			.post(format!("{BASE_URL}/api/import/xml/validate"))
			.header("content-type", format!("multipart/form-data; boundary={boundary}"))
			.body(body)
			.send()
			.await?;
		println!(">> XML validate status: {}", res.status());
		println!(">> XML validate body: {}", res.text().await?);
	} else {
		println!(">> E2BR3_EXAMPLES_DIR not set; skipping XML validation step.");
	}

	// --- CASE CREATE ---
	println!("\n--- CASE ---\n");
	let create_case_res = hc
		.do_post(
			"/api/cases",
			json!({
				"data": {
					"organization_id": DEMO_ORG_ID,
					"safety_report_id": format!("SR-TEST-{}", Uuid::new_v4()),
					"status": "draft"
				}
			}),
		)
		.await?;
	create_case_res.print().await?;
	let case_id = create_case_res
		.json_value::<String>("/data/id")
		.unwrap_or_default();
	if case_id.is_empty() {
		return Err("case_id not returned".into());
	}

	// --- MESSAGE HEADER ---
	println!("\n--- MESSAGE HEADER ---\n");
	hc.do_post(
		&format!("/api/cases/{case_id}/message-header"),
		json!({
			"data": {
				"case_id": case_id,
				"message_number": format!("MSG-{case_id}"),
				"message_sender_identifier": "DSJP",
				"message_receiver_identifier": "ICHTEST",
				"message_date": "20240101120000"
			}
		}),
	)
	.await?
	.print()
	.await?;
	hc.do_put(
		&format!("/api/cases/{case_id}/message-header"),
		json!({
			"data": {
				"batch_number": format!("BATCH-{case_id}"),
				"batch_sender_identifier": "DSJP",
				"batch_receiver_identifier": "ICHTEST"
			}
		}),
	)
	.await?
	.print()
	.await?;

	// --- CASE VALIDATE ---
	println!("\n--- CASE VALIDATE ---\n");
	hc.do_put(
		&format!("/api/cases/{case_id}"),
		json!({
			"data": { "status": "validated" }
		}),
	)
	.await?
	.print()
	.await?;

	// --- SAFETY REPORT ---
	println!("\n--- SAFETY REPORT ---\n");
	hc.do_post(
		&format!("/api/cases/{case_id}/safety-report"),
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
	.await?
	.print()
	.await?;

	// --- EXPORT XML ---
	println!("\n--- EXPORT XML ---\n");
	let export_res = hc
		.reqwest_client()
		.get(format!("{BASE_URL}/api/cases/{case_id}/export/xml"))
		.send()
		.await?;
	println!(">> Export status: {}", export_res.status());
	let xml = export_res.text().await?;
	let export_path = "/Users/hyundonghoon/Documents/e2br3_export.xml";
	fs::write(export_path, &xml)?;
	println!(">> Export XML saved to {export_path} ({} bytes)", xml.len());
	println!(">> Export XML (first 500 chars): {}", &xml[..xml.len().min(500)]);

	println!("\n========== XML Generate Dev Complete ==========");

	Ok(())
}
