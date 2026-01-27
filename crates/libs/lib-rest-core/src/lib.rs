// region:    --- Modules

mod error;
pub mod rest_params;
pub mod rest_result;
mod utils;

pub use self::error::{Error, Result};
pub use rest_params::*;
pub use rest_result::*;

use lib_core::ctx::Ctx;
use lib_core::model::acs::{has_permission, Permission};

pub fn require_permission(ctx: &Ctx, permission: Permission) -> Result<()> {
	if !has_permission(ctx.role(), permission) {
		return Err(Error::PermissionDenied {
			required_permission: format!("{}", permission),
		});
	}
	Ok(())
}

pub mod prelude;

// endregion: --- Modules
