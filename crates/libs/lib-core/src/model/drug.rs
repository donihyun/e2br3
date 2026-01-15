// Section G - Drug/Biological Information

use crate::ctx::Ctx;
use crate::model::base::base_uuid;
use crate::model::base::DbBmc;
use crate::model::store::{dbx, set_user_context};
use crate::model::ModelManager;
use crate::model::Result;
use modql::field::Fields;
use modql::filter::{FilterNodes, ListOptions, OpValsValue};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::types::time::{Date, OffsetDateTime, Time};
use sqlx::types::Uuid;
use sqlx::FromRow;

// -- DrugInformation

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct DrugInformation {
	pub id: Uuid,
	pub case_id: Uuid,
	pub sequence_number: i32,

	// G.k.1 - Drug role (MANDATORY)
	pub drug_characterization: String,

	// G.k.2.2 - Product name
	pub medicinal_product: String,

	// G.k.2.4-5 - Product identifiers
	pub mpid: Option<String>,
	pub mpid_version: Option<String>,
	pub phpid: Option<String>,
	pub phpid_version: Option<String>,

	// G.k.3.1 - Obtain Drug Country
	pub obtain_drug_country: Option<String>,

	// G.k.3.2 - Brand Name
	pub brand_name: Option<String>,

	// G.k.3.3 - Manufacturer
	pub manufacturer_name: Option<String>,
	pub manufacturer_country: Option<String>,

	// G.k.3.4 - Batch/Lot Number
	pub batch_lot_number: Option<String>,

	// G.k.5 - Dosage Text
	pub dosage_text: Option<String>,

	// G.k.7 - Action taken
	pub action_taken: Option<String>,

	// G.k.8 - Rechallenge/Recurrence
	pub rechallenge: Option<String>,

	// G.k.10 - Parent Route
	pub parent_route: Option<String>,

	// G.k.11 - Parent Dosage
	pub parent_dosage_text: Option<String>,

	// Timestamps
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
	pub created_by: Uuid,
	pub updated_by: Option<Uuid>,
}

#[derive(Fields, Deserialize)]
pub struct DrugInformationForCreate {
	pub case_id: Uuid,
	pub sequence_number: i32,
	pub drug_characterization: String,
	pub medicinal_product: String,
}

#[derive(Fields, Deserialize)]
pub struct DrugInformationForUpdate {
	pub medicinal_product: Option<String>,
	pub drug_characterization: Option<String>,
	pub brand_name: Option<String>,
	pub manufacturer_name: Option<String>,
	pub batch_lot_number: Option<String>,
	pub action_taken: Option<String>,
}

// -- DrugActiveSubstance

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct DrugActiveSubstance {
	pub id: Uuid,
	pub drug_id: Uuid,
	pub sequence_number: i32,

	// G.k.2.3.r.1 - Substance Name
	pub substance_name: Option<String>,

	// G.k.2.3.r.2 - Substance TermID
	pub substance_termid: Option<String>,
	pub substance_termid_version: Option<String>,

	// G.k.2.3.r.3 - Strength
	pub strength_value: Option<Decimal>,
	pub strength_unit: Option<String>,
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
	pub created_by: Uuid,
	pub updated_by: Option<Uuid>,
}

