// Section E - Reaction/Event

use crate::ctx::Ctx;
use crate::model::base::DbBmc;
use crate::model::store::dbx;
use crate::model::ModelManager;
use crate::model::Result;
use modql::field::Fields;
use modql::filter::{FilterNodes, OpValsBool, OpValsValue};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::types::time::{Date, OffsetDateTime};
use sqlx::types::Uuid;
use sqlx::FromRow;

// -- Reaction

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct Reaction {
	pub id: Uuid,
	pub case_id: Uuid,
	pub sequence_number: i32,

	// E.i.1.1 - Reaction as reported
	pub primary_source_reaction: String,
	pub reaction_language: Option<String>,

	// E.i.2.1 - MedDRA coding
	pub reaction_meddra_version: Option<String>,
	pub reaction_meddra_code: Option<String>,

	// E.i.3 - Term Highlighted by Reporter
	pub term_highlighted: Option<bool>,

	// E.i.3.1 - Seriousness (MANDATORY if serious)
	pub serious: Option<bool>,

	// E.i.3.2 - Seriousness Criteria
	pub criteria_death: bool,
	pub criteria_life_threatening: bool,
	pub criteria_hospitalization: bool,
	pub criteria_disabling: bool,
	pub criteria_congenital_anomaly: bool,
	pub criteria_other_medically_important: bool,

	// E.i.4-6 - Timing
	pub start_date: Option<Date>,
	pub end_date: Option<Date>,
	pub duration_value: Option<Decimal>,
	pub duration_unit: Option<String>,

	// E.i.7 - Outcome
	pub outcome: Option<String>,

	// E.i.8 - Medical Confirmation
	pub medical_confirmation: Option<bool>,

	// E.i.9 - Country
	pub country_code: Option<String>,

	// Timestamps
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct ReactionForCreate {
	pub case_id: Uuid,
	pub sequence_number: i32,
	pub primary_source_reaction: String,
}

#[derive(Fields, Deserialize)]
pub struct ReactionForUpdate {
	pub primary_source_reaction: Option<String>,
	pub reaction_meddra_code: Option<String>,
	pub reaction_meddra_version: Option<String>,
	pub serious: Option<bool>,
	pub criteria_death: Option<bool>,
	pub criteria_life_threatening: Option<bool>,
	pub criteria_hospitalization: Option<bool>,
	pub start_date: Option<Date>,
	pub end_date: Option<Date>,
	pub outcome: Option<String>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct ReactionFilter {
	pub case_id: Option<OpValsValue>,
	pub serious: Option<OpValsBool>,
}

// -- BMC

pub struct ReactionBmc;
impl DbBmc for ReactionBmc {
	const TABLE: &'static str = "reactions";
}

impl ReactionBmc {
	pub async fn create(
		_ctx: &Ctx,
		mm: &ModelManager,
		reaction_c: ReactionForCreate,
	) -> Result<Uuid> {
		let sql = format!(
			"INSERT INTO {} (case_id, sequence_number, primary_source_reaction, created_at, updated_at)
			 VALUES ($1, $2, $3, now(), now())
			 RETURNING id",
			Self::TABLE
		);
		let id: Uuid = sqlx::query_scalar(&sql)
			.bind(reaction_c.case_id)
			.bind(reaction_c.sequence_number)
			.bind(reaction_c.primary_source_reaction)
			.fetch_one(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;
		Ok(id)
	}

	pub async fn get(_ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<Reaction> {
		let sql = format!("SELECT * FROM {} WHERE id = $1", Self::TABLE);
		let reaction = sqlx::query_as::<_, Reaction>(&sql)
			.bind(id)
			.fetch_optional(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?
			.ok_or(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			})?;
		Ok(reaction)
	}

	pub async fn update(
		_ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		reaction_u: ReactionForUpdate,
	) -> Result<()> {
		let sql = format!(
			"UPDATE {}
			 SET primary_source_reaction = COALESCE($2, primary_source_reaction),
			     reaction_meddra_code = COALESCE($3, reaction_meddra_code),
			     reaction_meddra_version = COALESCE($4, reaction_meddra_version),
			     serious = COALESCE($5, serious),
			     criteria_death = COALESCE($6, criteria_death),
			     criteria_life_threatening = COALESCE($7, criteria_life_threatening),
			     criteria_hospitalization = COALESCE($8, criteria_hospitalization),
			     start_date = COALESCE($9, start_date),
			     end_date = COALESCE($10, end_date),
			     outcome = COALESCE($11, outcome),
			     updated_at = now()
			 WHERE id = $1",
			Self::TABLE
		);
		let result = sqlx::query(&sql)
			.bind(id)
			.bind(reaction_u.primary_source_reaction)
			.bind(reaction_u.reaction_meddra_code)
			.bind(reaction_u.reaction_meddra_version)
			.bind(reaction_u.serious)
			.bind(reaction_u.criteria_death)
			.bind(reaction_u.criteria_life_threatening)
			.bind(reaction_u.criteria_hospitalization)
			.bind(reaction_u.start_date)
			.bind(reaction_u.end_date)
			.bind(reaction_u.outcome)
			.execute(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;
		if result.rows_affected() == 0 {
			return Err(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			});
		}
		Ok(())
	}

	pub async fn list_by_case(
		_ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
	) -> Result<Vec<Reaction>> {
		let sql = format!(
			"SELECT * FROM {} WHERE case_id = $1 ORDER BY sequence_number",
			Self::TABLE
		);
		let reactions = sqlx::query_as::<_, Reaction>(&sql)
			.bind(case_id)
			.fetch_all(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;
		Ok(reactions)
	}

	pub async fn get_in_case(
		_ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
		id: Uuid,
	) -> Result<Reaction> {
		let sql = format!(
			"SELECT * FROM {} WHERE id = $1 AND case_id = $2",
			Self::TABLE
		);
		let reaction = sqlx::query_as::<_, Reaction>(&sql)
			.bind(id)
			.bind(case_id)
			.fetch_optional(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?
			.ok_or(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			})?;
		Ok(reaction)
	}

	pub async fn update_in_case(
		_ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
		id: Uuid,
		reaction_u: ReactionForUpdate,
	) -> Result<()> {
		let sql = format!(
			"UPDATE {}
			 SET primary_source_reaction = COALESCE($3, primary_source_reaction),
			     reaction_meddra_code = COALESCE($4, reaction_meddra_code),
			     reaction_meddra_version = COALESCE($5, reaction_meddra_version),
			     serious = COALESCE($6, serious),
			     criteria_death = COALESCE($7, criteria_death),
			     criteria_life_threatening = COALESCE($8, criteria_life_threatening),
			     criteria_hospitalization = COALESCE($9, criteria_hospitalization),
			     start_date = COALESCE($10, start_date),
			     end_date = COALESCE($11, end_date),
			     outcome = COALESCE($12, outcome),
			     updated_at = now()
			 WHERE id = $1 AND case_id = $2",
			Self::TABLE
		);
		let result = sqlx::query(&sql)
			.bind(id)
			.bind(case_id)
			.bind(reaction_u.primary_source_reaction)
			.bind(reaction_u.reaction_meddra_code)
			.bind(reaction_u.reaction_meddra_version)
			.bind(reaction_u.serious)
			.bind(reaction_u.criteria_death)
			.bind(reaction_u.criteria_life_threatening)
			.bind(reaction_u.criteria_hospitalization)
			.bind(reaction_u.start_date)
			.bind(reaction_u.end_date)
			.bind(reaction_u.outcome)
			.execute(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;
		if result.rows_affected() == 0 {
			return Err(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			});
		}
		Ok(())
	}

	pub async fn delete(
		_ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
	) -> Result<()> {
		let sql = format!("DELETE FROM {} WHERE id = $1", Self::TABLE);
		let result = sqlx::query(&sql)
			.bind(id)
			.execute(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;
		if result.rows_affected() == 0 {
			return Err(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			});
		}
		Ok(())
	}

	pub async fn delete_in_case(
		_ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
		id: Uuid,
	) -> Result<()> {
		let sql = format!(
			"DELETE FROM {} WHERE id = $1 AND case_id = $2",
			Self::TABLE
		);
		let result = sqlx::query(&sql)
			.bind(id)
			.bind(case_id)
			.execute(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;
		if result.rows_affected() == 0 {
			return Err(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			});
		}
		Ok(())
	}
}
