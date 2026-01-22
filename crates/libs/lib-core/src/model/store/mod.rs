// region:    --- Modules

pub(in crate::model) mod dbx;

use crate::core_config;
use crate::model::Error;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres, Transaction};
use std::env;
use uuid::Uuid;

// endregion: --- Modules

pub type Db = Pool<Postgres>;

pub async fn new_db_pool() -> sqlx::Result<Db> {
	// * See NOTE 1) below
	let max_connections = if cfg!(test) { 1 } else { 5 };

	PgPoolOptions::new()
		.max_connections(max_connections)
		.after_connect(|conn, _meta| {
			Box::pin(async move {
				if let Ok(user_id) = env::var("E2BR3_TEST_CURRENT_USER_ID") {
					sqlx::query("SELECT set_config('app.current_user_id', $1, false)")
						.bind(user_id)
						.execute(&mut *conn)
						.await?;
				}
				if let Ok(role) = env::var("E2BR3_DB_ROLE") {
					if !role.is_empty() && is_safe_role_name(&role) {
						let sql = format!("SET ROLE {}", role);
						if let Err(err) =
							sqlx::query(&sql).execute(&mut *conn).await
						{
							if should_ignore_role_set_error(&err) {
								println!(
									"db warning: skipping SET ROLE due to missing/permission error: {}",
									role
								);
							} else {
								return Err(err);
							}
						}
					}
				}
				Ok(())
			})
		})
		.connect(&core_config().DB_URL)
		.await
}

// region:    --- User Context Helpers

/// Sets the current user context for PostgreSQL session.
/// This enables audit triggers to capture user_id for regulatory compliance.
///
/// CRITICAL: This must be called at the start of every transaction that performs
/// INSERT, UPDATE, or DELETE operations to ensure audit trail compliance with
/// 21 CFR Part 11, EMA GVP Module VI, and ALCOA+ principles.
pub async fn set_user_context(
	tx: &mut Transaction<'_, Postgres>,
	user_id: Uuid,
) -> Result<(), Error> {
	sqlx::query("SELECT set_current_user_context($1)")
		.bind(user_id)
		.execute(&mut **tx)
		.await
		.map_err(|e| Error::Store(format!("Failed to set user context: {}", e)))?;

	Ok(())
}

/// Sets the current user context using Dbx (respects existing transactions).
pub async fn set_user_context_dbx(
	dbx: &dbx::Dbx,
	user_id: Uuid,
) -> Result<(), Error> {
	let query = sqlx::query("SELECT set_current_user_context($1)").bind(user_id);
	dbx.execute(query)
		.await
		.map_err(|e| Error::Store(format!("Failed to set user context: {}", e)))?;

	Ok(())
}

/// Gets the current user context from PostgreSQL session.
/// Used for verification and debugging purposes.
#[allow(dead_code)]
pub async fn get_user_context(
	tx: &mut Transaction<'_, Postgres>,
) -> Result<Uuid, Error> {
	let row: (Uuid,) = sqlx::query_as("SELECT get_current_user_context()")
		.fetch_one(&mut **tx)
		.await
		.map_err(|e| Error::Store(format!("Failed to get user context: {}", e)))?;

	Ok(row.0)
}

// endregion: --- User Context Helpers

fn is_safe_role_name(role: &str) -> bool {
	role.chars()
		.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn should_ignore_role_set_error(err: &sqlx::Error) -> bool {
	let strict = env::var("E2BR3_DB_ROLE_STRICT")
		.map(|v| v == "1")
		.unwrap_or(false);
	if strict {
		return false;
	}

	match err {
		sqlx::Error::Database(db_err) => matches!(
			db_err.code().as_deref(),
			Some("42704") | Some("42501")
		),
		_ => false,
	}
}

// NOTE 1) This is not an ideal situation; however, with sqlx 0.7.1, when executing `cargo test`, some tests that use sqlx fail at a
//         rather low level (in the tokio scheduler). It appears to be a low-level thread/async issue, as removing/adding
//         tests causes different tests to fail. The cause remains uncertain, but setting max_connections to 1 resolves the issue.
//         The good news is that max_connections still function normally for a regular run.
//         This issue is likely due to the unique requirements unit tests impose on their execution, and therefore,
//         while not ideal, it should serve as an acceptable temporary solution.
//         It's a very challenging issue to investigate and narrow down. The alternative would have been to stick with sqlx 0.6.x, which
//         is potentially less ideal and might lead to confusion as to why we are maintaining the older version in this blueprint.
