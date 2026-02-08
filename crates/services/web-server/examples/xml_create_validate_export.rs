#![allow(unused)] // Example convenience script.

//! Create case from scratch -> validate -> export XML.

use httpc_test::Client;
use serde_json::json;
use serde_json::Value;
use std::fs;
use uuid::Uuid;

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

const BASE_URL: &str = "http://localhost:8080";
const DEMO_EMAIL: &str = "demo.user@example.com";
const DEMO_PWD: &str = "welcome";
const DEMO_USER_ID: &str = "11111111-1111-1111-1111-111111111111";

#[tokio::main]
async fn main() -> Result<()> {
	println!(">> Setting up demo user password...");
	let mm = lib_core::model::ModelManager::new().await?;
	let root_ctx = lib_core::ctx::Ctx::root_ctx();
	let user_id = Uuid::parse_str(DEMO_USER_ID)?;
	lib_core::model::user::UserBmc::update_pwd(&root_ctx, &mm, user_id, DEMO_PWD)
		.await?;
	println!(">> Demo user password set successfully!");

	let hc = httpc_test::new_client(BASE_URL)?;
	println!("\n--- AUTH ---\n");
	ensure_auth(&hc).await?;

	let case_id = create_case(&hc).await?;
	println!(">> Created case_id: {case_id}");

	seed_minimum_case_data(&hc, &case_id).await?;

	// Mark case validated (required for export).
	put_json(
		&hc,
		&format!("/api/cases/{case_id}"),
		json!({ "data": { "status": "validated" } }),
	)
	.await?;

	let export = export_case(&hc, &case_id).await?;
	println!(">> Export XML length: {}", export.len());
	save_export(&export, "export_from_scratch").await?;

	Ok(())
}

async fn create_case(hc: &Client) -> Result<String> {
	let safety_report_id = format!("US-TEST-{}", Uuid::new_v4());
	let res = post_json(
		hc,
		"/api/cases",
		json!({
			"data": {
				"status": "draft",
				"organization_id": "00000000-0000-0000-0000-000000000001",
				"safety_report_id": safety_report_id
			}
		}),
	)
	.await?;
	res.ok_or("case create failed".into())
}

