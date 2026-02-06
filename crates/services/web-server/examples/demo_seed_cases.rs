#![allow(unused)] // Example convenience script.

//! Demo seed script: create a batch of example cases.
//!
//! Requires users/orgs to exist (run demo_seed_users_orgs.rs first).

use httpc_test::Client;
use serde_json::json;
use serde_json::Value;
use uuid::Uuid;

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

const BASE_URL: &str = "http://localhost:8080";

const DEMO_EMAIL: &str = "demo.user@example.com";
const DEMO_PWD: &str = "welcome";
const DEMO_USER_ID: &str = "11111111-1111-1111-1111-111111111111";
const DEMO_ORG_ID: &str = "00000000-0000-0000-0000-000000000001";

const DEFAULT_CASES: usize = 6;

#[tokio::main]
async fn main() -> Result<()> {
	println!("== Demo Seed: Cases ==");

	let mm = lib_core::model::ModelManager::new().await?;
	let root_ctx = lib_core::ctx::Ctx::root_ctx();
	let _ = maybe_set_demo_pwd(&mm, &root_ctx).await;

	let hc = httpc_test::new_client(BASE_URL)?;
	ensure_auth(&hc).await?;

	let count = std::env::var("E2BR3_DEMO_CASES")
		.ok()
		.and_then(|v| v.parse::<usize>().ok())
		.unwrap_or(DEFAULT_CASES);

	for i in 0..count {
		let case_id = create_case(&hc, i).await?;
		println!(">> Created case {i}: {case_id}");
		seed_minimum_case_data(&hc, &case_id, i).await?;

		if i == 0 {
			put_json(
				&hc,
				&format!("/api/cases/{case_id}"),
				json!({ "data": { "status": "validated" } }),
			)
			.await?;
		}
	}

	println!("== Case seed complete ==");
	Ok(())
}

async fn maybe_set_demo_pwd(
	mm: &lib_core::model::ModelManager,
	ctx: &lib_core::ctx::Ctx,
) -> Result<()> {
	let user_id = Uuid::parse_str(DEMO_USER_ID)?;
	if lib_core::model::user::UserBmc::get::<lib_core::model::user::User>(
		ctx, mm, user_id,
	)
	.await
	.is_ok()
	{
		lib_core::model::user::UserBmc::update_pwd(ctx, mm, user_id, DEMO_PWD)
			.await?;
	}
	Ok(())
}

async fn create_case(hc: &Client, idx: usize) -> Result<String> {
	let safety_report_id = format!("US-DEMO-{}-{}", idx + 1, Uuid::new_v4());
	let res = post_json(
		hc,
		"/api/cases",
		json!({
			"data": {
				"status": "draft",
				"organization_id": DEMO_ORG_ID,
				"safety_report_id": safety_report_id
			}
		}),
	)
	.await?;
	res.ok_or("case create failed".into())
}

async fn seed_minimum_case_data(
	hc: &Client,
	case_id: &str,
	idx: usize,
) -> Result<()> {
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
	let report_type = if idx % 2 == 0 { "1" } else { "2" };
	post_json_required(
		hc,
		&format!("/api/cases/{case_id}/safety-report"),
		json!({
			"data": {
				"case_id": case_id,
				"transmission_date": [2024, 15],
				"report_type": report_type,
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

	// Sender
	post_json_required(
		hc,
		&format!("/api/cases/{case_id}/safety-report/senders"),
		json!({
			"data": {
				"case_id": case_id,
				"sender_type": "1",
				"organization_name": "Demo Sender Org"
			}
		}),
	)
	.await?;

	// Patient
	post_json_required(
		hc,
		&format!("/api/cases/{case_id}/patient"),
		json!({
			"data": {
				"case_id": case_id,
				"patient_initials": format!("D{idx}"),
				"sex": if idx % 2 == 0 { "1" } else { "2" }
			}
		}),
	)
	.await?;

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
	post_json_required(
		hc,
		&format!("/api/cases/{case_id}/reactions"),
		json!({
			"data": {
				"case_id": case_id,
				"sequence_number": 1,
				"primary_source_reaction": if idx % 2 == 0 { "Headache" } else { "Nausea" },
				"serious": false,
				"outcome": "1"
			}
		}),
	)
	.await?;

	// Drug
	post_json_required(
		hc,
		&format!("/api/cases/{case_id}/drugs"),
		json!({
			"data": {
				"case_id": case_id,
				"sequence_number": 1,
				"drug_characterization": "1",
				"medicinal_product": if idx % 2 == 0 { "Drug A" } else { "Drug B" }
			}
		}),
	)
	.await?;

	// Narrative
	post_json_required(
		hc,
		&format!("/api/cases/{case_id}/narrative"),
		json!({ "data": { "case_id": case_id, "case_narrative": "Demo case narrative" } }),
	)
	.await?;

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
