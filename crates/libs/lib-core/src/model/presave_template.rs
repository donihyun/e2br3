use crate::ctx::Ctx;
use crate::model::base::DbBmc;
use crate::model::store::set_full_context_dbx;
use crate::model::ModelManager;
use crate::model::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::types::time::OffsetDateTime;
use sqlx::types::Uuid;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct PresaveTemplate {
	pub id: Uuid,
	pub organization_id: Uuid,
	pub entity_type: String,
	pub name: String,
	pub description: Option<String>,
	pub data: JsonValue,
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
	pub created_by: Uuid,
	pub updated_by: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct PresaveTemplateForCreate {
	pub entity_type: String,
	pub name: String,
	pub description: Option<String>,
	pub data: JsonValue,
}

#[derive(Deserialize)]
pub struct PresaveTemplateForUpdate {
	pub entity_type: Option<String>,
	pub name: Option<String>,
	pub description: Option<String>,
	pub data: Option<JsonValue>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct PresaveTemplateAudit {
	pub id: i64,
	pub template_id: Uuid,
	pub organization_id: Uuid,
	pub action: String,
	pub changed_by: Uuid,
	pub old_values: Option<JsonValue>,
	pub new_values: Option<JsonValue>,
	pub created_at: OffsetDateTime,
}

pub struct PresaveTemplateBmc;
impl DbBmc for PresaveTemplateBmc {
	const TABLE: &'static str = "presave_templates";
}

impl PresaveTemplateBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		template_c: PresaveTemplateForCreate,
	) -> Result<Uuid> {
		let dbx = mm.dbx();
		dbx.begin_txn().await?;
		if let Err(err) = set_full_context_dbx(
			dbx,
			ctx.user_id(),
			ctx.organization_id(),
			ctx.role(),
		)
		.await
		{
			dbx.rollback_txn().await?;
			return Err(err);
		}

		let sql = "INSERT INTO presave_templates (organization_id, entity_type, name, description, data, created_by, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW()) RETURNING id";
		let res = dbx
			.fetch_one(
				sqlx::query_as::<_, (Uuid,)>(sql)
					.bind(ctx.organization_id())
					.bind(template_c.entity_type)
					.bind(template_c.name)
					.bind(template_c.description)
					.bind(template_c.data)
					.bind(ctx.user_id()),
			)
			.await;

		let (id,) = match res {
			Ok(v) => v,
			Err(err) => {
				dbx.rollback_txn().await?;
				return Err(err.into());
			}
		};
		dbx.commit_txn().await?;
		Ok(id)
	}

	pub async fn get(
		_ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
	) -> Result<PresaveTemplate> {
		let sql = format!("SELECT * FROM {} WHERE id = $1", Self::TABLE);
		let entity = mm
			.dbx()
			.fetch_optional(sqlx::query_as::<_, PresaveTemplate>(&sql).bind(id))
			.await?
			.ok_or(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			})?;
		Ok(entity)
	}

	pub async fn list(
		_ctx: &Ctx,
		mm: &ModelManager,
	) -> Result<Vec<PresaveTemplate>> {
		let sql = format!(
			"SELECT * FROM {} ORDER BY updated_at DESC LIMIT 1000",
			Self::TABLE
		);
		let rows = mm
			.dbx()
			.fetch_all(sqlx::query_as::<_, PresaveTemplate>(&sql))
			.await?;
		Ok(rows)
	}

	pub async fn list_by_entity_type(
		_ctx: &Ctx,
		mm: &ModelManager,
		entity_type: &str,
	) -> Result<Vec<PresaveTemplate>> {
		let sql = format!(
			"SELECT * FROM {} WHERE entity_type = $1 ORDER BY updated_at DESC",
			Self::TABLE
		);
		let rows = mm
			.dbx()
			.fetch_all(sqlx::query_as::<_, PresaveTemplate>(&sql).bind(entity_type))
			.await?;
		Ok(rows)
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		template_u: PresaveTemplateForUpdate,
	) -> Result<()> {
		let dbx = mm.dbx();
		dbx.begin_txn().await?;
		if let Err(err) = set_full_context_dbx(
			dbx,
			ctx.user_id(),
			ctx.organization_id(),
			ctx.role(),
		)
		.await
		{
			dbx.rollback_txn().await?;
			return Err(err);
		}

		let sql = format!(
			"UPDATE {} \
			 SET entity_type = COALESCE($2, entity_type), \
			 name = COALESCE($3, name), \
			 description = COALESCE($4, description), \
			 data = COALESCE($5, data), \
			 updated_by = $6, \
			 updated_at = NOW() \
			 WHERE id = $1",
			Self::TABLE
		);

		let count = match dbx
			.execute(
				sqlx::query(&sql)
					.bind(id)
					.bind(template_u.entity_type)
					.bind(template_u.name)
					.bind(template_u.description)
					.bind(template_u.data)
					.bind(ctx.user_id()),
			)
			.await
		{
			Ok(v) => v,
			Err(err) => {
				dbx.rollback_txn().await?;
				return Err(err.into());
			}
		};

		if count == 0 {
			dbx.rollback_txn().await?;
			return Err(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			});
		}

		dbx.commit_txn().await?;
		Ok(())
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		let dbx = mm.dbx();
		dbx.begin_txn().await?;
		if let Err(err) = set_full_context_dbx(
			dbx,
			ctx.user_id(),
			ctx.organization_id(),
			ctx.role(),
		)
		.await
		{
			dbx.rollback_txn().await?;
			return Err(err);
		}

		let sql = format!("DELETE FROM {} WHERE id = $1", Self::TABLE);
		let count = match dbx.execute(sqlx::query(&sql).bind(id)).await {
			Ok(v) => v,
			Err(err) => {
				dbx.rollback_txn().await?;
				return Err(err.into());
			}
		};

		if count == 0 {
			dbx.rollback_txn().await?;
			return Err(crate::model::Error::EntityUuidNotFound {
				entity: Self::TABLE,
				id,
			});
		}

		dbx.commit_txn().await?;
		Ok(())
	}
}

pub struct PresaveTemplateAuditBmc;
impl DbBmc for PresaveTemplateAuditBmc {
	const TABLE: &'static str = "presave_template_audits";

	fn has_timestamps() -> bool {
		false
	}
}

impl PresaveTemplateAuditBmc {
	pub async fn list_by_template(
		_ctx: &Ctx,
		mm: &ModelManager,
		template_id: Uuid,
	) -> Result<Vec<PresaveTemplateAudit>> {
		let sql = format!(
			"SELECT * FROM {} WHERE template_id = $1 ORDER BY created_at DESC",
			Self::TABLE
		);
		let rows = mm
			.dbx()
			.fetch_all(
				sqlx::query_as::<_, PresaveTemplateAudit>(&sql).bind(template_id),
			)
			.await?;
		Ok(rows)
	}
}