async fn seed_minimum_case_data(hc: &Client, case_id: &str) -> Result<()> {
	// Message header
	post_json_required(
		hc,
		&format!("/api/cases/{case_id}/message-header"),
		json!({
			"data": {
				"case_id": case_id,
				"message_date": "20240101120000",
				"message_date_format": "204",
				"message_format_release": "2.0",
				"message_format_version": "2.1",
				"message_number": format!("MSG-{case_id}"),
				"message_receiver_identifier": "CDER",
				"message_sender_identifier": "DSJP",
				"message_type": "ichicsr"
			}
		}),
	)
	.await?;

	// Safety report
	post_json_required(
		hc,
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
	.await?;

	put_json_required(
		hc,
		&format!("/api/cases/{case_id}/safety-report"),
		json!({
			"data": {
				"local_criteria_report_type": "1",
				"combination_product_report_indicator": "false"
			}
		}),
	)
	.await?;

	// Sender information (C.3.x)
	let sender_id = post_json_required(
		hc,
		&format!("/api/cases/{case_id}/safety-report/senders"),
		json!({
			"data": {
				"case_id": case_id,
				"sender_type": "1",
				"organization_name": "Sender Org"
			}
		}),
	)
	.await?;
	if let Some(sender_id) = sender_id {
		put_json_required(
			hc,
			&format!("/api/cases/{case_id}/safety-report/senders/{sender_id}"),
			json!({
				"data": {
					"department": "PV",
					"street_address": "1 Main St",
					"city": "Boston",
					"state": "MA",
					"postcode": "02101",
					"country_code": "US",
					"person_title": "Dr",
					"person_given_name": "Alex",
					"person_middle_name": "Q",
					"person_family_name": "Smith",
					"telephone": "6170000000",
					"fax": "6170000001",
					"email": "sender@example.com"
				}
			}),
		)
		.await?;
	}

	// Patient
	post_json_required(
		hc,
		&format!("/api/cases/{case_id}/patient"),
		json!({
			"data": {
				"case_id": case_id,
				"patient_initials": "PT",
				"sex": "2"
			}
		}),
	)
	.await?;
	// Populate FDA-required patient fields (race + history)
	put_json_required(
		hc,
		&format!("/api/cases/{case_id}/patient"),
		json!({
			"data": {
				"age_group": "5",
				"race_code": "C41260",
				"medical_history_text": "None"
			}
		}),
	)
	.await?;

	// Reaction
	let reaction_id = post_json_required(
		hc,
		&format!("/api/cases/{case_id}/reactions"),
		json!({
			"data": {
				"case_id": case_id,
				"sequence_number": 1,
				"primary_source_reaction": "Headache",
				"serious": false,
				"outcome": "1"
			}
		}),
	)
	.await?;
	let _ = reaction_id;

	// Drug
	post_json_required(
		hc,
		&format!("/api/cases/{case_id}/drugs"),
		json!({
			"data": {
				"case_id": case_id,
				"sequence_number": 1,
				"drug_characterization": "1",
				"medicinal_product": "Drug A"
			}
		}),
	)
	.await?;

	// Test result (optional but good to exercise)
	post_json_required(
		hc,
		&format!("/api/cases/{case_id}/test-results"),
		json!({
			"data": { "case_id": case_id, "sequence_number": 1, "test_name": "Baseline Test" }
		}),
	)
	.await?;

	// Narrative
	let narrative_id = post_json_required(
		hc,
		&format!("/api/cases/{case_id}/narrative"),
		json!({ "data": { "case_id": case_id, "case_narrative": "Case narrative" } }),
	)
	.await?;

	// Narrative summary (H.5.r)
	if let Some(narrative_id) = narrative_id {
		let summary_id = post_json(
			hc,
			&format!("/api/cases/{case_id}/narrative/summaries"),
			json!({
				"data": {
					"case_id": case_id,
					"narrative_id": narrative_id,
					"sequence_number": 1,
					"summary_text": "Case summary for FDA validation"
				}
			}),
		)
		.await?;
		let summary_id = summary_id
			.or(get_first_id(hc, &format!("/api/cases/{case_id}/narrative/summaries")).await?);
		if let Some(summary_id) = summary_id {
			let _ = put_json(
				hc,
				&format!("/api/cases/{case_id}/narrative/summaries/{summary_id}"),
				json!({
					"data": {
						"summary_type": "2",
						"language_code": "en",
						"summary_text": "Case summary for FDA validation"
					}
				}),
			)
			.await?;
		}
	}

	Ok(())
}

async fn export_case(hc: &Client, case_id: &str) -> Result<String> {
	let cookie = auth_cookie_header(hc)?;
	let res = hc
		.reqwest_client()
		.get(format!("{BASE_URL}/api/cases/{case_id}/export/xml"))
		.header("cookie", cookie)
		.send()
		.await?;
	let status = res.status();
	let body = res.text().await?;
	if !status.is_success() {
		return Err(format!("export status {status} body {body}").into());
	}
	Ok(body)
}

async fn save_export(xml: &str, label: &str) -> Result<()> {
	let filename = format!(
		"/Users/hyundonghoon/Documents/e2br3_export_{label}_{}.xml",
		Uuid::new_v4()
	);
	fs::write(&filename, xml)?;
	println!(">> Export XML saved to {filename}");
	Ok(())
}

async fn ensure_auth(hc: &Client) -> Result<()> {
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
	if !login_res.status().is_success() {
		return Err("login failed; check demo user credentials/seed".into());
	}
	if hc.cookie_value("auth-token").is_none() {
		return Err("login succeeded but auth-token cookie not set".into());
	}
	Ok(())
}

fn auth_cookie_header(hc: &Client) -> Result<String> {
	let token = hc
		.cookie_value("auth-token")
		.ok_or("missing auth-token cookie; login required")?;
	Ok(format!("auth-token={token}"))
}

async fn post_json(hc: &Client, path: &str, body: Value) -> Result<Option<String>> {
	let cookie = auth_cookie_header(hc)?;
	let res = hc
		.reqwest_client()
		.post(format!("{BASE_URL}{path}"))
		.header("cookie", cookie)
		.json(&body)
		.send()
		.await?;
	let status = res.status();
	let text = res.text().await?;
	if !status.is_success() {
		println!(">> POST {path} failed: {status} {text}");
		return Ok(None);
	}
	let value: Value = serde_json::from_str(&text)?;
	let id = value
		.get("data")
		.and_then(|v| v.get("id"))
		.and_then(|v| v.as_str())
		.map(|v| v.to_string());
	Ok(id)
}

async fn put_json(hc: &Client, path: &str, body: Value) -> Result<()> {
	let cookie = auth_cookie_header(hc)?;
	let res = hc
		.reqwest_client()
		.put(format!("{BASE_URL}{path}"))
		.header("cookie", cookie)
		.json(&body)
		.send()
		.await?;
	let status = res.status();
	let text = res.text().await?;
	if !status.is_success() {
		println!(">> PUT {path} failed: {status} {text}");
	}
	Ok(())
}

async fn post_json_required(hc: &Client, path: &str, body: Value) -> Result<Option<String>> {
	let id = post_json(hc, path, body).await?;
	if id.is_none() {
		return Err(format!("required POST failed: {path}").into());
	}
	Ok(id)
}

async fn put_json_required(hc: &Client, path: &str, body: Value) -> Result<()> {
	let cookie = auth_cookie_header(hc)?;
	let res = hc
		.reqwest_client()
		.put(format!("{BASE_URL}{path}"))
		.header("cookie", cookie)
		.json(&body)
		.send()
		.await?;
	let status = res.status();
	let text = res.text().await?;
	if !status.is_success() {
		return Err(format!("required PUT failed: {path} {status} {text}").into());
	}
	Ok(())
}

async fn get_first_id(hc: &Client, path: &str) -> Result<Option<String>> {
	let cookie = auth_cookie_header(hc)?;
	let res = hc
		.reqwest_client()
		.get(format!("{BASE_URL}{path}"))
		.header("cookie", cookie)
		.send()
		.await?;
	let status = res.status();
	let text = res.text().await?;
	if !status.is_success() {
		println!(">> GET {path} failed: {status} {text}");
		return Ok(None);
	}
	let value: Value = serde_json::from_str(&text)?;
	let id = value
		.get("data")
		.and_then(|v| v.as_array())
		.and_then(|arr| arr.first())
		.and_then(|v| v.get("id"))
		.and_then(|v| v.as_str())
		.map(|v| v.to_string());
	Ok(id)
}
