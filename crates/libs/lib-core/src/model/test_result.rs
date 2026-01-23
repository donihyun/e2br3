// Section F - Tests and Procedures

use crate::ctx::Ctx;
use crate::model::base::DbBmc;
use crate::model::store::{dbx, set_user_context};
use crate::model::ModelManager;
use crate::model::Result;
use modql::field::Fields;
use serde::{Deserialize, Serialize};
use sqlx::types::time::{Date, OffsetDateTime};
use sqlx::types::Uuid;
use sqlx::FromRow;

// -- TestResult

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct TestResult {
	pub id: Uuid,
	pub case_id: Uuid,
	pub sequence_number: i32,

	// F.r.1 - Test Date
	pub test_date: Option<Date>,

	// F.r.2 - Test Name
	pub test_name: String,

	// F.r.2.1 - Test Name (MedDRA coded)
	pub test_meddra_version: Option<String>,
	pub test_meddra_code: Option<String>,

	// F.r.3.1 - Test Result (coded)
	pub test_result_code: Option<String>,

	// F.r.3.2 - Test Result (value/finding)
	pub test_result_value: Option<String>,

	// F.r.3.3 - Test Result Unit
	pub test_result_unit: Option<String>,

	// F.r.3.4 - Result Unstructured Data
	pub result_unstructured: Option<String>,

	// F.r.4-5 - Normal Range
	pub normal_low_value: Option<String>,
	pub normal_high_value: Option<String>,

	// F.r.6 - Comments
	pub comments: Option<String>,

	// F.r.7 - More Information Available
	pub more_info_available: Option<bool>,

	// Timestamps
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
	pub created_by: Uuid,
	pub updated_by: Option<Uuid>,
}

#[derive(Fields, Deserialize)]
pub struct TestResultForCreate {
	pub case_id: Uuid,
	pub sequence_number: i32,
	pub test_name: String,
}

#[derive(Fields, Deserialize)]
pub struct TestResultForUpdate {
	pub test_name: Option<String>,
	pub test_date: Option<Date>,
	pub test_result_value: Option<String>,
	pub test_result_unit: Option<String>,
	pub normal_low_value: Option<String>,
	pub normal_high_value: Option<String>,
	pub comments: Option<String>,
}

// -- BMC

pub struct TestResultBmc;
impl DbBmc for TestResultBmc {
	const TABLE: &'static str = "test_results";
}

