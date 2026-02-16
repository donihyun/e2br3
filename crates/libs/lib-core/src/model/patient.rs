// Section D - Patient Information

use crate::ctx::Ctx;
use crate::model::base::base_uuid;
use crate::model::base::DbBmc;
use crate::model::modql_utils::uuid_to_sea_value;
use crate::model::store::set_full_context_dbx_or_rollback;
use crate::model::ModelManager;
use crate::model::Result;
use modql::field::Fields;
use modql::filter::{FilterNodes, ListOptions, OpValsString, OpValsValue};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::types::time::{Date, OffsetDateTime};
use sqlx::types::Uuid;
use sqlx::FromRow;

// -- PatientInformation

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct PatientInformation {
	pub id: Uuid,
	pub case_id: Uuid,

	// D.1 - Patient identification
	pub patient_initials: Option<String>,
	pub patient_given_name: Option<String>,
	pub patient_family_name: Option<String>,

	// D.2 - Age
	pub birth_date: Option<Date>,
	pub age_at_time_of_onset: Option<Decimal>,
	pub age_unit: Option<String>,
	pub gestation_period: Option<Decimal>,
	pub gestation_period_unit: Option<String>,
	pub age_group: Option<String>,

	// D.3-5 - Physical
	pub weight_kg: Option<Decimal>,
	pub height_cm: Option<Decimal>,
	pub sex: Option<String>,

	// FDA.D.11 / FDA.D.12 - Race / Ethnicity (FDA)
	pub race_code: Option<String>,
	pub ethnicity_code: Option<String>,

	// D.6 - Last Menstrual Period
	pub last_menstrual_period_date: Option<Date>,

	// D.7.2 - Medical history
	pub medical_history_text: Option<String>,
	// D.7.3 - Concomitant Therapies
	pub concomitant_therapy: Option<bool>,

	// Timestamps
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
	pub created_by: Uuid,
	pub updated_by: Option<Uuid>,
}

#[derive(Fields, Deserialize)]
pub struct PatientInformationForCreate {
	pub case_id: Uuid,
	pub patient_initials: Option<String>,
	pub sex: Option<String>,
	pub concomitant_therapy: Option<bool>,
}

#[derive(Fields, Deserialize)]
pub struct PatientInformationForUpdate {
	pub patient_initials: Option<String>,
	pub patient_given_name: Option<String>,
	pub patient_family_name: Option<String>,
	#[serde(
		default,
		deserialize_with = "crate::serde::flex_date::deserialize_option_date"
	)]
	pub birth_date: Option<Date>,
	pub age_at_time_of_onset: Option<Decimal>,
	pub age_unit: Option<String>,
	pub gestation_period: Option<Decimal>,
	pub gestation_period_unit: Option<String>,
	pub age_group: Option<String>,
	pub weight_kg: Option<Decimal>,
	pub height_cm: Option<Decimal>,
	pub sex: Option<String>,
	pub race_code: Option<String>,
	pub ethnicity_code: Option<String>,
	#[serde(
		default,
		deserialize_with = "crate::serde::flex_date::deserialize_option_date"
	)]
	pub last_menstrual_period_date: Option<Date>,
	pub medical_history_text: Option<String>,
	pub concomitant_therapy: Option<bool>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct PatientInformationFilter {
	pub patient_initials: Option<OpValsString>,
	pub patient_given_name: Option<OpValsString>,
	pub patient_family_name: Option<OpValsString>,
	pub sex: Option<OpValsString>,
}

// -- PatientIdentifier (D.1.1.x)

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct PatientIdentifier {
	pub id: Uuid,
	pub patient_id: Uuid,
	pub sequence_number: i32,
	pub identifier_type_code: String,
	pub identifier_value: String,
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
	pub created_by: Uuid,
	pub updated_by: Option<Uuid>,
}

#[derive(Fields, Deserialize)]
pub struct PatientIdentifierForCreate {
	pub patient_id: Uuid,
	pub sequence_number: i32,
	pub identifier_type_code: String,
	pub identifier_value: String,
}

