#![allow(unexpected_cfgs)]

// region:    --- Modules

mod error;

pub use self::error::{Error, Result};

// endregion: --- Modules

#[allow(unexpected_cfgs)]
#[cfg_attr(feature = "with-rpc", derive(rpc_router::RpcResource))]
#[derive(Clone, Debug)]
pub struct Ctx {
	user_id: uuid::Uuid,
}

// Constructors.
impl Ctx {
	/// Creates a root context with the system user ID.
	/// Used for migrations, background jobs, and system operations.
	pub fn root_ctx() -> Self {
		Ctx {
			// System user UUID from database schema
			user_id: uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001")
				.expect("Invalid system UUID"),
		}
	}

	/// Creates a new context with the given user UUID.
	/// This is the primary constructor for user-initiated operations.
	pub fn new(user_id: uuid::Uuid) -> Result<Self> {
		if user_id.is_nil() {
			return Err(Error::CtxCannotNewNilUuid);
		}

		Ok(Self { user_id })
	}

	/// Alias for `new()` - kept for backwards compatibility.
	#[deprecated(since = "0.2.0", note = "Use `Ctx::new()` instead")]
	pub fn new_with_uuid(user_id: uuid::Uuid) -> Result<Self> {
		Self::new(user_id)
	}
}

// Property Accessors.
impl Ctx {
	pub fn user_id(&self) -> uuid::Uuid {
		self.user_id
	}
}
