#![allow(unused)] // Example convenience script.

//! XML sample pipeline:
//! login -> validate source XML -> import -> validate case -> export -> validate export

use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use httpc_test::Client;

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

const BASE_URL: &str = "http://localhost:8080";

// Demo user credentials (from seed data)
const DEMO_USER_ID: &str = "11111111-1111-1111-1111-111111111111";
const DEMO_EMAIL: &str = "demo.user@example.com";
const DEMO_PWD: &str = "welcome";
const DEMO_ORG_ID: &str = "00000000-0000-0000-0000-000000000001";

const DEFAULT_SAMPLE_FILES: &[&str] = &[
	"FAERS2022Scenario1.xml",
	"FAERS2022Scenario2.xml",
	"FAERS2022Scenario3.xml",
	"FAERS2022Scenario4.xml",
	"FAERS2022Scenario5-1.xml",
	"FAERS2022Scenario5-2.xml",
	"FAERS2022Scenario5-3.xml",
	"FAERS2022Scenario6.xml",
	"FAERS2022Scenario7.xml",
	"FAERS2022Scenario8.xml",
];

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

	let examples_dir = std::env::var("E2BR3_EXAMPLES_DIR")
		.map(PathBuf::from)
		.unwrap_or_else(|_| PathBuf::from("/Users/hyundonghoon/projects/rust/e2br3/e2br3/docs/refs/instances"));
	let sample_paths = pick_samples(&examples_dir);
	if sample_paths.is_empty() {
		return Err(format!(
			"No sample XML files found in {}",
			examples_dir.display()
		)
		.into());
	}

	let batch_receiver = std::env::var("E2BR3_BATCH_RECEIVER").unwrap_or_else(|_| "ZZFDA".into());
	let message_receiver =
		std::env::var("E2BR3_MESSAGE_RECEIVER").unwrap_or_else(|_| "CDER".into());

	let hc = httpc_test::new_client(BASE_URL)?;

	println!("\n========== E2B(R3) XML Sample Export ==========" );

	// --- AUTH ---
	println!("\n--- AUTH ---\n");
	ensure_auth(&hc).await?;

	for sample_path in sample_paths {
		println!(
			"\n--- SAMPLE: {} ---\n",
			sample_path.file_name().unwrap().to_string_lossy()
		);

		let mut xml = fs::read_to_string(&sample_path)?;
		let unique_id = format!("US-TEST-{}", Uuid::new_v4());
		xml = replace_id_extension(&xml, "2.16.840.1.113883.3.989.2.1.3.1", &unique_id);
		xml = replace_id_extension(&xml, "2.16.840.1.113883.3.989.2.1.3.2", &unique_id);

		validate_xml(&hc, &xml, "source.xml").await?;

		let case_id = import_xml(&hc, &xml).await?;
		println!(">> Imported case_id: {case_id}");

		update_message_header(
			&hc,
			&case_id,
			&batch_receiver,
			&message_receiver,
		)
		.await?;

		validate_case(&hc, &case_id).await?;

		let export_xml = export_case(&hc, &case_id).await?;
		let export_path = format!(
			"/Users/hyundonghoon/Documents/e2br3_export_{}_{}.xml",
			sample_path.file_stem().unwrap().to_string_lossy(),
			case_id
		);
		fs::write(&export_path, &export_xml)?;
		println!(
			">> Export XML saved to {export_path} ({} bytes)",
			export_xml.len()
		);

		validate_xml(&hc, &export_xml, "export.xml").await?;
	}

	println!("\n========== XML Sample Export Complete ==========");
	Ok(())
}

fn pick_samples(dir: &Path) -> Vec<PathBuf> {
	DEFAULT_SAMPLE_FILES
		.iter()
		.map(|name| dir.join(name))
		.filter(|path| path.exists())
		.collect()
}

