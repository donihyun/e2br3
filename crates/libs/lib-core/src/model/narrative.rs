// Section H - Narrative and Other Information

use crate::model::base::DbBmc;
use crate::model::store::dbx;
use crate::model::ModelManager;
use crate::model::Result;
use modql::field::Fields;
use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;
use sqlx::types::Uuid;
use sqlx::FromRow;

// -- NarrativeInformation

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct NarrativeInformation {
	pub id: Uuid,
	pub case_id: Uuid,

	// H.1 - Case Narrative
	pub case_narrative: String,

	// H.2 - Reporter's Comments
	pub reporter_comments: Option<String>,

	// H.4 - Sender's Comments
	pub sender_comments: Option<String>,

	// Timestamps
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct NarrativeInformationForCreate {
	pub case_id: Uuid,
	pub case_narrative: String,
}

#[derive(Fields, Deserialize)]
pub struct NarrativeInformationForUpdate {
	pub case_narrative: Option<String>,
	pub reporter_comments: Option<String>,
	pub sender_comments: Option<String>,
}

// -- SenderDiagnosis

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct SenderDiagnosis {
	pub id: Uuid,
	pub narrative_id: Uuid,
	pub sequence_number: i32,

	// H.3.r.1 - Diagnosis/Syndrome (MedDRA coded)
	pub diagnosis_meddra_version: Option<String>,
	pub diagnosis_meddra_code: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct SenderDiagnosisForCreate {
	pub narrative_id: Uuid,
	pub sequence_number: i32,
	pub diagnosis_meddra_code: Option<String>,
}

// -- CaseSummaryInformation

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct CaseSummaryInformation {
	pub id: Uuid,
	pub narrative_id: Uuid,
	pub sequence_number: i32,

	// H.5.r.1 - Case Summary Type
	pub summary_type: Option<String>,

	// H.5.r.2 - Case Summary Language
	pub language_code: Option<String>,

	// H.5.r.3 - Text
	pub summary_text: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct CaseSummaryInformationForCreate {
	pub narrative_id: Uuid,
	pub sequence_number: i32,
	pub summary_text: Option<String>,
}

// -- BMCs

pub struct NarrativeInformationBmc;
impl DbBmc for NarrativeInformationBmc {
	const TABLE: &'static str = "narrative_information";
}

impl NarrativeInformationBmc {
	pub async fn create(
		_mm_ctx: &crate::ctx::Ctx,
		mm: &ModelManager,
		data: NarrativeInformationForCreate,
	) -> Result<Uuid> {
		let sql = format!(
			"INSERT INTO {} (case_id, case_narrative, created_at, updated_at)
			 VALUES ($1, $2, now(), now())
			 RETURNING id",
			Self::TABLE
		);
		let id: Uuid = sqlx::query_scalar(&sql)
			.bind(data.case_id)
			.bind(data.case_narrative)
			.fetch_one(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;
		Ok(id)
	}

	pub async fn get_by_case(
		_ctx: &crate::ctx::Ctx,
		mm: &ModelManager,
		case_id: Uuid,
	) -> Result<NarrativeInformation> {
		let sql = format!("SELECT * FROM {} WHERE case_id = $1", Self::TABLE);
		let narrative = sqlx::query_as::<_, NarrativeInformation>(&sql)
			.bind(case_id)
			.fetch_optional(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;
		narrative.ok_or(crate::model::Error::EntityUuidNotFound {
			entity: Self::TABLE,
			id: case_id,
		})
	}

	pub async fn update_by_case(
		_ctx: &crate::ctx::Ctx,
		mm: &ModelManager,
		case_id: Uuid,
		data: NarrativeInformationForUpdate,
	) -> Result<()> {
		let sql = format!(
			"UPDATE {}
			 SET case_narrative = COALESCE($2, case_narrative),
			     reporter_comments = COALESCE($3, reporter_comments),
			     sender_comments = COALESCE($4, sender_comments),
			     updated_at = now()
			 WHERE case_id = $1",
			Self::TABLE
		);
		let result = sqlx::query(&sql)
			.bind(case_id)
			.bind(data.case_narrative)
			.bind(data.reporter_comments)
			.bind(data.sender_comments)
			.execute(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;
		if result.rows_affected() == 0 {
			return Err(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id: case_id,
			});
		}
		Ok(())
	}

	pub async fn delete_by_case(
		_ctx: &crate::ctx::Ctx,
		mm: &ModelManager,
		case_id: Uuid,
	) -> Result<()> {
		let sql = format!("DELETE FROM {} WHERE case_id = $1", Self::TABLE);
		let result = sqlx::query(&sql)
			.bind(case_id)
			.execute(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;
		if result.rows_affected() == 0 {
			return Err(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id: case_id,
			});
		}
		Ok(())
	}
}

pub struct SenderDiagnosisBmc;
impl DbBmc for SenderDiagnosisBmc {
	const TABLE: &'static str = "sender_diagnoses";
}

pub struct CaseSummaryInformationBmc;
impl DbBmc for CaseSummaryInformationBmc {
	const TABLE: &'static str = "case_summary_information";
}