#[derive(Fields, Deserialize)]
pub struct PatientIdentifierForUpdate {
	pub identifier_type_code: Option<String>,
	pub identifier_value: Option<String>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct PatientIdentifierFilter {
	#[modql(to_sea_value_fn = "uuid_to_sea_value")]
	pub patient_id: Option<OpValsValue>,
	pub sequence_number: Option<OpValsValue>,
}

// -- MedicalHistoryEpisode

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct MedicalHistoryEpisode {
	pub id: Uuid,
	pub patient_id: Uuid,
	pub sequence_number: i32,

	// D.7.1.r.1a - Disease/Surgical Procedure
	pub meddra_version: Option<String>,
	pub meddra_code: Option<String>,

	// D.7.1.r.2-4
	pub start_date: Option<Date>,
	pub continuing: Option<bool>,
	pub end_date: Option<Date>,
	pub comments: Option<String>,
	pub family_history: Option<bool>,

	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
	pub created_by: Uuid,
	pub updated_by: Option<Uuid>,
}

#[derive(Fields, Deserialize)]
pub struct MedicalHistoryEpisodeForCreate {
	pub patient_id: Uuid,
	pub sequence_number: i32,
	pub meddra_code: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct MedicalHistoryEpisodeForUpdate {
	pub meddra_version: Option<String>,
	pub meddra_code: Option<String>,
	#[serde(
		default,
		deserialize_with = "crate::serde::flex_date::deserialize_option_date"
	)]
	pub start_date: Option<Date>,
	pub continuing: Option<bool>,
	#[serde(
		default,
		deserialize_with = "crate::serde::flex_date::deserialize_option_date"
	)]
	pub end_date: Option<Date>,
	pub comments: Option<String>,
	pub family_history: Option<bool>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct MedicalHistoryEpisodeFilter {
	#[modql(to_sea_value_fn = "uuid_to_sea_value")]
	pub patient_id: Option<OpValsValue>,
	pub sequence_number: Option<OpValsValue>,
}

// -- PastDrugHistory

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct PastDrugHistory {
	pub id: Uuid,
	pub patient_id: Uuid,
	pub sequence_number: i32,

	// D.8.r.1 - Drug Name
	pub drug_name: Option<String>,

	// D.8.r.2-3 - Product IDs
	pub mpid: Option<String>,
	pub mpid_version: Option<String>,
	pub phpid: Option<String>,
	pub phpid_version: Option<String>,

	// D.8.r.4-5 - Dates
	pub start_date: Option<Date>,
	pub end_date: Option<Date>,

	// D.8.r.6a - Indication
	pub indication_meddra_version: Option<String>,
	pub indication_meddra_code: Option<String>,

	// D.8.r.7 - Reaction(s)
	pub reaction_meddra_version: Option<String>,
	pub reaction_meddra_code: Option<String>,

	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
	pub created_by: Uuid,
	pub updated_by: Option<Uuid>,
}

#[derive(Fields, Deserialize)]
pub struct PastDrugHistoryForCreate {
	pub patient_id: Uuid,
	pub sequence_number: i32,
	pub drug_name: Option<String>,
	pub mpid: Option<String>,
	pub mpid_version: Option<String>,
	pub phpid: Option<String>,
	pub phpid_version: Option<String>,
	#[serde(
		default,
		deserialize_with = "crate::serde::flex_date::deserialize_option_date"
	)]
	pub start_date: Option<Date>,
	#[serde(
		default,
		deserialize_with = "crate::serde::flex_date::deserialize_option_date"
	)]
	pub end_date: Option<Date>,
	pub indication_meddra_version: Option<String>,
	pub indication_meddra_code: Option<String>,
	pub reaction_meddra_version: Option<String>,
	pub reaction_meddra_code: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct PastDrugHistoryForUpdate {
	pub drug_name: Option<String>,
	pub mpid: Option<String>,
	pub mpid_version: Option<String>,
	pub phpid: Option<String>,
	pub phpid_version: Option<String>,
	#[serde(
		default,
		deserialize_with = "crate::serde::flex_date::deserialize_option_date"
	)]
	pub start_date: Option<Date>,
	#[serde(
		default,
		deserialize_with = "crate::serde::flex_date::deserialize_option_date"
	)]
	pub end_date: Option<Date>,
	pub indication_meddra_version: Option<String>,
	pub indication_meddra_code: Option<String>,
	pub reaction_meddra_version: Option<String>,
	pub reaction_meddra_code: Option<String>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct PastDrugHistoryFilter {
	#[modql(to_sea_value_fn = "uuid_to_sea_value")]
	pub patient_id: Option<OpValsValue>,
	pub sequence_number: Option<OpValsValue>,
}

