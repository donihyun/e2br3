#![allow(unused)] // Example convenience script.

//! XML merge check:
//! import -> export (baseline) -> update reaction outcome -> export (merged)

use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use httpc_test::Client;

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

const BASE_URL: &str = "http://localhost:8080";

// Demo user credentials (from seed data)
const DEMO_EMAIL: &str = "demo.user@example.com";
const DEMO_PWD: &str = "welcome";
const DEMO_USER_ID: &str = "11111111-1111-1111-1111-111111111111";

const DEFAULT_SAMPLE: &str = "FAERS2022Scenario1.xml";

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
	let sample_path = examples_dir.join(DEFAULT_SAMPLE);
	if !sample_path.exists() {
		return Err(format!(
			"Sample not found: {}",
			sample_path.display()
		)
		.into());
	}

	let hc = httpc_test::new_client(BASE_URL)?;

	println!("\n--- AUTH ---\n");
	ensure_auth(&hc).await?;

	let mut xml = fs::read_to_string(&sample_path)?;
	let unique_id = format!("US-TEST-{}", Uuid::new_v4());
	xml = replace_id_extension(&xml, "2.16.840.1.113883.3.989.2.1.3.1", &unique_id);
	xml = replace_id_extension(&xml, "2.16.840.1.113883.3.989.2.1.3.2", &unique_id);
	validate_xml(&hc, &xml, "source.xml").await?;

	let case_id = import_xml(&hc, &xml).await?;
	println!(">> Imported case_id: {case_id}");

	validate_case(&hc, &case_id).await?;

	let export_before = export_case(&hc, &case_id).await?;
	println!(">> Export before updates: {} bytes", export_before.len());
	save_export(&export_before, "export_before").await?;

	update_safety_report(&hc, &case_id).await?;
	let export_after_c = export_case(&hc, &case_id).await?;
	println!(">> Export after C update: {} bytes", export_after_c.len());
	save_export(&export_after_c, "export_after_c").await?;
	if export_before == export_after_c {
		println!(">> WARNING: export unchanged after C update");
	} else {
		println!(">> OK: export changed after C update");
	}

	update_patient_weight(&hc, &case_id).await?;
	let export_after_d = export_case(&hc, &case_id).await?;
	println!(">> Export after D update: {} bytes", export_after_d.len());
	save_export(&export_after_d, "export_after_d").await?;
	if export_after_c == export_after_d {
		println!(">> WARNING: export unchanged after D update");
	} else {
		println!(">> OK: export changed after D update");
	}

	let test_result_id = first_test_result_id(&hc, &case_id).await?;
	println!(">> Test result id: {test_result_id}");
	update_test_result_name(&hc, &case_id, &test_result_id).await?;
	let export_after_f = export_case(&hc, &case_id).await?;
	println!(">> Export after F update: {} bytes", export_after_f.len());
	save_export(&export_after_f, "export_after_f").await?;
	if export_after_d == export_after_f {
		println!(">> WARNING: export unchanged after F update");
	} else {
		println!(">> OK: export changed after F update");
	}

	let drug_id = first_drug_id(&hc, &case_id).await?;
	println!(">> Drug id: {drug_id}");
	update_drug_medicinal_product(&hc, &case_id, &drug_id).await?;
	let export_after_g = export_case(&hc, &case_id).await?;
	println!(">> Export after G update: {} bytes", export_after_g.len());
	save_export(&export_after_g, "export_after_g").await?;
	if export_after_f == export_after_g {
		println!(">> WARNING: export unchanged after G update");
	} else {
		println!(">> OK: export changed after G update");
	}

	update_narrative_sender_comment(&hc, &case_id).await?;
	let export_after_h = export_case(&hc, &case_id).await?;
	println!(">> Export after H update: {} bytes", export_after_h.len());
	save_export(&export_after_h, "export_after_h").await?;
	if export_after_g == export_after_h {
		println!(">> WARNING: export unchanged after H update");
	} else {
		println!(">> OK: export changed after H update");
	}

	let reaction_id = first_reaction_id(&hc, &case_id).await?;
	println!(">> Reaction id: {reaction_id}");

	update_reaction_outcome(&hc, &case_id, &reaction_id, "1").await?;
	let export_after_e = export_case(&hc, &case_id).await?;
	println!(">> Export after E update: {} bytes", export_after_e.len());
	save_export(&export_after_e, "export_after_e").await?;
	if export_after_c == export_after_e {
		println!(">> WARNING: export unchanged after E update");
	} else {
		println!(">> OK: export changed after E update");
	}

	Ok(())
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

