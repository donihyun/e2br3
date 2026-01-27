// D.10.7 - Parent Medical History
// D.10.8 - Parent Past Drug History

use crate::ctx::Ctx;
use crate::model::base::base_uuid;
use crate::model::base::DbBmc;
use crate::model::modql_utils::uuid_to_sea_value;
use crate::model::ModelManager;
use crate::model::Result;
use modql::field::Fields;
use modql::filter::{FilterNodes, ListOptions, OpValsValue};
use serde::{Deserialize, Serialize};
use sqlx::types::time::{Date, OffsetDateTime};
use sqlx::types::Uuid;
use sqlx::FromRow;

// -- ParentMedicalHistory
// D.10.7.1.r - Parent's relevant medical history episodes

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct ParentMedicalHistory {
	pub id: Uuid,
	pub parent_id: Uuid,
	pub sequence_number: i32,

	// D.10.7.1.r.1a - MedDRA Version
	pub meddra_version: Option<String>,

	// D.10.7.1.r.1b - Parent's Relevant Medical History (MedDRA code)
	pub meddra_code: Option<String>,

	// D.10.7.1.r.2 - Start Date
	pub start_date: Option<Date>,

	// D.10.7.1.r.3 - Continuing
	pub continuing: Option<bool>,

	// D.10.7.1.r.4 - End Date
	pub end_date: Option<Date>,

	// D.10.7.1.r.5 - Comments
	pub comments: Option<String>,

	// Timestamps
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
	pub created_by: Uuid,
	pub updated_by: Option<Uuid>,
}

#[derive(Fields, Deserialize)]
pub struct ParentMedicalHistoryForCreate {
	pub parent_id: Uuid,
	pub sequence_number: i32,
	pub meddra_code: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct ParentMedicalHistoryForUpdate {
	pub meddra_version: Option<String>,
	pub meddra_code: Option<String>,
	pub start_date: Option<Date>,
	pub continuing: Option<bool>,
	pub end_date: Option<Date>,
	pub comments: Option<String>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct ParentMedicalHistoryFilter {
	#[modql(to_sea_value_fn = "uuid_to_sea_value")]
	pub parent_id: Option<OpValsValue>,
	pub sequence_number: Option<OpValsValue>,
}

// -- ParentPastDrugHistory
// D.10.8.r - Parent's past drug history

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct ParentPastDrugHistory {
	pub id: Uuid,
	pub parent_id: Uuid,
	pub sequence_number: i32,

	// D.10.8.r.1 - Drug Name
	pub drug_name: Option<String>,

	// D.10.8.r.2 - MPID
	pub mpid: Option<String>,
	pub mpid_version: Option<String>,

	// D.10.8.r.3 - PhPID
	pub phpid: Option<String>,
	pub phpid_version: Option<String>,

	// D.10.8.r.4 - Start Date
	pub start_date: Option<Date>,

	// D.10.8.r.5 - End Date
	pub end_date: Option<Date>,

	// D.10.8.r.6a/b - Indication (MedDRA)
	pub indication_meddra_version: Option<String>,
	pub indication_meddra_code: Option<String>,

	// D.10.8.r.7a/b - Reaction (MedDRA)
	pub reaction_meddra_version: Option<String>,
	pub reaction_meddra_code: Option<String>,

	// Timestamps
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
	pub created_by: Uuid,
	pub updated_by: Option<Uuid>,
}

#[derive(Fields, Deserialize)]
pub struct ParentPastDrugHistoryForCreate {
	pub parent_id: Uuid,
	pub sequence_number: i32,
	pub drug_name: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct ParentPastDrugHistoryForUpdate {
	pub drug_name: Option<String>,
	pub mpid: Option<String>,
	pub mpid_version: Option<String>,
	pub phpid: Option<String>,
	pub phpid_version: Option<String>,
	pub start_date: Option<Date>,
	pub end_date: Option<Date>,
	pub indication_meddra_version: Option<String>,
	pub indication_meddra_code: Option<String>,
	pub reaction_meddra_version: Option<String>,
	pub reaction_meddra_code: Option<String>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct ParentPastDrugHistoryFilter {
	#[modql(to_sea_value_fn = "uuid_to_sea_value")]
	pub parent_id: Option<OpValsValue>,
	pub sequence_number: Option<OpValsValue>,
}

// -- BMCs

pub struct ParentMedicalHistoryBmc;
impl DbBmc for ParentMedicalHistoryBmc {
	const TABLE: &'static str = "parent_medical_history";
}

impl ParentMedicalHistoryBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		data: ParentMedicalHistoryForCreate,
	) -> Result<Uuid> {
		base_uuid::create::<Self, _>(ctx, mm, data).await
	}

	pub async fn get(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
	) -> Result<ParentMedicalHistory> {
		base_uuid::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filters: Option<Vec<ParentMedicalHistoryFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<ParentMedicalHistory>> {
		base_uuid::list::<Self, _, _>(ctx, mm, filters, list_options).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		data: ParentMedicalHistoryForUpdate,
	) -> Result<()> {
		base_uuid::update::<Self, _>(ctx, mm, id, data).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		base_uuid::delete::<Self>(ctx, mm, id).await
	}
}

pub struct ParentPastDrugHistoryBmc;
impl DbBmc for ParentPastDrugHistoryBmc {
	const TABLE: &'static str = "parent_past_drug_history";
}

impl ParentPastDrugHistoryBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		data: ParentPastDrugHistoryForCreate,
	) -> Result<Uuid> {
		base_uuid::create::<Self, _>(ctx, mm, data).await
	}

	pub async fn get(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
	) -> Result<ParentPastDrugHistory> {
		base_uuid::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filters: Option<Vec<ParentPastDrugHistoryFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<ParentPastDrugHistory>> {
		base_uuid::list::<Self, _, _>(ctx, mm, filters, list_options).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		data: ParentPastDrugHistoryForUpdate,
	) -> Result<()> {
		base_uuid::update::<Self, _>(ctx, mm, id, data).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		base_uuid::delete::<Self>(ctx, mm, id).await
	}
}