// -- PatientDeathInformation

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct PatientDeathInformation {
	pub id: Uuid,
	pub patient_id: Uuid,

	// D.9.1 - Date of Death
	pub date_of_death: Option<Date>,

	// D.9.3 - Autopsy
	pub autopsy_performed: Option<bool>,

	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
	pub created_by: Uuid,
	pub updated_by: Option<Uuid>,
}

#[derive(Fields, Deserialize)]
pub struct PatientDeathInformationForCreate {
	pub patient_id: Uuid,
	#[serde(
		default,
		deserialize_with = "crate::serde::flex_date::deserialize_option_date"
	)]
	pub date_of_death: Option<Date>,
	pub autopsy_performed: Option<bool>,
}

#[derive(Fields, Deserialize)]
pub struct PatientDeathInformationForUpdate {
	#[serde(
		default,
		deserialize_with = "crate::serde::flex_date::deserialize_option_date"
	)]
	pub date_of_death: Option<Date>,
	pub autopsy_performed: Option<bool>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct PatientDeathInformationFilter {
	#[modql(to_sea_value_fn = "uuid_to_sea_value")]
	pub patient_id: Option<OpValsValue>,
}

// -- ReportedCauseOfDeath

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct ReportedCauseOfDeath {
	pub id: Uuid,
	pub death_info_id: Uuid,
	pub sequence_number: i32,
	pub meddra_version: Option<String>,
	pub meddra_code: Option<String>,
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
	pub created_by: Uuid,
	pub updated_by: Option<Uuid>,
}

