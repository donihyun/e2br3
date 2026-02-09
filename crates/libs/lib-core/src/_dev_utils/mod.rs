// region:    --- Modules

mod dev_db;

use crate::ctx::Ctx;
use crate::model::user::UserBmc;
use crate::model::ModelManager;
use tokio::sync::OnceCell;
use tracing::{info, warn};

// endregion: --- Modules

/// Initialize environment for local development.
/// (for early development, will be called from main()).
/// Skip if SKIP_DEV_INIT=1 (e.g., when using Docker PostgreSQL).
pub async fn init_dev() {
	// Skip if using Docker PostgreSQL (already initialized)
	if std::env::var("SKIP_DEV_INIT").unwrap_or_default() == "1" {
		info!(
			"{:<12} - init_dev() SKIPPED (SKIP_DEV_INIT=1)",
			"FOR-DEV-ONLY"
		);
		maybe_set_demo_pwd().await;
		return;
	}

	static INIT: OnceCell<()> = OnceCell::const_new();

	INIT.get_or_init(|| async {
		info!("{:<12} - init_dev_all()", "FOR-DEV-ONLY");

		dev_db::init_dev_db().await.unwrap();
		maybe_set_demo_pwd().await;
	})
	.await;
}

async fn maybe_set_demo_pwd() {
	let pwd = match std::env::var("DEMO_USER_PWD") {
		Ok(value) if !value.trim().is_empty() => value,
		_ => return,
	};
	let email = std::env::var("DEMO_USER_EMAIL")
		.unwrap_or_else(|_| "demo.user@example.com".to_string());

	let mm = match ModelManager::new().await {
		Ok(mm) => mm,
		Err(err) => {
			warn!("FOR-DEV-ONLY - demo pwd skipped; db init failed: {err}");
			return;
		}
	};

	let ctx = Ctx::root_ctx();
	// Use auth_email-based lookup to bypass RLS when no org context is set.
	let user = match UserBmc::auth_login_by_email(&mm, &email).await {
		Ok(user) => user,
		Err(err) => {
			warn!("FOR-DEV-ONLY - demo pwd lookup failed: {err}");
			return;
		}
	};

	let Some(user) = user else {
		warn!("FOR-DEV-ONLY - demo pwd skipped; user not found: {email}");
		return;
	};

	if user.pwd.is_some() {
		return;
	}

	if let Err(err) = UserBmc::update_pwd(&ctx, &mm, user.id, &pwd).await {
		warn!("FOR-DEV-ONLY - demo pwd update failed: {err}");
		return;
	}

	info!("FOR-DEV-ONLY - demo pwd set for {email}");
}

/// Initialize test environment.
pub async fn init_test() -> ModelManager {
	static INIT: OnceCell<ModelManager> = OnceCell::const_new();

	let mm = INIT
		.get_or_init(|| async {
			init_dev().await;
			// NOTE: Rare occasion where unwrap is kind of ok.
			ModelManager::new().await.unwrap()
		})
		.await;

	mm.clone()
}

// NOTE: Test seed/clean helpers for user/agent/conv have been removed
// as those models were replaced with the E2B(R3) SafetyDB models.
// Add new test helpers here as needed for User, Organization, Case, etc.

pub fn fx_org_id() -> uuid::Uuid {
	// NOTE: This org_id is created via sql/dev_initial/00-recreate-db.sql
	uuid::uuid!("00000000-0000-0000-0000-000000000001")
}