fn replace_id_extension(xml: &str, root: &str, new_ext: &str) -> String {
	let pattern = format!("root=\"{root}\"");
	let mut out = xml.to_string();
	let mut start = 0;
	while let Some(pos) = out[start..].find(&pattern) {
		let abs = start + pos;
		if let Some(ext_pos) = out[..abs].rfind("extension=\"") {
			let value_start = ext_pos + "extension=\"".len();
			if let Some(end_rel) = out[value_start..].find('"') {
				let value_end = value_start + end_rel;
				out.replace_range(value_start..value_end, new_ext);
			}
		}
		start = abs + pattern.len();
	}
	out
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

async fn validate_xml(hc: &httpc_test::Client, xml: &str, filename: &str) -> Result<()> {
	let boundary = "X-BOUNDARY-XML-VALIDATE";
	let body = format!(
		"--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: application/xml\r\n\r\n{xml}\r\n--{boundary}--\r\n"
	);
	let cookie = auth_cookie_header(hc)?;
	let res = hc
		.reqwest_client()
		.post(format!("{BASE_URL}/api/import/xml/validate"))
		.header("content-type", format!("multipart/form-data; boundary={boundary}"))
		.header("cookie", cookie)
		.body(body)
		.send()
		.await?;
	println!(">> XML validate status: {}", res.status());
	println!(">> XML validate body: {}", res.text().await?);
	Ok(())
}

async fn import_xml(hc: &httpc_test::Client, xml: &str) -> Result<String> {
	let boundary = "X-BOUNDARY-XML-IMPORT";
	let body = format!(
		"--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"case.xml\"\r\nContent-Type: application/xml\r\n\r\n{xml}\r\n--{boundary}--\r\n"
	);
	let cookie = auth_cookie_header(hc)?;
	let res = hc
		.reqwest_client()
		.post(format!("{BASE_URL}/api/import/xml"))
		.header("content-type", format!("multipart/form-data; boundary={boundary}"))
		.header("cookie", cookie)
		.body(body)
		.send()
		.await?;
	let status = res.status();
	let body = res.text().await?;
	if !status.is_success() {
		return Err(format!("import status {status} body {body}").into());
	}
	let value: serde_json::Value = serde_json::from_str(&body)?;
	let case_id = value
		.get("data")
		.and_then(|v| v.get("case_id"))
		.and_then(|v| v.as_str())
		.ok_or("missing case_id in import response")?;
	Ok(case_id.to_string())
}

async fn update_message_header(
	hc: &httpc_test::Client,
	case_id: &str,
	batch_receiver: &str,
	message_receiver: &str,
) -> Result<()> {
	let message_number = format!("MSG-{case_id}");
	let body = json!({
		"data": {
			"case_id": case_id,
			"message_number": message_number,
			"message_sender_identifier": "DSJP",
			"message_receiver_identifier": message_receiver,
			"message_date": "20240101120000",
			"batch_number": format!("BATCH-{case_id}"),
			"batch_sender_identifier": "DSJP",
			"batch_receiver_identifier": batch_receiver
		}
	});
	let post_res = hc
		.do_post(&format!("/api/cases/{case_id}/message-header"), body.clone())
		.await?;
	if !post_res.status().is_success() {
		hc.do_put(&format!("/api/cases/{case_id}/message-header"), body)
			.await?
			.print()
			.await?;
	} else {
		post_res.print().await?;
	}
	Ok(())
}

async fn validate_case(hc: &httpc_test::Client, case_id: &str) -> Result<()> {
	// Ensure safety report identification exists for export.
	let safety_body = json!({
		"data": {
			"case_id": case_id,
			"transmission_date": [2024, 15],
			"report_type": "1",
			"date_first_received_from_source": [2024, 10],
			"date_of_most_recent_information": [2024, 15],
			"fulfil_expedited_criteria": true
		}
	});
	let safety_res = hc
		.do_post(&format!("/api/cases/{case_id}/safety-report"), safety_body)
		.await?;
	if !safety_res.status().is_success() {
		safety_res.print().await?;
	}

	hc.do_put(
		&format!("/api/cases/{case_id}"),
		json!({ "data": { "status": "validated" } }),
	)
	.await?
	.print()
	.await?;
	Ok(())
}

async fn export_case(hc: &httpc_test::Client, case_id: &str) -> Result<String> {
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