#[derive(Fields, Deserialize)]
pub struct ReportedCauseOfDeathForCreate {
	pub death_info_id: Uuid,
	pub sequence_number: i32,
	pub meddra_code: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct ReportedCauseOfDeathForUpdate {
	pub meddra_version: Option<String>,
	pub meddra_code: Option<String>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct ReportedCauseOfDeathFilter {
	#[modql(to_sea_value_fn = "uuid_to_sea_value")]
	pub death_info_id: Option<OpValsValue>,
	pub sequence_number: Option<OpValsValue>,
}

// -- AutopsyCauseOfDeath

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct AutopsyCauseOfDeath {
	pub id: Uuid,
	pub death_info_id: Uuid,
	pub sequence_number: i32,
	pub meddra_version: Option<String>,
	pub meddra_code: Option<String>,
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
	pub created_by: Uuid,
	pub updated_by: Option<Uuid>,
}

#[derive(Fields, Deserialize)]
pub struct AutopsyCauseOfDeathForCreate {
	pub death_info_id: Uuid,
	pub sequence_number: i32,
	pub meddra_code: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct AutopsyCauseOfDeathForUpdate {
	pub meddra_version: Option<String>,
	pub meddra_code: Option<String>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct AutopsyCauseOfDeathFilter {
	#[modql(to_sea_value_fn = "uuid_to_sea_value")]
	pub death_info_id: Option<OpValsValue>,
	pub sequence_number: Option<OpValsValue>,
}

// -- ParentInformation

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct ParentInformation {
	pub id: Uuid,
	pub patient_id: Uuid,

	pub parent_identification: Option<String>,
	pub parent_birth_date: Option<Date>,
	pub parent_age: Option<Decimal>,
	pub parent_age_unit: Option<String>,
	pub last_menstrual_period_date: Option<Date>,
	pub weight_kg: Option<Decimal>,
	pub height_cm: Option<Decimal>,
	pub sex: Option<String>,
	pub medical_history_text: Option<String>,

	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
	pub created_by: Uuid,
	pub updated_by: Option<Uuid>,
}

#[derive(Fields, Deserialize)]
pub struct ParentInformationForCreate {
	pub patient_id: Uuid,
	pub sex: Option<String>,
	pub medical_history_text: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct ParentInformationForUpdate {
	pub parent_identification: Option<String>,
	#[serde(
		default,
		deserialize_with = "crate::serde::flex_date::deserialize_option_date"
	)]
	pub parent_birth_date: Option<Date>,
	pub parent_age: Option<Decimal>,
	pub parent_age_unit: Option<String>,
	#[serde(
		default,
		deserialize_with = "crate::serde::flex_date::deserialize_option_date"
	)]
	pub last_menstrual_period_date: Option<Date>,
	pub weight_kg: Option<Decimal>,
	pub height_cm: Option<Decimal>,
	pub sex: Option<String>,
	pub medical_history_text: Option<String>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct ParentInformationFilter {
	#[modql(to_sea_value_fn = "uuid_to_sea_value")]
	pub patient_id: Option<OpValsValue>,
}

// -- BMCs

pub struct PatientInformationBmc;
impl DbBmc for PatientInformationBmc {
	const TABLE: &'static str = "patient_information";
}

impl PatientInformationBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		data: PatientInformationForCreate,
	) -> Result<Uuid> {
		mm.dbx().begin_txn().await?;
		set_full_context_dbx_or_rollback(
			mm.dbx(),
			ctx.user_id(),
			ctx.organization_id(),
			ctx.role(),
		)
		.await?;

		let sql = format!(
			"INSERT INTO {} (case_id, patient_initials, sex, concomitant_therapy, created_at, updated_at, created_by)
			 VALUES ($1, $2, $3, $4, now(), now(), $5)
			 RETURNING id",
			Self::TABLE
		);
		let (id,) = mm
			.dbx()
			.fetch_one(
				sqlx::query_as::<_, (Uuid,)>(&sql)
					.bind(data.case_id)
					.bind(data.patient_initials)
					.bind(data.sex)
					.bind(data.concomitant_therapy)
					.bind(ctx.user_id()),
			)
			.await?;
		mm.dbx().commit_txn().await?;
		Ok(id)
	}