#[derive(Fields, Deserialize)]
pub struct DrugActiveSubstanceForCreate {
	pub drug_id: Uuid,
	pub sequence_number: i32,
	pub substance_name: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct DrugActiveSubstanceForUpdate {
	pub substance_name: Option<String>,
	pub substance_termid: Option<String>,
	pub substance_termid_version: Option<String>,
	pub strength_value: Option<Decimal>,
	pub strength_unit: Option<String>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct DrugActiveSubstanceFilter {
	pub drug_id: Option<OpValsValue>,
	pub sequence_number: Option<OpValsValue>,
}

// -- DosageInformation

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct DosageInformation {
	pub id: Uuid,
	pub drug_id: Uuid,
	pub sequence_number: i32,

	// G.k.4.r.1 - Dose
	pub dose_value: Option<Decimal>,
	pub dose_unit: Option<String>,

	// G.k.4.r.2 - Number of Separate Dosages
	pub number_of_units: Option<i32>,

	// G.k.4.r.3 - Dose Frequency
	pub frequency_value: Option<Decimal>,
	pub frequency_unit: Option<String>,

	// G.k.4.r.4 - Date/Time of First Administration
	pub first_administration_date: Option<Date>,
	pub first_administration_time: Option<Time>,

	// G.k.4.r.5 - Date/Time of Last Administration
	pub last_administration_date: Option<Date>,
	pub last_administration_time: Option<Time>,

	// G.k.4.r.6 - Duration
	pub duration_value: Option<Decimal>,
	pub duration_unit: Option<String>,

	// G.k.4.r.7 - Batch/Lot Number
	pub batch_lot_number: Option<String>,

	// G.k.4.r.8 - Dosage Text
	pub dosage_text: Option<String>,

	// G.k.4.r.9.1 - Pharmaceutical Dose Form
	pub dose_form: Option<String>,

	// G.k.4.r.10 - Route of Administration
	pub route_of_administration: Option<String>,

	// G.k.4.r.11 - Parent Route
	pub parent_route: Option<String>,

	// Timestamps
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
	pub created_by: Uuid,
	pub updated_by: Option<Uuid>,
}

#[derive(Fields, Deserialize)]
pub struct DosageInformationForCreate {
	pub drug_id: Uuid,
	pub sequence_number: i32,
}

#[derive(Fields, Deserialize)]
pub struct DosageInformationForUpdate {
	pub dose_value: Option<Decimal>,
	pub dose_unit: Option<String>,
	pub number_of_units: Option<i32>,
	pub frequency_value: Option<Decimal>,
	pub frequency_unit: Option<String>,
	pub first_administration_date: Option<Date>,
	pub first_administration_time: Option<Time>,
	pub last_administration_date: Option<Date>,
	pub last_administration_time: Option<Time>,
	pub duration_value: Option<Decimal>,
	pub duration_unit: Option<String>,
	pub batch_lot_number: Option<String>,
	pub dosage_text: Option<String>,
	pub dose_form: Option<String>,
	pub route_of_administration: Option<String>,
	pub parent_route: Option<String>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct DosageInformationFilter {
	pub drug_id: Option<OpValsValue>,
	pub sequence_number: Option<OpValsValue>,
}

// -- DrugIndication

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct DrugIndication {
	pub id: Uuid,
	pub drug_id: Uuid,
	pub sequence_number: i32,

	// G.k.6.r.1 - Indication (free text)
	pub indication_text: Option<String>,

	// G.k.6.r.2 - Indication (MedDRA coded)
	pub indication_meddra_version: Option<String>,
	pub indication_meddra_code: Option<String>,

	// Timestamps
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
	pub created_by: Uuid,
	pub updated_by: Option<Uuid>,
}

#[derive(Fields, Deserialize)]
pub struct DrugIndicationForCreate {
	pub drug_id: Uuid,
	pub sequence_number: i32,
	pub indication_text: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct DrugIndicationForUpdate {
	pub indication_text: Option<String>,
	pub indication_meddra_version: Option<String>,
	pub indication_meddra_code: Option<String>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct DrugIndicationFilter {
	pub drug_id: Option<OpValsValue>,
	pub sequence_number: Option<OpValsValue>,
}

// -- BMCs

pub struct DrugInformationBmc;
impl DbBmc for DrugInformationBmc {
	const TABLE: &'static str = "drug_information";
}

impl DrugInformationBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		drug_c: DrugInformationForCreate,
	) -> Result<Uuid> {
		let db = mm.dbx().db();
		let mut tx = db.begin().await.map_err(|e| dbx::Error::from(e))?;
		set_user_context(&mut tx, ctx.user_id()).await?;

		let sql = format!(
			"INSERT INTO {} (case_id, sequence_number, drug_characterization, medicinal_product, created_at, updated_at, created_by)
			 VALUES ($1, $2, $3, $4, now(), now(), $5)
			 RETURNING id",
			Self::TABLE
		);
		let id: Uuid = sqlx::query_scalar(&sql)
			.bind(drug_c.case_id)
			.bind(drug_c.sequence_number)
			.bind(drug_c.drug_characterization)
			.bind(drug_c.medicinal_product)
			.bind(ctx.user_id())
			.fetch_one(&mut *tx)
			.await
			.map_err(|e| dbx::Error::from(e))?;

		tx.commit().await.map_err(|e| dbx::Error::from(e))?;
		Ok(id)
	}

	pub async fn get(_ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<DrugInformation> {
		let sql = format!("SELECT * FROM {} WHERE id = $1", Self::TABLE);
		let drug = sqlx::query_as::<_, DrugInformation>(&sql)
			.bind(id)
			.fetch_optional(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?
			.ok_or(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			})?;
		Ok(drug)
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		drug_u: DrugInformationForUpdate,
	) -> Result<()> {
		let db = mm.dbx().db();
		let mut tx = db.begin().await.map_err(|e| dbx::Error::from(e))?;
		set_user_context(&mut tx, ctx.user_id()).await?;

		let sql = format!(
			"UPDATE {}
			 SET medicinal_product = COALESCE($2, medicinal_product),
			     drug_characterization = COALESCE($3, drug_characterization),
			     brand_name = COALESCE($4, brand_name),
			     manufacturer_name = COALESCE($5, manufacturer_name),
			     batch_lot_number = COALESCE($6, batch_lot_number),
			     action_taken = COALESCE($7, action_taken),
			     updated_at = now(),
			     updated_by = $8
			 WHERE id = $1",
			Self::TABLE
		);
		let result = sqlx::query(&sql)
			.bind(id)
			.bind(drug_u.medicinal_product)
			.bind(drug_u.drug_characterization)
			.bind(drug_u.brand_name)
			.bind(drug_u.manufacturer_name)
			.bind(drug_u.batch_lot_number)
			.bind(drug_u.action_taken)
			.bind(ctx.user_id())
			.execute(&mut *tx)
			.await
			.map_err(|e| dbx::Error::from(e))?;
		if result.rows_affected() == 0 {
			return Err(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			});
		}
		tx.commit().await.map_err(|e| dbx::Error::from(e))?;
		Ok(())
	}

	pub async fn list_by_case(
		_ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
	) -> Result<Vec<DrugInformation>> {
		let sql = format!(
			"SELECT * FROM {} WHERE case_id = $1 ORDER BY sequence_number",
			Self::TABLE
		);
		let drugs = sqlx::query_as::<_, DrugInformation>(&sql)
			.bind(case_id)
			.fetch_all(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;
		Ok(drugs)
	}

	pub async fn get_in_case(
		_ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
		id: Uuid,
	) -> Result<DrugInformation> {
		let sql = format!(
			"SELECT * FROM {} WHERE id = $1 AND case_id = $2",
			Self::TABLE
		);
		let drug = sqlx::query_as::<_, DrugInformation>(&sql)
			.bind(id)
			.bind(case_id)
			.fetch_optional(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?
			.ok_or(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			})?;
		Ok(drug)
	}

	pub async fn update_in_case(
		ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
		id: Uuid,
		drug_u: DrugInformationForUpdate,
	) -> Result<()> {
		let db = mm.dbx().db();
		let mut tx = db.begin().await.map_err(|e| dbx::Error::from(e))?;
		set_user_context(&mut tx, ctx.user_id()).await?;

		let sql = format!(
			"UPDATE {}
			 SET medicinal_product = COALESCE($3, medicinal_product),
			     drug_characterization = COALESCE($4, drug_characterization),
			     brand_name = COALESCE($5, brand_name),
			     manufacturer_name = COALESCE($6, manufacturer_name),
			     batch_lot_number = COALESCE($7, batch_lot_number),
			     action_taken = COALESCE($8, action_taken),
			     updated_at = now(),
			     updated_by = $9
			 WHERE id = $1 AND case_id = $2",
			Self::TABLE
		);
		let result = sqlx::query(&sql)
			.bind(id)
			.bind(case_id)
			.bind(drug_u.medicinal_product)
			.bind(drug_u.drug_characterization)
			.bind(drug_u.brand_name)
			.bind(drug_u.manufacturer_name)
			.bind(drug_u.batch_lot_number)
			.bind(drug_u.action_taken)
			.bind(ctx.user_id())
			.execute(&mut *tx)
			.await
			.map_err(|e| dbx::Error::from(e))?;
		if result.rows_affected() == 0 {
			return Err(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			});
		}
		tx.commit().await.map_err(|e| dbx::Error::from(e))?;
		Ok(())
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		let db = mm.dbx().db();
		let mut tx = db.begin().await.map_err(|e| dbx::Error::from(e))?;
		set_user_context(&mut tx, ctx.user_id()).await?;

		let sql = format!("DELETE FROM {} WHERE id = $1", Self::TABLE);
		let result = sqlx::query(&sql)
			.bind(id)
			.execute(&mut *tx)
			.await
			.map_err(|e| dbx::Error::from(e))?;
		if result.rows_affected() == 0 {
			return Err(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			});
		}
		tx.commit().await.map_err(|e| dbx::Error::from(e))?;
		Ok(())
	}

	pub async fn delete_in_case(
		ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
		id: Uuid,
	) -> Result<()> {
		let db = mm.dbx().db();
		let mut tx = db.begin().await.map_err(|e| dbx::Error::from(e))?;
		set_user_context(&mut tx, ctx.user_id()).await?;

		let sql = format!(
			"DELETE FROM {} WHERE id = $1 AND case_id = $2",
			Self::TABLE
		);
		let result = sqlx::query(&sql)
			.bind(id)
			.bind(case_id)
			.execute(&mut *tx)
			.await
			.map_err(|e| dbx::Error::from(e))?;
		if result.rows_affected() == 0 {
			return Err(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			});
		}
		tx.commit().await.map_err(|e| dbx::Error::from(e))?;
		Ok(())
	}
}

pub struct DrugActiveSubstanceBmc;
impl DbBmc for DrugActiveSubstanceBmc {
	const TABLE: &'static str = "drug_active_substances";
}

impl DrugActiveSubstanceBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		data: DrugActiveSubstanceForCreate,
	) -> Result<Uuid> {
		base_uuid::create::<Self, _>(ctx, mm, data).await
	}

	pub async fn get(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
	) -> Result<DrugActiveSubstance> {
		base_uuid::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filters: Option<Vec<DrugActiveSubstanceFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<DrugActiveSubstance>> {
		base_uuid::list::<Self, _, _>(ctx, mm, filters, list_options).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		data: DrugActiveSubstanceForUpdate,
	) -> Result<()> {
		base_uuid::update::<Self, _>(ctx, mm, id, data).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		base_uuid::delete::<Self>(ctx, mm, id).await
	}
}

