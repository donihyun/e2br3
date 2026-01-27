use crate::ctx::Ctx;
use crate::model::base::base_uuid;
use crate::model::base::{prep_fields_for_update, DbBmc};
use crate::model::store::set_full_context_dbx;
use crate::model::{Error, ModelManager, Result};
use lib_auth::pwd::{self, ContentToHash};
use modql::field::{Fields, HasSeaFields, SeaField, SeaFields};
use modql::filter::{FilterNodes, ListOptions, OpValsString, OpValsValue};
use sea_query::{Expr, Iden, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::types::time::OffsetDateTime;
use sqlx::types::Uuid;
use sqlx::{FromRow, query};

// -- Types

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct User {
	pub id: Uuid,
	pub organization_id: Uuid,
	pub email: String,
	pub username: String,

	// Auth fields (not serialized)
	#[serde(skip)]
	pub pwd: Option<String>,
	#[serde(skip)]
	pub pwd_salt: Uuid,
	#[serde(skip)]
	pub token_salt: Uuid,

	pub role: String,
	pub first_name: Option<String>,
	pub last_name: Option<String>,
	pub active: bool,
	pub last_login_at: Option<OffsetDateTime>,

	// Audit fields (standardized UUID-based)
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
	pub created_by: Option<Uuid>,
	pub updated_by: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct UserForCreate {
	pub organization_id: Uuid,
	pub email: String,
	pub username: String,
	pub pwd_clear: String,
	pub role: Option<String>,
	pub first_name: Option<String>,
	pub last_name: Option<String>,
}

#[derive(Fields)]
pub struct UserForInsert {
	pub organization_id: Uuid,
	pub email: String,
	pub username: String,
	pub role: Option<String>,
	pub first_name: Option<String>,
	pub last_name: Option<String>,
}

#[derive(Clone, FromRow, Fields, Debug)]
pub struct UserForLogin {
	pub id: Uuid,
	pub organization_id: Uuid,
	pub email: String,
	pub username: String,
	pub role: String,

	// -- pwd and token info
	pub pwd: Option<String>, // encrypted
	pub pwd_salt: Uuid,
	pub token_salt: Uuid,
}

#[derive(Clone, FromRow, Fields, Debug)]
pub struct UserForAuth {
	pub id: Uuid,
	pub organization_id: Uuid,
	pub email: String,
	pub username: String,
	pub role: String,

	// -- token info
	pub token_salt: Uuid,
}

#[derive(Fields, Deserialize)]
pub struct UserForUpdate {
	pub email: Option<String>,
	pub role: Option<String>,
	pub first_name: Option<String>,
	pub last_name: Option<String>,
	pub active: Option<bool>,
	pub last_login_at: Option<OffsetDateTime>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct UserFilter {
	pub organization_id: Option<OpValsValue>,
	pub email: Option<OpValsString>,
	pub username: Option<OpValsString>,
	pub role: Option<OpValsString>,
}

/// Marker trait for different User representations
pub trait UserBy: HasSeaFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl UserBy for User {}
impl UserBy for UserForLogin {}
impl UserBy for UserForAuth {}

#[derive(Iden)]
enum UserIden {
	Id,
	Email,
	Pwd,
}

// -- UserBmc

pub struct UserBmc;

impl DbBmc for UserBmc {
	const TABLE: &'static str = "users";
}

impl UserBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		user_c: UserForCreate,
	) -> Result<Uuid> {
		let UserForCreate {
			organization_id,
			email,
			username,
			pwd_clear,
			role,
			first_name,
			last_name,
		} = user_c;

		// -- Create the user row
		let user_fi = UserForInsert {
			organization_id,
			email: email.clone(),
			username,
			role,
			first_name,
			last_name,
		};

		// Start (or reuse) the transaction on the current dbx so request context is preserved.
		mm.dbx().begin_txn().await?;

		let user_id = match base_uuid::create::<Self, _>(ctx, mm, user_fi)
			.await
			.map_err(|model_error| {
				Error::resolve_unique_violation(
					model_error,
					Some(|table: &str, constraint: &str| {
						if table == "users" && constraint.contains("email") {
							Some(Error::UserAlreadyExists { email })
						} else {
							None
						}
					}),
				)
			}) {
			Ok(user_id) => user_id,
			Err(err) => {
				mm.dbx().rollback_txn().await?;
				return Err(err);
			}
		};

		// -- Update the password
		if let Err(err) = Self::update_pwd(ctx, mm, user_id, &pwd_clear).await {
			mm.dbx().rollback_txn().await?;
			return Err(err);
		}

		// Commit the transaction
		mm.dbx().commit_txn().await?;

		Ok(user_id)
	}

	pub async fn get<E>(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<E>
	where
		E: UserBy,
	{
		base_uuid::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filters: Option<Vec<UserFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<User>> {
		base_uuid::list::<Self, _, _>(ctx, mm, filters, list_options).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		user_u: UserForUpdate,
	) -> Result<()> {
		base_uuid::update::<Self, _>(ctx, mm, id, user_u).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		base_uuid::delete::<Self>(ctx, mm, id).await
	}

	pub async fn first_by_email<E>(
		_ctx: &Ctx,
		mm: &ModelManager,
		email: &str,
	) -> Result<Option<E>>
	where
		E: UserBy,
	{
		// -- Build query
		let mut query = Query::select();
		query
			.from(Self::table_ref())
			.columns(E::sea_idens())
			.and_where(Expr::col(UserIden::Email).eq(email));

		// -- Execute query
		let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

		let sqlx_query = sqlx::query_as_with::<_, E, _>(&sql, values);
		let entity = mm.dbx().fetch_optional(sqlx_query).await?;

		Ok(entity)
	}

	pub async fn update_pwd(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		pwd_clear: &str,
	) -> Result<()> {
		let dbx = mm.dbx();
		dbx.begin_txn().await.map_err(Error::Dbx)?;
		if let Err(err) =
			set_full_context_dbx(dbx, ctx.user_id(), ctx.organization_id(), ctx.role()).await
		{
			dbx.rollback_txn().await.map_err(Error::Dbx)?;
			return Err(err);
		}

		// -- Prep password
		let user: UserForLogin = Self::get(ctx, mm, id).await?;
		let pwd = pwd::hash_pwd(ContentToHash {
			content: pwd_clear.to_string(),
			salt: user.pwd_salt,
		})
		.await?;

		// -- Prep the data
		let mut fields = SeaFields::new(vec![SeaField::new(UserIden::Pwd, pwd)]);
		prep_fields_for_update::<Self>(&mut fields, ctx.user_id());

		// -- Build query
		let fields = fields.for_sea_update();
		let mut query = Query::update();
		query
			.table(Self::table_ref())
			.values(fields)
			.and_where(Expr::col(UserIden::Id).eq(id));

		// -- Exec query
		let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
		let sqlx_query = sqlx::query_with(&sql, values);
		if let Err(err) = dbx.execute(sqlx_query).await {
			dbx.rollback_txn().await.map_err(Error::Dbx)?;
			return Err(err.into());
		}

		dbx.commit_txn().await.map_err(Error::Dbx)?;

		Ok(())
	}

	pub async fn auth_by_email(
		mm: &ModelManager, 
		email: &str,
	) -> Result<Option<UserForAuth>> {
		let mm = mm.new_with_txn()?;
		mm.dbx().begin_txn().await.map_err(Error::Dbx)?;
		if let Err(err) = mm.dbx().execute(query("SELECT set_config('app.auth_email', $1, true)").bind(email)).await {
			mm.dbx().rollback_txn().await.map_err(Error::Dbx)?;
			return Err(err.into());
		}
		let user = match Self::first_by_email::<UserForAuth>(&Ctx::root_ctx(), &mm, email).await {
			Ok(user) => user,
			Err(err) => {
				mm.dbx().rollback_txn().await.map_err(Error::Dbx)?;
				return Err(err);
			}
		};
		mm.dbx().commit_txn().await.map_err(Error::Dbx)?;
		Ok(user)
	}

	pub async fn auth_login_by_email(
		mm: &ModelManager,
		email: &str,
	) -> Result<Option<UserForLogin>> {
		let mm = mm.new_with_txn()?;
		mm.dbx().begin_txn().await.map_err(Error::Dbx)?;
		if let Err(err) = mm
			.dbx()
			.execute(query("SELECT set_config('app.auth_email', $1, true)").bind(email))
			.await
		{
			mm.dbx().rollback_txn().await.map_err(Error::Dbx)?;
			return Err(err.into());
		}
		let user = match Self::first_by_email::<UserForLogin>(&Ctx::root_ctx(), &mm, email).await {
			Ok(user) => user,
			Err(err) => {
				mm.dbx().rollback_txn().await.map_err(Error::Dbx)?;
				return Err(err);
			}
		};
		mm.dbx().commit_txn().await.map_err(Error::Dbx)?;
		Ok(user)
	}
}

// Tests moved to crates/libs/lib-core/tests/model_crud.rs
