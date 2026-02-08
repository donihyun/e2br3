#![allow(unused)] // Example convenience script.

//! Modify-all example:
//! import -> then exercise (create/update) all case-modifying endpoints once.

use httpc_test::Client;
use serde_json::json;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

const BASE_URL: &str = "http://localhost:8080";
const DEMO_EMAIL: &str = "demo.user@example.com";
const DEMO_PWD: &str = "welcome";
const DEMO_USER_ID: &str = "11111111-1111-1111-1111-111111111111";
const DEFAULT_SAMPLE: &str = "FAERS2022Scenario1.xml";

#[tokio::main]
async fn main() -> Result<()> {
	println!(">> Setting up demo user password...");
	let mm = lib_core::model::ModelManager::new().await?;
	let root_ctx = lib_core::ctx::Ctx::root_ctx();
	let user_id = Uuid::parse_str(DEMO_USER_ID)?;
	lib_core::model::user::UserBmc::update_pwd(&root_ctx, &mm, user_id, DEMO_PWD)
		.await?;
	println!(">> Demo user password set successfully!");

	let examples_dir = std::env::var("E2BR3_EXAMPLES_DIR")
		.map(PathBuf::from)
		.unwrap_or_else(|_| {
			PathBuf::from("/Users/hyundonghoon/projects/rust/e2br3/e2br3/docs/refs/instances")
		});
	let sample_path = examples_dir.join(DEFAULT_SAMPLE);
	if !sample_path.exists() {
		return Err(format!("Sample not found: {}", sample_path.display()).into());
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

	// Ensure case is validated (required for export)
	validate_case(&hc, &case_id).await?;

	// Exercise all endpoints (best-effort).
	exercise_case_endpoints(&hc, &case_id).await?;

	// Export to ensure no crash.
	let export = export_case(&hc, &case_id).await?;
	println!(">> Export final: {} bytes", export.len());
	save_export(&export, "export_after_modify_all").await?;

	Ok(())
}

async fn exercise_case_endpoints(hc: &Client, case_id: &str) -> Result<()> {
	// Message header
	let _ = post_json(
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
	let _ = put_json(
		hc,
		&format!("/api/cases/{case_id}/message-header"),
		json!({ "data": { "batch_number": "BATCH-1" } }),
	)
	.await?;

	// Safety report (create if missing, otherwise update)
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
	let safety_exists =
		get_singleton_id(hc, &format!("/api/cases/{case_id}/safety-report"))
			.await?
			.is_some();
	if safety_exists {
		let _ = put_json(
			hc,
			&format!("/api/cases/{case_id}/safety-report"),
			safety_body.clone(),
		)
		.await?;
	} else {
		let _ = post_json(
			hc,
			&format!("/api/cases/{case_id}/safety-report"),
			safety_body.clone(),
		)
		.await?;
	}
	let _ = put_json(
		hc,
		&format!("/api/cases/{case_id}/safety-report"),
		json!({ "data": { "report_type": "2" } }),
	)
	.await?;

	// Receiver (Section A)
	let receiver_exists =
		get_singleton_id(hc, &format!("/api/cases/{case_id}/receiver"))
			.await?
			.is_some();
	if receiver_exists {
		let _ = put_json(
			hc,
			&format!("/api/cases/{case_id}/receiver"),
			json!({ "data": { "organization_name": "FDA", "country_code": "US" } }),
		)
		.await?;
	} else {
		let _ = post_json(
			hc,
			&format!("/api/cases/{case_id}/receiver"),
			json!({ "data": { "case_id": case_id, "receiver_type": "1", "organization_name": "FDA" } }),
		)
		.await?;
	}
	let _ = put_json(
		hc,
		&format!("/api/cases/{case_id}/receiver"),
		json!({ "data": { "organization_name": "FDA Center", "country_code": "US" } }),
	)
	.await?;

	// Sender information
	let sender_id = ensure_first_id(
		hc,
		&format!("/api/cases/{case_id}/safety-report/senders"),
		Some(json!({
			"data": { "case_id": case_id, "sender_type": "1", "organization_name": "Sender Org" }
		})),
	)
	.await?;
	if let Some(sender_id) = sender_id {
		let _ = put_json(
			hc,
			&format!("/api/cases/{case_id}/safety-report/senders/{sender_id}"),
			json!({ "data": { "department": "PV" } }),
		)
		.await?;
	}

	// Primary source
	let source_id = ensure_first_id(
		hc,
		&format!("/api/cases/{case_id}/safety-report/primary-sources"),
		Some(json!({
			"data": { "case_id": case_id, "sequence_number": 1, "qualification": "1" }
		})),
	)
	.await?;
	if let Some(source_id) = source_id {
		let _ = put_json(
			hc,
			&format!("/api/cases/{case_id}/safety-report/primary-sources/{source_id}"),
			json!({ "data": { "reporter_given_name": "John", "reporter_family_name": "Doe", "primary_source_regulatory": "1" } }),
		)
		.await?;
	}

	// Literature references
	let lit_id = ensure_first_id(
		hc,
		&format!("/api/cases/{case_id}/safety-report/literature"),
		Some(json!({
			"data": { "case_id": case_id, "reference_text": "Reference text", "sequence_number": 1 }
		})),
	)
	.await?;
	if let Some(lit_id) = lit_id {
		let _ = put_json(
			hc,
			&format!("/api/cases/{case_id}/safety-report/literature/{lit_id}"),
			json!({ "data": { "reference_text": "Reference updated" } }),
		)
		.await?;
	}

	// Study info and registrations
	let study_id = ensure_first_id(
		hc,
		&format!("/api/cases/{case_id}/safety-report/studies"),
		Some(json!({
			"data": { "case_id": case_id, "study_name": "Study", "sponsor_study_number": "STUDY-1" }
		})),
	)
	.await?;
	if let Some(study_id) = study_id {
		let _ = put_json(
			hc,
			&format!("/api/cases/{case_id}/safety-report/studies/{study_id}"),
			json!({ "data": { "study_type_reaction": "1" } }),
		)
		.await?;

		let reg_id = ensure_first_id(
			hc,
			&format!("/api/cases/{case_id}/safety-report/studies/{study_id}/registrations"),
			Some(json!({
				"data": { "study_information_id": study_id, "registration_number": "REG-1", "country_code": "US", "sequence_number": 1 }
			})),
		)
		.await?;
		if let Some(reg_id) = reg_id {
			let _ = put_json(
				hc,
				&format!("/api/cases/{case_id}/safety-report/studies/{study_id}/registrations/{reg_id}"),
				json!({ "data": { "registration_number": "REG-2" } }),
			)
			.await?;
		}
	}

	// Other case identifiers
	let other_id = ensure_first_id(
		hc,
		&format!("/api/cases/{case_id}/other-identifiers"),
		Some(json!({
			"data": { "case_id": case_id, "sequence_number": 1, "source_of_identifier": "SRC", "case_identifier": "CASE-123" }
		})),
	)
	.await?;
	if let Some(other_id) = other_id {
		let _ = put_json(
			hc,
			&format!("/api/cases/{case_id}/other-identifiers/{other_id}"),
			json!({ "data": { "case_identifier": "CASE-456" } }),
		)
		.await?;
	}

	// Linked reports
	let linked_id = ensure_first_id(
		hc,
		&format!("/api/cases/{case_id}/linked-reports"),
		Some(json!({
			"data": { "case_id": case_id, "sequence_number": 1, "linked_report_number": "LINK-1" }
		})),
	)
	.await?;
	if let Some(linked_id) = linked_id {
		let _ = put_json(
			hc,
			&format!("/api/cases/{case_id}/linked-reports/{linked_id}"),
			json!({ "data": { "linked_report_number": "LINK-2" } }),
		)
		.await?;
	}

	// Patient (singleton)
	let patient_exists =
		get_singleton_id(hc, &format!("/api/cases/{case_id}/patient"))
			.await?
			.is_some();
	if patient_exists {
		let _ = put_json(
			hc,
			&format!("/api/cases/{case_id}/patient"),
			json!({ "data": { "patient_initials": "PT", "sex": "2" } }),
		)
		.await?;
	} else {
		let _ = post_json(
			hc,
			&format!("/api/cases/{case_id}/patient"),
			json!({ "data": { "case_id": case_id, "patient_initials": "PT", "sex": "2" } }),
		)
		.await?;
	}
	let _ = put_json(
		hc,
		&format!("/api/cases/{case_id}/patient"),
		json!({ "data": { "weight_kg": 70.5, "race_code": "2106-3" } }),
	)
	.await?;

	// Patient identifiers
	let patient_id = get_singleton_id(hc, &format!("/api/cases/{case_id}/patient")).await?;
	if let Some(patient_id) = patient_id {
		let pid_id = ensure_first_id(
			hc,
			&format!("/api/cases/{case_id}/patient/identifiers"),
			Some(json!({
				"data": { "patient_id": patient_id, "sequence_number": 1, "identifier_type_code": "1", "identifier_value": "MRN-1" }
			})),
		)
		.await?;
		if let Some(pid_id) = pid_id {
			let _ = put_json(
				hc,
				&format!("/api/cases/{case_id}/patient/identifiers/{pid_id}"),
				json!({ "data": { "identifier_value": "MRN-2" } }),
			)
			.await?;
		}

		let mh_id = ensure_first_id(
			hc,
			&format!("/api/cases/{case_id}/patient/medical-history"),
			Some(json!({
				"data": { "patient_id": patient_id, "sequence_number": 1, "meddra_code": "10000001" }
			})),
		)
		.await?;
		if let Some(mh_id) = mh_id {
			let _ = put_json(
				hc,
				&format!("/api/cases/{case_id}/patient/medical-history/{mh_id}"),
				json!({ "data": { "comments": "History updated" } }),
			)
			.await?;
		}

		let past_id = ensure_first_id(
			hc,
			&format!("/api/cases/{case_id}/patient/past-drugs"),
			Some(json!({
				"data": { "patient_id": patient_id, "sequence_number": 1, "drug_name": "Old Drug" }
			})),
		)
		.await?;
		if let Some(past_id) = past_id {
			let _ = put_json(
				hc,
				&format!("/api/cases/{case_id}/patient/past-drugs/{past_id}"),
				json!({ "data": { "drug_name": "Old Drug Updated" } }),
			)
			.await?;
		}

		let death_id = ensure_first_id(
			hc,
			&format!("/api/cases/{case_id}/patient/death-info"),
			Some(json!({
				"data": { "patient_id": patient_id, "autopsy_performed": false }
			})),
		)
		.await?;
		if let Some(death_id) = death_id {
			let _ = put_json(
				hc,
				&format!("/api/cases/{case_id}/patient/death-info/{death_id}"),
				json!({ "data": { "autopsy_performed": true } }),
			)
			.await?;

			let rep_id = ensure_first_id(
				hc,
				&format!("/api/cases/{case_id}/patient/death-info/{death_id}/reported-causes"),
				Some(json!({
					"data": { "death_info_id": death_id, "sequence_number": 1, "meddra_code": "10000002" }
				})),
			)
			.await?;
			if let Some(rep_id) = rep_id {
				let _ = put_json(
					hc,
					&format!("/api/cases/{case_id}/patient/death-info/{death_id}/reported-causes/{rep_id}"),
					json!({ "data": { "meddra_code": "10000003" } }),
				)
				.await?;
			}

			let aut_id = ensure_first_id(
				hc,
				&format!("/api/cases/{case_id}/patient/death-info/{death_id}/autopsy-causes"),
				Some(json!({
					"data": { "death_info_id": death_id, "sequence_number": 1, "meddra_code": "10000004" }
				})),
			)
			.await?;
			if let Some(aut_id) = aut_id {
				let _ = put_json(
					hc,
					&format!("/api/cases/{case_id}/patient/death-info/{death_id}/autopsy-causes/{aut_id}"),
					json!({ "data": { "meddra_code": "10000005" } }),
				)
				.await?;
			}
		}

		let parent_id = ensure_first_id(
			hc,
			&format!("/api/cases/{case_id}/patient/parents"),
			Some(json!({
				"data": { "patient_id": patient_id, "sex": "1", "medical_history_text": "Parent history" }
			})),
		)
		.await?;
		if let Some(parent_id) = parent_id {
			let _ = put_json(
				hc,
				&format!("/api/cases/{case_id}/patient/parents/{parent_id}"),
				json!({ "data": { "parent_identification": "Parent-1" } }),
			)
			.await?;

			let pmh_id = ensure_first_id(
				hc,
				&format!("/api/cases/{case_id}/patient/parent/{parent_id}/medical-history"),
				Some(json!({
					"data": { "parent_id": parent_id, "sequence_number": 1, "meddra_code": "10000006" }
				})),
			)
			.await?;
			if let Some(pmh_id) = pmh_id {
				let _ = put_json(
					hc,
					&format!("/api/cases/{case_id}/patient/parent/{parent_id}/medical-history/{pmh_id}"),
					json!({ "data": { "comments": "Parent history updated" } }),
				)
				.await?;
			}

			let ppd_id = ensure_first_id(
				hc,
				&format!("/api/cases/{case_id}/patient/parent/{parent_id}/past-drugs"),
				Some(json!({
					"data": { "parent_id": parent_id, "sequence_number": 1, "drug_name": "Parent Drug" }
				})),
			)
			.await?;
			if let Some(ppd_id) = ppd_id {
				let _ = put_json(
					hc,
					&format!("/api/cases/{case_id}/patient/parent/{parent_id}/past-drugs/{ppd_id}"),
					json!({ "data": { "drug_name": "Parent Drug Updated" } }),
				)
				.await?;
			}
		}
	}

	// Reactions
	let reaction_id = ensure_first_id(
		hc,
		&format!("/api/cases/{case_id}/reactions"),
		Some(json!({
			"data": { "case_id": case_id, "sequence_number": 1, "primary_source_reaction": "Headache", "serious": false, "outcome": "1" }
		})),
	)
	.await?;
	if let Some(ref reaction_id) = reaction_id {
		let _ = put_json(
			hc,
			&format!("/api/cases/{case_id}/reactions/{reaction_id}"),
			json!({ "data": { "outcome": "2" } }),
		)
		.await?;
	}

	// Test results
	let test_id = ensure_first_id(
		hc,
		&format!("/api/cases/{case_id}/test-results"),
		Some(json!({
			"data": { "case_id": case_id, "sequence_number": 1, "test_name": "Baseline Test" }
		})),
	)
	.await?;
	if let Some(test_id) = test_id {
		let _ = put_json(
			hc,
			&format!("/api/cases/{case_id}/test-results/{test_id}"),
			json!({ "data": { "test_result_value": "111", "test_result_unit": "mg/dL", "result_unstructured": "Test unstructured" } }),
		)
		.await?;
	}

	// Drugs
	let drug_id = ensure_first_id(
		hc,
		&format!("/api/cases/{case_id}/drugs"),
		Some(json!({
			"data": { "case_id": case_id, "sequence_number": 1, "drug_characterization": "1", "medicinal_product": "Drug A" }
		})),
	)
	.await?;
	if let Some(drug_id) = drug_id {
		let _ = put_json(
			hc,
			&format!("/api/cases/{case_id}/drugs/{drug_id}"),
			json!({ "data": { "medicinal_product": "Drug A Updated" } }),
		)
		.await?;

		let active_id = ensure_first_id(
			hc,
			&format!("/api/cases/{case_id}/drugs/{drug_id}/active-substances"),
			Some(json!({
				"data": { "drug_id": drug_id, "sequence_number": 1, "substance_name": "Substance", "strength_value": 10.0, "strength_unit": "mg" }
			})),
		)
		.await?;
		if let Some(active_id) = active_id {
			let _ = put_json(
				hc,
				&format!("/api/cases/{case_id}/drugs/{drug_id}/active-substances/{active_id}"),
				json!({ "data": { "substance_name": "Substance Updated" } }),
			)
			.await?;
		}

		let dosage_id = ensure_first_id(
			hc,
			&format!("/api/cases/{case_id}/drugs/{drug_id}/dosages"),
			Some(json!({
				"data": { "drug_id": drug_id, "sequence_number": 1, "dose_value": 1.0, "dose_unit": "tab", "frequency_value": 1.0, "frequency_unit": "day" }
			})),
		)
		.await?;
		if let Some(dosage_id) = dosage_id {
			let _ = put_json(
				hc,
				&format!("/api/cases/{case_id}/drugs/{drug_id}/dosages/{dosage_id}"),
				json!({ "data": { "dose_value": 2.0 } }),
			)
			.await?;
		}

		let indication_id = ensure_first_id(
			hc,
			&format!("/api/cases/{case_id}/drugs/{drug_id}/indications"),
			Some(json!({
				"data": { "drug_id": drug_id, "sequence_number": 1, "indication_text": "Indication", "indication_meddra_version": "26.0", "indication_meddra_code": "10000007" }
			})),
		)
		.await?;
		if let Some(indication_id) = indication_id {
			let _ = put_json(
				hc,
				&format!("/api/cases/{case_id}/drugs/{drug_id}/indications/{indication_id}"),
				json!({ "data": { "indication_text": "Indication Updated" } }),
			)
			.await?;
		}

		let assess_id = ensure_first_id(
			hc,
			&format!("/api/cases/{case_id}/drugs/{drug_id}/reaction-assessments"),
			Some(json!({
				"data": { "drug_id": drug_id, "reaction_id": reaction_id.clone(), "sequence_number": 1, "drug_role_characterization": "1" }
			})),
		)
		.await?;
		if let Some(assess_id) = assess_id {
			let _ = put_json(
				hc,
				&format!("/api/cases/{case_id}/drugs/{drug_id}/reaction-assessments/{assess_id}"),
				json!({ "data": { "drug_role_characterization": "2" } }),
			)
			.await?;

			let related_id = ensure_first_id(
				hc,
				&format!("/api/cases/{case_id}/drugs/{drug_id}/reaction-assessments/{assess_id}/relatedness"),
				Some(json!({
					"data": { "drug_reaction_assessment_id": assess_id, "sequence_number": 1, "result_of_assessment": "RELATED" }
				})),
			)
			.await?;
			if let Some(related_id) = related_id {
				let _ = put_json(
					hc,
					&format!("/api/cases/{case_id}/drugs/{drug_id}/reaction-assessments/{assess_id}/relatedness/{related_id}"),
					json!({ "data": { "result_of_assessment": "NOT_RELATED" } }),
				)
				.await?;
			}
		}

		let recur_id = ensure_first_id(
			hc,
			&format!("/api/cases/{case_id}/drugs/{drug_id}/recurrences"),
			Some(json!({
				"data": { "drug_id": drug_id, "sequence_number": 1, "recurrence": "1", "rechallenge": "1" }
			})),
		)
		.await?;
		if let Some(recur_id) = recur_id {
			let _ = put_json(
				hc,
				&format!("/api/cases/{case_id}/drugs/{drug_id}/recurrences/{recur_id}"),
				json!({ "data": { "recurrence": "2" } }),
			)
			.await?;
		}
	}

	// Narrative (singleton)
	let narrative_exists =
		get_singleton_id(hc, &format!("/api/cases/{case_id}/narrative"))
			.await?
			.is_some();
	if narrative_exists {
		let _ = put_json(
			hc,
			&format!("/api/cases/{case_id}/narrative"),
			json!({ "data": { "case_narrative": "Case narrative" } }),
		)
		.await?;
	} else {
		let _ = post_json(
			hc,
			&format!("/api/cases/{case_id}/narrative"),
			json!({ "data": { "case_id": case_id, "case_narrative": "Case narrative" } }),
		)
		.await?;
	}
	let _ = put_json(
		hc,
		&format!("/api/cases/{case_id}/narrative"),
		json!({ "data": { "sender_comments": "Sender comments updated" } }),
	)
	.await?;

	let narrative_id = get_singleton_id(hc, &format!("/api/cases/{case_id}/narrative")).await?;

	// Sender diagnoses
	let diag_id = if let Some(narrative_id) = narrative_id.as_deref() {
		ensure_first_id(
			hc,
			&format!("/api/cases/{case_id}/narrative/sender-diagnoses"),
			Some(json!({
				"data": { "narrative_id": narrative_id, "sequence_number": 1, "diagnosis_meddra_version": "26.0", "diagnosis_meddra_code": "10000008" }
			})),
		)
		.await?
	} else {
		None
	};
	if let Some(diag_id) = diag_id {
		let _ = put_json(
			hc,
			&format!("/api/cases/{case_id}/narrative/sender-diagnoses/{diag_id}"),
			json!({ "data": { "diagnosis_meddra_code": "10000009" } }),
		)
		.await?;
	}

	// Case summaries
	let summary_id = if let Some(narrative_id) = narrative_id.as_deref() {
		ensure_first_id(
			hc,
			&format!("/api/cases/{case_id}/narrative/summaries"),
			Some(json!({
				"data": { "narrative_id": narrative_id, "sequence_number": 1, "summary_type": "01", "language_code": "en", "summary_text": "Summary" }
			})),
		)
		.await?
	} else {
		None
	};
	if let Some(summary_id) = summary_id {
		let _ = put_json(
			hc,
			&format!("/api/cases/{case_id}/narrative/summaries/{summary_id}"),
			json!({ "data": { "summary_text": "Summary updated" } }),
		)
		.await?;
	}

	Ok(())
}

async fn validate_xml(hc: &Client, xml: &str, filename: &str) -> Result<()> {
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

async fn import_xml(hc: &Client, xml: &str) -> Result<String> {
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
	let value: Value = serde_json::from_str(&body)?;
	let case_id = value
		.get("data")
		.and_then(|v| v.get("case_id"))
		.and_then(|v| v.as_str())
		.ok_or("missing case_id in import response")?;
	Ok(case_id.to_string())
}

async fn validate_case(hc: &Client, case_id: &str) -> Result<()> {
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
	let safety_exists =
		get_singleton_id(hc, &format!("/api/cases/{case_id}/safety-report"))
			.await?
			.is_some();
	if safety_exists {
		let _ = put_json(
			hc,
			&format!("/api/cases/{case_id}/safety-report"),
			safety_body,
		)
		.await?;
	} else {
		let _ = post_json(
			hc,
			&format!("/api/cases/{case_id}/safety-report"),
			safety_body,
		)
		.await?;
	}
	let _ = put_json(
		hc,
		&format!("/api/cases/{case_id}"),
		json!({ "data": { "status": "validated" } }),
	)
	.await?;
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

async fn get_singleton_id(hc: &Client, path: &str) -> Result<Option<String>> {
	let cookie = auth_cookie_header(hc)?;
	let res = hc
		.reqwest_client()
		.get(format!("{BASE_URL}{path}"))
		.header("cookie", cookie)
		.send()
		.await?;
	let status = res.status();
	let body = res.text().await?;
	if !status.is_success() {
		return Ok(None);
	}
	let value: Value = serde_json::from_str(&body)?;
	let id = value
		.get("data")
		.and_then(|v| v.get("id"))
		.and_then(|v| v.as_str())
		.map(|v| v.to_string());
	Ok(id)
}

async fn ensure_first_id(
	hc: &Client,
	list_path: &str,
	create_body: Option<Value>,
) -> Result<Option<String>> {
	if let Some(id) = first_id_from_list(hc, list_path).await? {
		return Ok(Some(id));
	}
	if let Some(body) = create_body {
		let created = post_json(hc, list_path, body).await?;
		if let Some(id) = created {
			return Ok(Some(id));
		}
	}
	Ok(None)
}

async fn first_id_from_list(hc: &Client, path: &str) -> Result<Option<String>> {
	let cookie = auth_cookie_header(hc)?;
	let res = hc
		.reqwest_client()
		.get(format!("{BASE_URL}{path}"))
		.header("cookie", cookie)
		.send()
		.await?;
	let status = res.status();
	let body = res.text().await?;
	if !status.is_success() {
		return Ok(None);
	}
	let value: Value = serde_json::from_str(&body)?;
	let id = value
		.get("data")
		.and_then(|v| v.as_array())
		.and_then(|arr| arr.first())
		.and_then(|v| v.get("id"))
		.and_then(|v| v.as_str())
		.map(|v| v.to_string());
	Ok(id)
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