async fn validate_case(hc: &httpc_test::Client, case_id: &str) -> Result<()> {
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
	let cookie = auth_cookie_header(hc)?;
	let exists = hc
		.reqwest_client()
		.get(format!("{BASE_URL}/api/cases/{case_id}/safety-report"))
		.header("cookie", cookie)
		.send()
		.await?
		.status()
		.is_success();
	let safety_res = if exists {
		hc.do_put(&format!("/api/cases/{case_id}/safety-report"), safety_body)
			.await?
	} else {
		hc.do_post(&format!("/api/cases/{case_id}/safety-report"), safety_body)
			.await?
	};
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

async fn update_safety_report(hc: &httpc_test::Client, case_id: &str) -> Result<()> {
	let body = json!({
		"data": {
			"case_id": case_id,
			"report_type": "2"
		}
	});
	let res = hc
		.do_put(&format!("/api/cases/{case_id}/safety-report"), body)
		.await?;
	if !res.status().is_success() {
		res.print().await?;
		return Err("update safety report failed".into());
	}
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

async fn update_patient_weight(hc: &httpc_test::Client, case_id: &str) -> Result<()> {
	let body = json!({
		"data": {
			"weight_kg": 71.2
		}
	});
	let res = hc
		.do_put(&format!("/api/cases/{case_id}/patient"), body)
		.await?;
	if res.status().is_success() {
		return Ok(());
	}
	let status = res.status();
	if status.as_u16() == 404 {
		let create_body = json!({
			"data": {
				"case_id": case_id,
				"patient_initials": "PT",
				"sex": "2"
			}
		});
		let create_res = hc
			.do_post(&format!("/api/cases/{case_id}/patient"), create_body)
			.await?;
		if !create_res.status().is_success() {
			create_res.print().await?;
			return Err("create patient failed".into());
		}
		let retry = hc
			.do_put(
				&format!("/api/cases/{case_id}/patient"),
				json!({ "data": { "weight_kg": 71.2 } }),
			)
			.await?;
		if !retry.status().is_success() {
			retry.print().await?;
			return Err("update patient failed".into());
		}
		return Ok(());
	}
	res.print().await?;
	Err("update patient failed".into())
}

async fn first_reaction_id(hc: &httpc_test::Client, case_id: &str) -> Result<String> {
	let res = hc
		.reqwest_client()
		.get(format!("{BASE_URL}/api/cases/{case_id}/reactions"))
		.send()
		.await?;
	let status = res.status();
	let body = res.text().await?;
	if !status.is_success() {
		return Err(format!("list reactions status {status} body {body}").into());
	}
	let value: serde_json::Value = serde_json::from_str(&body)?;
	let id = value
		.get("data")
		.and_then(|v| v.as_array())
		.and_then(|arr| arr.first())
		.and_then(|v| v.get("id"))
		.and_then(|v| v.as_str())
		.ok_or("missing reaction id")?;
	Ok(id.to_string())
}

async fn first_test_result_id(hc: &httpc_test::Client, case_id: &str) -> Result<String> {
	let res = hc
		.reqwest_client()
		.get(format!("{BASE_URL}/api/cases/{case_id}/test-results"))
		.send()
		.await?;
	let status = res.status();
	let body = res.text().await?;
	if !status.is_success() {
		return Err(format!("list test results status {status} body {body}").into());
	}
	let value: serde_json::Value = serde_json::from_str(&body)?;
	if let Some(id) = value
		.get("data")
		.and_then(|v| v.as_array())
		.and_then(|arr| arr.first())
		.and_then(|v| v.get("id"))
		.and_then(|v| v.as_str())
	{
		return Ok(id.to_string());
	}

	let create_body = json!({
		"data": {
			"case_id": case_id,
			"sequence_number": 1,
			"test_name": "Baseline Test"
		}
	});
	let create_res = hc
		.do_post(&format!("/api/cases/{case_id}/test-results"), create_body)
		.await?;
	if !create_res.status().is_success() {
		create_res.print().await?;
		return Err("create test result failed".into());
	}

	let res = hc
		.reqwest_client()
		.get(format!("{BASE_URL}/api/cases/{case_id}/test-results"))
		.send()
		.await?;
	let status = res.status();
	let body = res.text().await?;
	if !status.is_success() {
		return Err(format!("list test results status {status} body {body}").into());
	}
	let value: serde_json::Value = serde_json::from_str(&body)?;
	let id = value
		.get("data")
		.and_then(|v| v.as_array())
		.and_then(|arr| arr.first())
		.and_then(|v| v.get("id"))
		.and_then(|v| v.as_str())
		.ok_or("missing test result id")?;
	Ok(id.to_string())
}

async fn update_test_result_name(
	hc: &httpc_test::Client,
	case_id: &str,
	test_result_id: &str,
) -> Result<()> {
	let body = json!({
		"data": {
			"test_name": "Updated Test Name",
			"test_result_value": "123.4",
			"test_result_unit": "mg/dL",
			"result_unstructured": "Updated Test Result (unstructured)"
		}
	});
	let res = hc
		.do_put(
			&format!("/api/cases/{case_id}/test-results/{test_result_id}"),
			body,
		)
		.await?;
	if !res.status().is_success() {
		res.print().await?;
		return Err("update test result failed".into());
	}
	Ok(())
}

async fn first_drug_id(hc: &httpc_test::Client, case_id: &str) -> Result<String> {
	let res = hc
		.reqwest_client()
		.get(format!("{BASE_URL}/api/cases/{case_id}/drugs"))
		.send()
		.await?;
	let status = res.status();
	let body = res.text().await?;
	if !status.is_success() {
		return Err(format!("list drugs status {status} body {body}").into());
	}
	let value: serde_json::Value = serde_json::from_str(&body)?;
	let id = value
		.get("data")
		.and_then(|v| v.as_array())
		.and_then(|arr| arr.first())
		.and_then(|v| v.get("id"))
		.and_then(|v| v.as_str())
		.ok_or("missing drug id")?;
	Ok(id.to_string())
}

async fn update_drug_medicinal_product(
	hc: &httpc_test::Client,
	case_id: &str,
	drug_id: &str,
) -> Result<()> {
	let body = json!({
		"data": {
			"medicinal_product": "Updated Medicinal Product"
		}
	});
	let res = hc
		.do_put(&format!("/api/cases/{case_id}/drugs/{drug_id}"), body)
		.await?;
	if !res.status().is_success() {
		res.print().await?;
		return Err("update drug failed".into());
	}
	Ok(())
}

async fn update_narrative_sender_comment(
	hc: &httpc_test::Client,
	case_id: &str,
) -> Result<()> {
	let body = json!({
		"data": {
			"sender_comments": "Updated sender comments"
		}
	});
	let res = hc
		.do_put(&format!("/api/cases/{case_id}/narrative"), body)
		.await?;
	if res.status().is_success() {
		return Ok(());
	}
	let status = res.status();
	if status.as_u16() == 404 {
		let create_body = json!({
			"data": {
				"case_id": case_id,
				"case_narrative": "Baseline narrative"
			}
		});
		let create_res = hc
			.do_post(&format!("/api/cases/{case_id}/narrative"), create_body)
			.await?;
		if !create_res.status().is_success() {
			create_res.print().await?;
			return Err("create narrative failed".into());
		}
		let retry = hc
			.do_put(
				&format!("/api/cases/{case_id}/narrative"),
				json!({ "data": { "sender_comments": "Updated sender comments" } }),
			)
			.await?;
		if !retry.status().is_success() {
			retry.print().await?;
			return Err("update narrative failed".into());
		}
		return Ok(());
	}
	res.print().await?;
	Err("update narrative failed".into())
}

async fn update_reaction_outcome(
	hc: &httpc_test::Client,
	case_id: &str,
	reaction_id: &str,
	outcome: &str,
) -> Result<()> {
	let body = json!({
		"data": {
			"outcome": outcome
		}
	});
	let res = hc
		.do_put(&format!("/api/cases/{case_id}/reactions/{reaction_id}"), body)
		.await?;
	if !res.status().is_success() {
		res.print().await?;
		return Err("update reaction failed".into());
	}
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

async fn save_export(xml: &str, label: &str) -> Result<()> {
	let filename = format!(
		"/Users/hyundonghoon/Documents/e2br3_export_{}_{}.xml",
		label,
		Uuid::new_v4()
	);
	fs::write(&filename, xml)?;
	println!(">> Export XML saved to {}", filename);
	Ok(())
}

fn replace_id_extension(xml: &str, root: &str, new_ext: &str) -> String {
	let pattern = format!("root=\"{root}\"");
	let mut out = xml.to_string();
	let mut start = 0;
	while let Some(pos) = out[start..].find(&pattern) {
		let abs = start + pos;
		if let Some(ext_pos) = out[..abs].rfind("extension=\"") {
			let value_start = ext_pos + "extension=\"".len();
			if let Some(end_rel) = out[value_start..].find('\"') {
				let value_end = value_start + end_rel;
				out.replace_range(value_start..value_end, new_ext);
			}
		}
		start = abs + pattern.len();
	}
	out
}
