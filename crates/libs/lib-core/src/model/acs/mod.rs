//! Access Control System (ACS) based on PBAC (Privilege Based Access Control)
//!
//! This module provides the permission framework for SafetyDB:
//!
//! - **Resources**: Entities that can be accessed (Case, User, Drug, etc.)
//! - **Actions**: Operations on resources (Create, Read, Update, Delete, etc.)
//! - **Permissions**: Resource + Action combinations
//! - **Roles**: Collections of permissions (admin, manager, user, viewer)
//!
//! # Usage
//!
//! ```rust,ignore
//! use lib_core::model::acs::{has_permission, Permission, Resource, Action, CASE_CREATE};
//!
//! // Check if a role has a specific permission
//! if has_permission("manager", CASE_CREATE) {
//!     // Allow the operation
//! }
//!
//! // Or use the permission constants directly
//! let perm = Permission::new(Resource::Case, Action::Create);
//! if has_permission(ctx.role(), perm) {
//!     // Allow
//! }
//! ```
//!
//! # Role Hierarchy
//!
//! | Role    | Description                                    |
//! |---------|------------------------------------------------|
//! | admin   | Full access to all resources                   |
//! | manager | Case management + user viewing + audit access  |
//! | user    | Case CRUD (no delete), no user management      |
//! | viewer  | Read-only access to cases and users            |

mod permission;

pub use permission::*;