	pub async fn get(
		_ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
	) -> Result<PatientInformation> {
		let sql = format!("SELECT * FROM {} WHERE id = $1", Self::TABLE);
		let patient = mm
			.dbx()
			.fetch_optional(sqlx::query_as::<_, PatientInformation>(&sql).bind(id))
			.await?
			.ok_or(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			})?;
		Ok(patient)
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filters: Option<Vec<PatientInformationFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<PatientInformation>> {
		base_uuid::list::<Self, _, _>(ctx, mm, filters, list_options).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		data: PatientInformationForUpdate,
	) -> Result<()> {
		mm.dbx().begin_txn().await?;
		set_full_context_dbx_or_rollback(
			mm.dbx(),
			ctx.user_id(),
			ctx.organization_id(),
			ctx.role(),
		)
		.await?;

		let sql = format!(
			"UPDATE {}
			 SET patient_initials = COALESCE($2, patient_initials),
			     patient_given_name = COALESCE($3, patient_given_name),
			     patient_family_name = COALESCE($4, patient_family_name),
			     birth_date = COALESCE($5, birth_date),
			     age_at_time_of_onset = COALESCE($6, age_at_time_of_onset),
			     age_unit = COALESCE($7, age_unit),
			     gestation_period = COALESCE($8, gestation_period),
			     gestation_period_unit = COALESCE($9, gestation_period_unit),
			     age_group = COALESCE($10, age_group),
			     weight_kg = COALESCE($11, weight_kg),
			     height_cm = COALESCE($12, height_cm),
			     sex = COALESCE($13, sex),
			     race_code = COALESCE($14, race_code),
			     ethnicity_code = COALESCE($15, ethnicity_code),
			     last_menstrual_period_date = COALESCE($16, last_menstrual_period_date),
			     medical_history_text = COALESCE($17, medical_history_text),
			     concomitant_therapy = COALESCE($18, concomitant_therapy),
			     updated_at = now(),
			     updated_by = $19
			 WHERE id = $1",
			Self::TABLE
		);
		let result = mm
			.dbx()
			.execute(
				sqlx::query(&sql)
					.bind(id)
					.bind(data.patient_initials)
					.bind(data.patient_given_name)
					.bind(data.patient_family_name)
					.bind(data.birth_date)
					.bind(data.age_at_time_of_onset)
					.bind(data.age_unit)
					.bind(data.gestation_period)
					.bind(data.gestation_period_unit)
					.bind(data.age_group)
					.bind(data.weight_kg)
					.bind(data.height_cm)
					.bind(data.sex)
					.bind(data.race_code)
					.bind(data.ethnicity_code)
					.bind(data.last_menstrual_period_date)
					.bind(data.medical_history_text)
					.bind(data.concomitant_therapy)
					.bind(ctx.user_id()),
			)
			.await?;

		if result == 0 {
			mm.dbx().rollback_txn().await?;
			return Err(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			});
		}
		mm.dbx().commit_txn().await?;
		Ok(())
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		mm.dbx().begin_txn().await?;
		set_full_context_dbx_or_rollback(
			mm.dbx(),
			ctx.user_id(),
			ctx.organization_id(),
			ctx.role(),
		)
		.await?;

		let sql = format!("DELETE FROM {} WHERE id = $1", Self::TABLE);
		let result = mm.dbx().execute(sqlx::query(&sql).bind(id)).await?;

		if result == 0 {
			mm.dbx().rollback_txn().await?;
			return Err(crate::model::Error::EntityNotFound {
				entity: Self::TABLE,
				id: 0,
			});
		}
		mm.dbx().commit_txn().await?;
		Ok(())
	}

	pub async fn get_by_case(
		_ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
	) -> Result<PatientInformation> {
		let sql = format!("SELECT * FROM {} WHERE case_id = $1", Self::TABLE);
		let patient = mm
			.dbx()
			.fetch_optional(
				sqlx::query_as::<_, PatientInformation>(&sql).bind(case_id),
			)
			.await?;
		patient.ok_or(crate::model::Error::EntityUuidNotFound {
			entity: Self::TABLE,
			id: case_id,
		})
	}

	pub async fn update_by_case(
		ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
		data: PatientInformationForUpdate,
	) -> Result<()> {
		mm.dbx().begin_txn().await?;
		set_full_context_dbx_or_rollback(
			mm.dbx(),
			ctx.user_id(),
			ctx.organization_id(),
			ctx.role(),
		)
		.await?;

		let sql = format!(
			"UPDATE {}
			 SET patient_initials = COALESCE($2, patient_initials),
			     patient_given_name = COALESCE($3, patient_given_name),
			     patient_family_name = COALESCE($4, patient_family_name),
			     birth_date = COALESCE($5, birth_date),
			     age_at_time_of_onset = COALESCE($6, age_at_time_of_onset),
			     age_unit = COALESCE($7, age_unit),
			     gestation_period = COALESCE($8, gestation_period),
			     gestation_period_unit = COALESCE($9, gestation_period_unit),
			     age_group = COALESCE($10, age_group),
			     weight_kg = COALESCE($11, weight_kg),
			     height_cm = COALESCE($12, height_cm),
			     sex = COALESCE($13, sex),
			     race_code = COALESCE($14, race_code),
			     ethnicity_code = COALESCE($15, ethnicity_code),
			     last_menstrual_period_date = COALESCE($16, last_menstrual_period_date),
			     medical_history_text = COALESCE($17, medical_history_text),
			     concomitant_therapy = COALESCE($18, concomitant_therapy),
			     updated_at = now(),
			     updated_by = $19
			 WHERE case_id = $1",
			Self::TABLE
		);
		let result = mm
			.dbx()
			.execute(
				sqlx::query(&sql)
					.bind(case_id)
					.bind(data.patient_initials)
					.bind(data.patient_given_name)
					.bind(data.patient_family_name)
					.bind(data.birth_date)
					.bind(data.age_at_time_of_onset)
					.bind(data.age_unit)
					.bind(data.gestation_period)
					.bind(data.gestation_period_unit)
					.bind(data.age_group)
					.bind(data.weight_kg)
					.bind(data.height_cm)
					.bind(data.sex)
					.bind(data.race_code)
					.bind(data.ethnicity_code)
					.bind(data.last_menstrual_period_date)
					.bind(data.medical_history_text)
					.bind(data.concomitant_therapy)
					.bind(ctx.user_id()),
			)
			.await?;
		if result == 0 {
			mm.dbx().rollback_txn().await?;
			return Err(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id: case_id,
			});
		}
		mm.dbx().commit_txn().await?;
		Ok(())
	}

	pub async fn delete_by_case(
		ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
	) -> Result<()> {
		mm.dbx().begin_txn().await?;
		set_full_context_dbx_or_rollback(
			mm.dbx(),
			ctx.user_id(),
			ctx.organization_id(),
			ctx.role(),
		)
		.await?;

		let sql = format!("DELETE FROM {} WHERE case_id = $1", Self::TABLE);
		let result = mm.dbx().execute(sqlx::query(&sql).bind(case_id)).await?;
		if result == 0 {
			mm.dbx().rollback_txn().await?;
			return Err(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id: case_id,
			});
		}
		mm.dbx().commit_txn().await?;
		Ok(())
	}
}