pub struct DosageInformationBmc;
impl DbBmc for DosageInformationBmc {
	const TABLE: &'static str = "dosage_information";
}

impl DosageInformationBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		data: DosageInformationForCreate,
	) -> Result<Uuid> {
		base_uuid::create::<Self, _>(ctx, mm, data).await
	}

	pub async fn get(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
	) -> Result<DosageInformation> {
		base_uuid::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filters: Option<Vec<DosageInformationFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<DosageInformation>> {
		base_uuid::list::<Self, _, _>(ctx, mm, filters, list_options).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		data: DosageInformationForUpdate,
	) -> Result<()> {
		base_uuid::update::<Self, _>(ctx, mm, id, data).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		base_uuid::delete::<Self>(ctx, mm, id).await
	}
}

pub struct DrugIndicationBmc;
impl DbBmc for DrugIndicationBmc {
	const TABLE: &'static str = "drug_indications";
}

impl DrugIndicationBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		data: DrugIndicationForCreate,
	) -> Result<Uuid> {
		base_uuid::create::<Self, _>(ctx, mm, data).await
	}

	pub async fn get(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
	) -> Result<DrugIndication> {
		base_uuid::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filters: Option<Vec<DrugIndicationFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<DrugIndication>> {
		base_uuid::list::<Self, _, _>(ctx, mm, filters, list_options).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		data: DrugIndicationForUpdate,
	) -> Result<()> {
		base_uuid::update::<Self, _>(ctx, mm, id, data).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		base_uuid::delete::<Self>(ctx, mm, id).await
	}
}
