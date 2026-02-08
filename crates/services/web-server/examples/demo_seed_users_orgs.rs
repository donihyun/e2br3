#![allow(unused)] // Example convenience script.

//! Demo seed script: organizations + users (for empty DB).
//!
//! Creates demo orgs/users with stable IDs and sets passwords.

use lib_core::ctx::{Ctx, ROLE_ADMIN, SYSTEM_ORG_ID, SYSTEM_USER_ID};
use lib_core::model::store::set_full_context_dbx;
use lib_core::model::user::UserBmc;
use lib_core::model::ModelManager;
use sqlx::query;
use uuid::Uuid;

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

const DEMO_ORG_ID: &str = "00000000-0000-0000-0000-000000000001";
const PHARMA_ORG_ID: &str = "00000000-0000-0000-0000-000000000002";
const CRO_ORG_ID: &str = "00000000-0000-0000-0000-000000000003";

const DEMO_USER_ID: &str = "11111111-1111-1111-1111-111111111111";
const MANAGER_USER_ID: &str = "11111111-1111-1111-1111-111111111112";
const USER_USER_ID: &str = "11111111-1111-1111-1111-111111111113";
const VIEWER_USER_ID: &str = "11111111-1111-1111-1111-111111111114";

const DEMO_PWD: &str = "welcome";

#[tokio::main]
async fn main() -> Result<()> {
	println!("== Demo Seed: Organizations + Users ==");

	let mm = ModelManager::new().await?;
	let dbx = mm.dbx();
	let system_user_id = Uuid::parse_str(SYSTEM_USER_ID)?;
	let system_org_id = Uuid::parse_str(SYSTEM_ORG_ID)?;

	// Transaction to keep context consistent for audit/RLS.
	dbx.begin_txn().await?;
	set_full_context_dbx(dbx, system_user_id, system_org_id, ROLE_ADMIN).await?;

	seed_orgs(&mm, system_user_id).await?;
	seed_users(&mm, system_user_id).await?;

	dbx.commit_txn().await?;

	// Set passwords (idempotent, uses existing IDs by email).
	let root_ctx = Ctx::root_ctx();
	set_pwd(&root_ctx, &mm, "demo.user@example.com", DEMO_PWD).await?;
	set_pwd(&root_ctx, &mm, "manager.user@example.com", DEMO_PWD).await?;
	set_pwd(&root_ctx, &mm, "regular.user@example.com", DEMO_PWD).await?;
	set_pwd(&root_ctx, &mm, "viewer.user@example.com", DEMO_PWD).await?;

	println!("== Seed complete ==");
	Ok(())
}

async fn seed_orgs(mm: &ModelManager, created_by: Uuid) -> Result<()> {
	insert_org(
		mm,
		DEMO_ORG_ID,
		"Demo Organization",
		"internal",
		"123 Demo St",
		"Metropolis",
		"CA",
		"12345",
		"US",
		"demo@example.com",
		"555-1234",
		created_by,
	)
	.await?;

	insert_org(
		mm,
		PHARMA_ORG_ID,
		"Acme Pharma",
		"pharma",
		"9 Market St",
		"Boston",
		"MA",
		"02110",
		"US",
		"pv@acmepharma.example",
		"617-555-0101",
		created_by,
	)
	.await?;

	insert_org(
		mm,
		CRO_ORG_ID,
		"Northwind CRO",
		"cro",
		"20 Research Way",
		"San Diego",
		"CA",
		"92121",
		"US",
		"contact@northwindcro.example",
		"858-555-0110",
		created_by,
	)
	.await?;

	Ok(())
}

async fn seed_users(mm: &ModelManager, created_by: Uuid) -> Result<()> {
	insert_user(
		mm,
		DEMO_USER_ID,
		DEMO_ORG_ID,
		"demo.user@example.com",
		"demo_user",
		"admin",
		"Demo",
		"Admin",
		created_by,
	)
	.await?;

	insert_user(
		mm,
		MANAGER_USER_ID,
		PHARMA_ORG_ID,
		"manager.user@example.com",
		"manager_user",
		"manager",
		"Manny",
		"Manager",
		created_by,
	)
	.await?;

	insert_user(
		mm,
		USER_USER_ID,
		PHARMA_ORG_ID,
		"regular.user@example.com",
		"regular_user",
		"user",
		"Riley",
		"User",
		created_by,
	)
	.await?;

	insert_user(
		mm,
		VIEWER_USER_ID,
		CRO_ORG_ID,
		"viewer.user@example.com",
		"viewer_user",
		"viewer",
		"Vera",
		"Viewer",
		created_by,
	)
	.await?;

	Ok(())
}

async fn insert_org(
	mm: &ModelManager,
	id: &str,
	name: &str,
	org_type: &str,
	address: &str,
	city: &str,
	state: &str,
	postcode: &str,
	country_code: &str,
	contact_email: &str,
	contact_phone: &str,
	created_by: Uuid,
) -> Result<()> {
	let id = Uuid::parse_str(id)?;
	let dbx = mm.dbx();
	let q = query(
		"INSERT INTO organizations (id, name, org_type, address, city, state, postcode, country_code, contact_email, contact_phone, active, created_by, created_at, updated_at) \
		 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, true, $11, NOW(), NOW()) \
		 ON CONFLICT (id) DO NOTHING",
	)
	.bind(id)
	.bind(name)
	.bind(org_type)
	.bind(address)
	.bind(city)
	.bind(state)
	.bind(postcode)
	.bind(country_code)
	.bind(contact_email)
	.bind(contact_phone)
	.bind(created_by);

	dbx.execute(q).await?;
	println!("Inserted org: {name} ({id})");
	Ok(())
}

async fn insert_user(
	mm: &ModelManager,
	id: &str,
	org_id: &str,
	email: &str,
	username: &str,
	role: &str,
	first_name: &str,
	last_name: &str,
	created_by: Uuid,
) -> Result<()> {
	let id = Uuid::parse_str(id)?;
	let org_id = Uuid::parse_str(org_id)?;
	let dbx = mm.dbx();
	let q = query(
		"INSERT INTO users (id, organization_id, email, username, role, first_name, last_name, active, created_by, created_at, updated_at) \
		 VALUES ($1, $2, $3, $4, $5, $6, $7, true, $8, NOW(), NOW()) \
		 ON CONFLICT (id) DO NOTHING",
	)
	.bind(id)
	.bind(org_id)
	.bind(email)
	.bind(username)
	.bind(role)
	.bind(first_name)
	.bind(last_name)
	.bind(created_by);

	dbx.execute(q).await?;
	println!("Inserted user: {email} ({role})");
	Ok(())
}

async fn set_pwd(ctx: &Ctx, mm: &ModelManager, email: &str, pwd: &str) -> Result<()> {
	// Use auth_email-based lookup to bypass RLS when no org context is set.
	let user = UserBmc::auth_login_by_email(mm, email).await?;
	if let Some(user) = user {
		UserBmc::update_pwd(ctx, mm, user.id, pwd).await?;
		println!("Set password for {email}");
	} else {
		println!("Skipping password set; user not found: {email}");
	}
	Ok(())
}
