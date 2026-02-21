use std::collections::HashMap;
use std::sync::OnceLock;

use lib_core::ctx::Ctx;
use lib_core::model::case::{CaseBmc, CaseForUpdate};
use lib_core::model::ModelManager;
use lib_core::xml::{export_case_xml, validate_e2b_xml};
use lib_rest_core::{Error, Result};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::runtime::Handle;
use tokio::sync::RwLock;
use tokio::task;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionStatus {
	Ack1Received,
	Ack2Received,
	Ack3Received,
	Ack4Received,
	Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionAck {
	pub level: u8,
	pub success: bool,
	pub code: Option<String>,
	pub message: Option<String>,
	pub received_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionRecord {
	pub id: Uuid,
	pub case_id: Uuid,
	pub gateway: String,
	pub remote_submission_id: String,
	pub status: SubmissionStatus,
	pub xml_bytes: usize,
	pub submitted_by: Uuid,
	pub submitted_at: OffsetDateTime,
	pub ack1: Option<SubmissionAck>,
	pub ack2: Option<SubmissionAck>,
	pub ack3: Option<SubmissionAck>,
	pub ack4: Option<SubmissionAck>,
}

#[derive(Debug, Deserialize)]
pub struct MockAckInput {
	pub level: u8,
	#[serde(default = "default_true")]
	pub success: bool,
	pub code: Option<String>,
	pub message: Option<String>,
}

fn default_true() -> bool {
	true
}

type SubmissionStore = RwLock<HashMap<Uuid, SubmissionRecord>>;

fn store() -> &'static SubmissionStore {
	static STORE: OnceLock<SubmissionStore> = OnceLock::new();
	STORE.get_or_init(|| RwLock::new(HashMap::new()))
}

fn is_fda_profile(case_profile: Option<&str>) -> bool {
	case_profile
		.map(|v| v.eq_ignore_ascii_case("fda"))
		.unwrap_or(true)
}

fn status_from_ack(level: u8, success: bool) -> Result<SubmissionStatus> {
	if !matches!(level, 1 | 2 | 3 | 4) {
		return Err(Error::BadRequest {
			message: "ack level must be one of: 1, 2, 3, 4".to_string(),
		});
	}
	if !success {
		return Ok(SubmissionStatus::Rejected);
	}
	let status = match level {
		1 => SubmissionStatus::Ack1Received,
		2 => SubmissionStatus::Ack2Received,
		3 => SubmissionStatus::Ack3Received,
		4 => SubmissionStatus::Ack4Received,
		_ => unreachable!(),
	};
	Ok(status)
}

pub async fn create_fda_submission(
	ctx: &Ctx,
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<SubmissionRecord> {
	let case = CaseBmc::get(ctx, mm, case_id).await?;
	if !is_fda_profile(case.validation_profile.as_deref()) {
		return Err(Error::BadRequest {
			message: "case validation_profile must be fda for FDA submission"
				.to_string(),
		});
	}

	let ctx_clone = ctx.clone();
	let mm_clone = mm.clone();
	let xml = task::spawn_blocking(move || {
		Handle::current().block_on(export_case_xml(&ctx_clone, &mm_clone, case_id))
	})
	.await
	.map_err(|err| Error::BadRequest {
		message: format!("submission export task failed: {err}"),
	})?
	.map_err(Error::from)?;
	let report = validate_e2b_xml(xml.as_bytes(), None).map_err(Error::from)?;
	if !report.ok {
		let preview = report
			.errors
			.iter()
			.take(3)
			.map(|err| err.message.as_str())
			.collect::<Vec<_>>()
			.join("; ");
		return Err(Error::BadRequest {
			message: format!(
				"cannot submit case: XML validation failed ({} issue(s)): {}",
				report.errors.len(),
				preview
			),
		});
	}

	let now = OffsetDateTime::now_utc();
	let submission_id = Uuid::new_v4();
	let remote_submission_id = format!(
		"FDA-MOCK-{}",
		submission_id.simple().to_string().to_uppercase()
	);
	let ack1 = SubmissionAck {
		level: 1,
		success: true,
		code: Some("ACK1_ACCEPTED".to_string()),
		message: Some("Upload accepted by mock FDA gateway".to_string()),
		received_at: now,
	};

	CaseBmc::update(
		ctx,
		mm,
		case_id,
		CaseForUpdate {
			safety_report_id: None,
			dg_prd_key: None,
			status: Some("submitted".to_string()),
			validation_profile: None,
			submitted_by: Some(ctx.user_id()),
			submitted_at: Some(now),
			raw_xml: Some(xml.as_bytes().to_vec()),
			dirty_c: Some(false),
			dirty_d: Some(false),
			dirty_e: Some(false),
			dirty_f: Some(false),
			dirty_g: Some(false),
			dirty_h: Some(false),
		},
	)
	.await?;

	let record = SubmissionRecord {
		id: submission_id,
		case_id,
		gateway: "fda-esg-nextgen-mock".to_string(),
		remote_submission_id,
		status: SubmissionStatus::Ack1Received,
		xml_bytes: xml.len(),
		submitted_by: ctx.user_id(),
		submitted_at: now,
		ack1: Some(ack1),
		ack2: None,
		ack3: None,
		ack4: None,
	};

	let mut st = store().write().await;
	st.insert(record.id, record.clone());
	Ok(record)
}

pub async fn list_by_case(case_id: Uuid) -> Vec<SubmissionRecord> {
	let st = store().read().await;
	let mut rows: Vec<SubmissionRecord> = st
		.values()
		.filter(|row| row.case_id == case_id)
		.cloned()
		.collect();
	rows.sort_by(|a, b| b.submitted_at.cmp(&a.submitted_at));
	rows
}

pub async fn get_submission(id: Uuid) -> Option<SubmissionRecord> {
	let st = store().read().await;
	st.get(&id).cloned()
}

pub async fn apply_mock_ack(
	submission_id: Uuid,
	input: MockAckInput,
) -> Result<SubmissionRecord> {
	let status = status_from_ack(input.level, input.success)?;
	let ack = SubmissionAck {
		level: input.level,
		success: input.success,
		code: input.code,
		message: input.message,
		received_at: OffsetDateTime::now_utc(),
	};

	let mut st = store().write().await;
	let record = st.get_mut(&submission_id).ok_or(Error::BadRequest {
		message: format!("submission not found: {submission_id}"),
	})?;

	match ack.level {
		1 => record.ack1 = Some(ack),
		2 => record.ack2 = Some(ack),
		3 => record.ack3 = Some(ack),
		4 => record.ack4 = Some(ack),
		_ => unreachable!(),
	}
	record.status = status;
	Ok(record.clone())
}

#[cfg(test)]
mod tests {
	use super::{status_from_ack, SubmissionStatus};

	#[test]
	fn ack_status_mapping_success() {
		assert_eq!(
			status_from_ack(1, true).unwrap(),
			SubmissionStatus::Ack1Received
		);
		assert_eq!(
			status_from_ack(2, true).unwrap(),
			SubmissionStatus::Ack2Received
		);
		assert_eq!(
			status_from_ack(3, true).unwrap(),
			SubmissionStatus::Ack3Received
		);
		assert_eq!(
			status_from_ack(4, true).unwrap(),
			SubmissionStatus::Ack4Received
		);
	}

	#[test]
	fn ack_status_mapping_rejected() {
		assert_eq!(
			status_from_ack(2, false).unwrap(),
			SubmissionStatus::Rejected
		);
	}
}
