//! Base Model Layer (UUID variant)
//!
//! This module provides CRUD operations for models that use UUID primary keys
//! instead of i64. It mirrors the functionality of the base module but adapted
//! for UUID-based E2B(R3) models.

use crate::ctx::Ctx;
use crate::model::base::DbBmc;
use crate::model::store::dbx;
use crate::model::ModelManager;
use crate::model::Result;
use modql::field::HasSeaFields;
use sea_query::{Expr, Iden, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::postgres::PgRow;
use sqlx::types::Uuid;
use sqlx::FromRow;

const LIST_LIMIT_DEFAULT: i64 = 1000;
const LIST_LIMIT_MAX: i64 = 5000;

// region:    --- Create

pub async fn create<MC, E>(_ctx: &Ctx, mm: &ModelManager, data: E) -> Result<Uuid>
where
	MC: DbBmc,
	E: HasSeaFields,
{
	let dbx = mm.dbx();

	// -- Build the SQL query
	let fields = data.not_none_sea_fields();
	let (columns, sea_values) = fields.for_sea_insert();

	let mut query = Query::insert();
	query
		.into_table(MC::table_ref())
		.columns(columns)
		.values(sea_values)?
		.returning(Query::returning().columns([CommonIden::Id]));

	// -- Execute the query
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	let (id,) = sqlx::query_as_with::<_, (Uuid,), _>(&sql, values)
		.fetch_one(dbx.db())
		.await
		.map_err(|ex| dbx::Error::from(ex))?;

	Ok(id)
}

// endregion: --- Create

// region:    --- Get

pub async fn get<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<E>
where
	MC: DbBmc,
	E: for<'r> FromRow<'r, PgRow> + Unpin + Send + HasSeaFields,
{
	let dbx = mm.dbx();

	// -- Build the SQL query
	let mut query = Query::select();
	query
		.from(MC::table_ref())
		.columns(E::sea_column_refs())
		.and_where(Expr::col(CommonIden::Id).eq(id));

	// -- Execute the query
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	let entity = sqlx::query_as_with::<_, E, _>(&sql, values)
		.fetch_optional(dbx.db())
		.await
		.map_err(|ex| dbx::Error::from(ex))?
		.ok_or(crate::model::Error::EntityNotFound {
			entity: MC::TABLE,
			id: 0, // Note: EntityNotFound uses i64, but we're using UUID
		})?;

	Ok(entity)
}

// endregion: --- Get

// region:    --- Update

pub async fn update<MC, E>(
	_ctx: &Ctx,
	mm: &ModelManager,
	id: Uuid,
	data: E,
) -> Result<()>
where
	MC: DbBmc,
	E: HasSeaFields,
{
	let dbx = mm.dbx();

	// -- Build the SQL query
	let fields = data.not_none_sea_fields();
	let fields = fields.for_sea_update();

	let mut query = Query::update();
	query
		.table(MC::table_ref())
		.values(fields)
		.and_where(Expr::col(CommonIden::Id).eq(id));

	// -- Execute the query
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	let count = sqlx::query_with(&sql, values)
		.execute(dbx.db())
		.await
		.map_err(|ex| dbx::Error::from(ex))?
		.rows_affected();

	// -- Check result
	if count == 0 {
		Err(crate::model::Error::EntityNotFound {
			entity: MC::TABLE,
			id: 0,
		})
	} else {
		Ok(())
	}
}

// endregion: --- Update

// region:    --- Delete

pub async fn delete<MC>(_ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()>
where
	MC: DbBmc,
{
	let dbx = mm.dbx();

	// -- Build the SQL query
	let mut query = Query::delete();
	query
		.from_table(MC::table_ref())
		.and_where(Expr::col(CommonIden::Id).eq(id));

	// -- Execute the query
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	let count = sqlx::query_with(&sql, values)
		.execute(dbx.db())
		.await
		.map_err(|ex| dbx::Error::from(ex))?
		.rows_affected();

	// -- Check result
	if count == 0 {
		Err(crate::model::Error::EntityNotFound {
			entity: MC::TABLE,
			id: 0,
		})
	} else {
		Ok(())
	}
}

// endregion: --- Delete

// region:    --- List

pub async fn list<MC, E, F>(
	_ctx: &Ctx,
	mm: &ModelManager,
	filters: Option<F>,
	list_options: Option<modql::filter::ListOptions>,
) -> Result<Vec<E>>
where
	MC: DbBmc,
	F: Into<modql::filter::FilterGroups>,
	E: for<'r> FromRow<'r, PgRow> + Unpin + Send + HasSeaFields,
{
	let dbx = mm.dbx();

	// -- Build the SQL query
	let mut query = Query::select();
	query.from(MC::table_ref()).columns(E::sea_column_refs());

	// condition from filter
	if let Some(filters) = filters {
		let filters: modql::filter::FilterGroups = filters.into();
		let cond: sea_query::Condition = filters.try_into()?;
		query.cond_where(cond);
	}

	// list options
	let list_options = compute_list_options(list_options)?;
	list_options.apply_to_sea_query(&mut query);

	// -- Execute the query
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	let entities = sqlx::query_as_with::<_, E, _>(&sql, values)
		.fetch_all(dbx.db())
		.await
		.map_err(|ex| dbx::Error::from(ex))?;

	Ok(entities)
}

fn compute_list_options(
	list_options: Option<modql::filter::ListOptions>,
) -> Result<modql::filter::ListOptions> {
	if let Some(mut list_options) = list_options {
		// Validate the limit
		if let Some(limit) = list_options.limit {
			if limit > LIST_LIMIT_MAX {
				return Err(crate::model::Error::ListLimitOverMax {
					max: LIST_LIMIT_MAX,
					actual: limit,
				});
			}
		}
		// Set default limit if absent
		else {
			list_options.limit = Some(LIST_LIMIT_DEFAULT);
		}
		Ok(list_options)
	} else {
		Ok(modql::filter::ListOptions {
			limit: Some(LIST_LIMIT_DEFAULT),
			offset: None,
			order_bys: Some("id".into()),
		})
	}
}

// endregion: --- List

// region:    --- CommonIden

#[derive(Iden)]
enum CommonIden {
	Id,
}

// endregion: --- CommonIden