pub struct PatientIdentifierBmc;
impl DbBmc for PatientIdentifierBmc {
	const TABLE: &'static str = "patient_identifiers";
}

impl PatientIdentifierBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		data: PatientIdentifierForCreate,
	) -> Result<Uuid> {
		base_uuid::create::<Self, _>(ctx, mm, data).await
	}

	pub async fn get(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
	) -> Result<PatientIdentifier> {
		base_uuid::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filters: Option<Vec<PatientIdentifierFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<PatientIdentifier>> {
		base_uuid::list::<Self, _, _>(ctx, mm, filters, list_options).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		data: PatientIdentifierForUpdate,
	) -> Result<()> {
		base_uuid::update::<Self, _>(ctx, mm, id, data).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		base_uuid::delete::<Self>(ctx, mm, id).await
	}
}

pub struct MedicalHistoryEpisodeBmc;
impl DbBmc for MedicalHistoryEpisodeBmc {
	const TABLE: &'static str = "medical_history_episodes";
}

impl MedicalHistoryEpisodeBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		data: MedicalHistoryEpisodeForCreate,
	) -> Result<Uuid> {
		base_uuid::create::<Self, _>(ctx, mm, data).await
	}

	pub async fn get(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
	) -> Result<MedicalHistoryEpisode> {
		base_uuid::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filters: Option<Vec<MedicalHistoryEpisodeFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<MedicalHistoryEpisode>> {
		base_uuid::list::<Self, _, _>(ctx, mm, filters, list_options).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		data: MedicalHistoryEpisodeForUpdate,
	) -> Result<()> {
		base_uuid::update::<Self, _>(ctx, mm, id, data).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		base_uuid::delete::<Self>(ctx, mm, id).await
	}
}

pub struct PastDrugHistoryBmc;
impl DbBmc for PastDrugHistoryBmc {
	const TABLE: &'static str = "past_drug_history";
}