impl TestResultBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		test_c: TestResultForCreate,
	) -> Result<Uuid> {
		let db = mm.dbx().db();
		let mut tx = db.begin().await.map_err(dbx::Error::from)?;
		set_user_context(&mut tx, ctx.user_id()).await?;

		let sql = format!(
			"INSERT INTO {} (case_id, sequence_number, test_name, created_at, updated_at, created_by)
			 VALUES ($1, $2, $3, now(), now(), $4)
			 RETURNING id",
			Self::TABLE
		);
		let id: Uuid = sqlx::query_scalar(&sql)
			.bind(test_c.case_id)
			.bind(test_c.sequence_number)
			.bind(test_c.test_name)
			.bind(ctx.user_id())
			.fetch_one(&mut *tx)
			.await
			.map_err(dbx::Error::from)?;

		tx.commit().await.map_err(dbx::Error::from)?;
		Ok(id)
	}

	pub async fn get(_ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<TestResult> {
		let sql = format!("SELECT * FROM {} WHERE id = $1", Self::TABLE);
		let test = sqlx::query_as::<_, TestResult>(&sql)
			.bind(id)
			.fetch_optional(mm.dbx().db())
			.await
			.map_err(dbx::Error::from)?
			.ok_or(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			})?;
		Ok(test)
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		test_u: TestResultForUpdate,
	) -> Result<()> {
		let db = mm.dbx().db();
		let mut tx = db.begin().await.map_err(dbx::Error::from)?;
		set_user_context(&mut tx, ctx.user_id()).await?;

		let sql = format!(
			"UPDATE {}
			 SET test_name = COALESCE($2, test_name),
			     test_date = COALESCE($3, test_date),
			     test_result_value = COALESCE($4, test_result_value),
			     test_result_unit = COALESCE($5, test_result_unit),
			     normal_low_value = COALESCE($6, normal_low_value),
			     normal_high_value = COALESCE($7, normal_high_value),
			     comments = COALESCE($8, comments),
			     updated_at = now(),
			     updated_by = $9
			 WHERE id = $1",
			Self::TABLE
		);
		let result = sqlx::query(&sql)
			.bind(id)
			.bind(test_u.test_name)
			.bind(test_u.test_date)
			.bind(test_u.test_result_value)
			.bind(test_u.test_result_unit)
			.bind(test_u.normal_low_value)
			.bind(test_u.normal_high_value)
			.bind(test_u.comments)
			.bind(ctx.user_id())
			.execute(&mut *tx)
			.await
			.map_err(dbx::Error::from)?;
		if result.rows_affected() == 0 {
			return Err(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			});
		}
		tx.commit().await.map_err(dbx::Error::from)?;
		Ok(())
	}

	pub async fn list_by_case(
		_ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
	) -> Result<Vec<TestResult>> {
		let sql = format!(
			"SELECT * FROM {} WHERE case_id = $1 ORDER BY sequence_number",
			Self::TABLE
		);
		let tests = sqlx::query_as::<_, TestResult>(&sql)
			.bind(case_id)
			.fetch_all(mm.dbx().db())
			.await
			.map_err(dbx::Error::from)?;
		Ok(tests)
	}

	pub async fn get_in_case(
		_ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
		id: Uuid,
	) -> Result<TestResult> {
		let sql = format!(
			"SELECT * FROM {} WHERE id = $1 AND case_id = $2",
			Self::TABLE
		);
		let test = sqlx::query_as::<_, TestResult>(&sql)
			.bind(id)
			.bind(case_id)
			.fetch_optional(mm.dbx().db())
			.await
			.map_err(dbx::Error::from)?
			.ok_or(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			})?;
		Ok(test)
	}

	pub async fn update_in_case(
		ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
		id: Uuid,
		test_u: TestResultForUpdate,
	) -> Result<()> {
		let db = mm.dbx().db();
		let mut tx = db.begin().await.map_err(dbx::Error::from)?;
		set_user_context(&mut tx, ctx.user_id()).await?;

		let sql = format!(
			"UPDATE {}
			 SET test_name = COALESCE($3, test_name),
			     test_date = COALESCE($4, test_date),
			     test_result_value = COALESCE($5, test_result_value),
			     test_result_unit = COALESCE($6, test_result_unit),
			     normal_low_value = COALESCE($7, normal_low_value),
			     normal_high_value = COALESCE($8, normal_high_value),
			     comments = COALESCE($9, comments),
			     updated_at = now(),
			     updated_by = $10
			 WHERE id = $1 AND case_id = $2",
			Self::TABLE
		);
		let result = sqlx::query(&sql)
			.bind(id)
			.bind(case_id)
			.bind(test_u.test_name)
			.bind(test_u.test_date)
			.bind(test_u.test_result_value)
			.bind(test_u.test_result_unit)
			.bind(test_u.normal_low_value)
			.bind(test_u.normal_high_value)
			.bind(test_u.comments)
			.bind(ctx.user_id())
			.execute(&mut *tx)
			.await
			.map_err(dbx::Error::from)?;
		if result.rows_affected() == 0 {
			return Err(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			});
		}
		tx.commit().await.map_err(dbx::Error::from)?;
		Ok(())
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		let db = mm.dbx().db();
		let mut tx = db.begin().await.map_err(dbx::Error::from)?;
		set_user_context(&mut tx, ctx.user_id()).await?;

		let sql = format!("DELETE FROM {} WHERE id = $1", Self::TABLE);
		let result = sqlx::query(&sql)
			.bind(id)
			.execute(&mut *tx)
			.await
			.map_err(dbx::Error::from)?;
		if result.rows_affected() == 0 {
			return Err(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			});
		}
		tx.commit().await.map_err(dbx::Error::from)?;
		Ok(())
	}

	pub async fn delete_in_case(
		ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
		id: Uuid,
	) -> Result<()> {
		let db = mm.dbx().db();
		let mut tx = db.begin().await.map_err(dbx::Error::from)?;
		set_user_context(&mut tx, ctx.user_id()).await?;

		let sql =
			format!("DELETE FROM {} WHERE id = $1 AND case_id = $2", Self::TABLE);
		let result = sqlx::query(&sql)
			.bind(id)
			.bind(case_id)
			.execute(&mut *tx)
			.await
			.map_err(dbx::Error::from)?;
		if result.rows_affected() == 0 {
			return Err(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			});
		}
		tx.commit().await.map_err(dbx::Error::from)?;
		Ok(())
	}
}