impl PastDrugHistoryBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		data: PastDrugHistoryForCreate,
	) -> Result<Uuid> {
		base_uuid::create::<Self, _>(ctx, mm, data).await
	}

	pub async fn get(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
	) -> Result<PastDrugHistory> {
		base_uuid::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filters: Option<Vec<PastDrugHistoryFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<PastDrugHistory>> {
		base_uuid::list::<Self, _, _>(ctx, mm, filters, list_options).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		data: PastDrugHistoryForUpdate,
	) -> Result<()> {
		base_uuid::update::<Self, _>(ctx, mm, id, data).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		base_uuid::delete::<Self>(ctx, mm, id).await
	}
}

pub struct PatientDeathInformationBmc;
impl DbBmc for PatientDeathInformationBmc {
	const TABLE: &'static str = "patient_death_information";
}

impl PatientDeathInformationBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		data: PatientDeathInformationForCreate,
	) -> Result<Uuid> {
		base_uuid::create::<Self, _>(ctx, mm, data).await
	}

	pub async fn get(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
	) -> Result<PatientDeathInformation> {
		base_uuid::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filters: Option<Vec<PatientDeathInformationFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<PatientDeathInformation>> {
		base_uuid::list::<Self, _, _>(ctx, mm, filters, list_options).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		data: PatientDeathInformationForUpdate,
	) -> Result<()> {
		base_uuid::update::<Self, _>(ctx, mm, id, data).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		base_uuid::delete::<Self>(ctx, mm, id).await
	}
}

pub struct ReportedCauseOfDeathBmc;
impl DbBmc for ReportedCauseOfDeathBmc {
	const TABLE: &'static str = "reported_causes_of_death";
}

impl ReportedCauseOfDeathBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		data: ReportedCauseOfDeathForCreate,
	) -> Result<Uuid> {
		base_uuid::create::<Self, _>(ctx, mm, data).await
	}

	pub async fn get(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
	) -> Result<ReportedCauseOfDeath> {
		base_uuid::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filters: Option<Vec<ReportedCauseOfDeathFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<ReportedCauseOfDeath>> {
		base_uuid::list::<Self, _, _>(ctx, mm, filters, list_options).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		data: ReportedCauseOfDeathForUpdate,
	) -> Result<()> {
		base_uuid::update::<Self, _>(ctx, mm, id, data).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		base_uuid::delete::<Self>(ctx, mm, id).await
	}
}

pub struct AutopsyCauseOfDeathBmc;
impl DbBmc for AutopsyCauseOfDeathBmc {
	const TABLE: &'static str = "autopsy_causes_of_death";
}

impl AutopsyCauseOfDeathBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		data: AutopsyCauseOfDeathForCreate,
	) -> Result<Uuid> {
		base_uuid::create::<Self, _>(ctx, mm, data).await
	}

	pub async fn get(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
	) -> Result<AutopsyCauseOfDeath> {
		base_uuid::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filters: Option<Vec<AutopsyCauseOfDeathFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<AutopsyCauseOfDeath>> {
		base_uuid::list::<Self, _, _>(ctx, mm, filters, list_options).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		data: AutopsyCauseOfDeathForUpdate,
	) -> Result<()> {
		base_uuid::update::<Self, _>(ctx, mm, id, data).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		base_uuid::delete::<Self>(ctx, mm, id).await
	}
}

pub struct ParentInformationBmc;
impl DbBmc for ParentInformationBmc {
	const TABLE: &'static str = "parent_information";
}

impl ParentInformationBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		data: ParentInformationForCreate,
	) -> Result<Uuid> {
		base_uuid::create::<Self, _>(ctx, mm, data).await
	}

	pub async fn get(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
	) -> Result<ParentInformation> {
		base_uuid::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filters: Option<Vec<ParentInformationFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<ParentInformation>> {
		base_uuid::list::<Self, _, _>(ctx, mm, filters, list_options).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		data: ParentInformationForUpdate,
	) -> Result<()> {
		base_uuid::update::<Self, _>(ctx, mm, id, data).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		base_uuid::delete::<Self>(ctx, mm, id).await
	}
}
