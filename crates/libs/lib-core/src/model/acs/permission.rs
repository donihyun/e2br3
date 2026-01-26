//! Permission definitions for the Access Control System
//!
//! Defines resources, actions, and the permission matrix for RBAC.

use crate::ctx::{ROLE_ADMIN, ROLE_MANAGER, ROLE_USER, ROLE_VIEWER};

// region:    --- Resource Enum

/// Resources that can be accessed in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Resource {
	// Core entities
	Case,
	Patient,
	Drug,
	Reaction,
	TestResult,
	Narrative,
	MessageHeader,
	SafetyReport,

	// Nested drug resources
	DrugDosage,
	DrugIndication,
	DrugSubstance,
	DrugReactionAssessment,

	// Other case resources
	PrimarySource,
	LiteratureReference,
	StudyRegistration,
	MedicalHistory,
	PastDrug,
	DeathCause,
	SenderDiagnosis,
	CaseSummary,

	// Administrative
	User,
	Organization,
	AuditLog,

	// Terminology
	Terminology,

	// XML operations
	XmlExport,
	XmlImport,
}

// endregion: --- Resource Enum

// region:    --- Action Enum

/// Actions that can be performed on resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
	Create,
	Read,
	Update,
	Delete,
	List,
	Export,
	Import,
	Approve,
}

// endregion: --- Action Enum

// region:    --- Permission Struct

/// A permission is a combination of a Resource and an Action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Permission(pub Resource, pub Action);

impl Permission {
	pub const fn new(resource: Resource, action: Action) -> Self {
		Self(resource, action)
	}

	pub fn resource(&self) -> Resource {
		self.0
	}

	pub fn action(&self) -> Action {
		self.1
	}
}

impl std::fmt::Display for Permission {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}.{:?}", self.0, self.1)
	}
}

// endregion: --- Permission Struct

// region:    --- Permission Constants

// Case permissions
pub const CASE_CREATE: Permission = Permission::new(Resource::Case, Action::Create);
pub const CASE_READ: Permission = Permission::new(Resource::Case, Action::Read);
pub const CASE_UPDATE: Permission = Permission::new(Resource::Case, Action::Update);
pub const CASE_DELETE: Permission = Permission::new(Resource::Case, Action::Delete);
pub const CASE_LIST: Permission = Permission::new(Resource::Case, Action::List);
pub const CASE_APPROVE: Permission = Permission::new(Resource::Case, Action::Approve);

// Patient permissions
pub const PATIENT_CREATE: Permission = Permission::new(Resource::Patient, Action::Create);
pub const PATIENT_READ: Permission = Permission::new(Resource::Patient, Action::Read);
pub const PATIENT_UPDATE: Permission = Permission::new(Resource::Patient, Action::Update);
pub const PATIENT_DELETE: Permission = Permission::new(Resource::Patient, Action::Delete);

// Drug permissions
pub const DRUG_CREATE: Permission = Permission::new(Resource::Drug, Action::Create);
pub const DRUG_READ: Permission = Permission::new(Resource::Drug, Action::Read);
pub const DRUG_UPDATE: Permission = Permission::new(Resource::Drug, Action::Update);
pub const DRUG_DELETE: Permission = Permission::new(Resource::Drug, Action::Delete);
pub const DRUG_LIST: Permission = Permission::new(Resource::Drug, Action::List);

// Reaction permissions
pub const REACTION_CREATE: Permission = Permission::new(Resource::Reaction, Action::Create);
pub const REACTION_READ: Permission = Permission::new(Resource::Reaction, Action::Read);
pub const REACTION_UPDATE: Permission = Permission::new(Resource::Reaction, Action::Update);
pub const REACTION_DELETE: Permission = Permission::new(Resource::Reaction, Action::Delete);
pub const REACTION_LIST: Permission = Permission::new(Resource::Reaction, Action::List);

// Test Result permissions
pub const TEST_RESULT_CREATE: Permission = Permission::new(Resource::TestResult, Action::Create);
pub const TEST_RESULT_READ: Permission = Permission::new(Resource::TestResult, Action::Read);
pub const TEST_RESULT_UPDATE: Permission = Permission::new(Resource::TestResult, Action::Update);
pub const TEST_RESULT_DELETE: Permission = Permission::new(Resource::TestResult, Action::Delete);
pub const TEST_RESULT_LIST: Permission = Permission::new(Resource::TestResult, Action::List);

// Narrative permissions
pub const NARRATIVE_CREATE: Permission = Permission::new(Resource::Narrative, Action::Create);
pub const NARRATIVE_READ: Permission = Permission::new(Resource::Narrative, Action::Read);
pub const NARRATIVE_UPDATE: Permission = Permission::new(Resource::Narrative, Action::Update);
pub const NARRATIVE_DELETE: Permission = Permission::new(Resource::Narrative, Action::Delete);

// MessageHeader permissions
pub const MESSAGE_HEADER_CREATE: Permission =
	Permission::new(Resource::MessageHeader, Action::Create);
pub const MESSAGE_HEADER_READ: Permission = Permission::new(Resource::MessageHeader, Action::Read);
pub const MESSAGE_HEADER_UPDATE: Permission =
	Permission::new(Resource::MessageHeader, Action::Update);
pub const MESSAGE_HEADER_DELETE: Permission =
	Permission::new(Resource::MessageHeader, Action::Delete);

// SafetyReport permissions
pub const SAFETY_REPORT_CREATE: Permission =
	Permission::new(Resource::SafetyReport, Action::Create);
pub const SAFETY_REPORT_READ: Permission = Permission::new(Resource::SafetyReport, Action::Read);
pub const SAFETY_REPORT_UPDATE: Permission =
	Permission::new(Resource::SafetyReport, Action::Update);
pub const SAFETY_REPORT_DELETE: Permission =
	Permission::new(Resource::SafetyReport, Action::Delete);

// User permissions
pub const USER_CREATE: Permission = Permission::new(Resource::User, Action::Create);
pub const USER_READ: Permission = Permission::new(Resource::User, Action::Read);
pub const USER_UPDATE: Permission = Permission::new(Resource::User, Action::Update);
pub const USER_DELETE: Permission = Permission::new(Resource::User, Action::Delete);
pub const USER_LIST: Permission = Permission::new(Resource::User, Action::List);

// Organization permissions
pub const ORG_CREATE: Permission = Permission::new(Resource::Organization, Action::Create);
pub const ORG_READ: Permission = Permission::new(Resource::Organization, Action::Read);
pub const ORG_UPDATE: Permission = Permission::new(Resource::Organization, Action::Update);
pub const ORG_DELETE: Permission = Permission::new(Resource::Organization, Action::Delete);
pub const ORG_LIST: Permission = Permission::new(Resource::Organization, Action::List);

// AuditLog permissions
pub const AUDIT_READ: Permission = Permission::new(Resource::AuditLog, Action::Read);
pub const AUDIT_LIST: Permission = Permission::new(Resource::AuditLog, Action::List);

// Terminology permissions
pub const TERMINOLOGY_READ: Permission = Permission::new(Resource::Terminology, Action::Read);

// XML permissions
pub const XML_EXPORT: Permission = Permission::new(Resource::XmlExport, Action::Export);
pub const XML_IMPORT: Permission = Permission::new(Resource::XmlImport, Action::Import);

// endregion: --- Permission Constants

// region:    --- Role Permission Mappings

/// Returns all permissions for the admin role
fn admin_permissions() -> &'static [Permission] {
	&[
		// Case - full access
		CASE_CREATE,
		CASE_READ,
		CASE_UPDATE,
		CASE_DELETE,
		CASE_LIST,
		CASE_APPROVE,
		// Patient
		PATIENT_CREATE,
		PATIENT_READ,
		PATIENT_UPDATE,
		PATIENT_DELETE,
		// Drug
		DRUG_CREATE,
		DRUG_READ,
		DRUG_UPDATE,
		DRUG_DELETE,
		DRUG_LIST,
		// Reaction
		REACTION_CREATE,
		REACTION_READ,
		REACTION_UPDATE,
		REACTION_DELETE,
		REACTION_LIST,
		// Test Result
		TEST_RESULT_CREATE,
		TEST_RESULT_READ,
		TEST_RESULT_UPDATE,
		TEST_RESULT_DELETE,
		TEST_RESULT_LIST,
		// Narrative
		NARRATIVE_CREATE,
		NARRATIVE_READ,
		NARRATIVE_UPDATE,
		NARRATIVE_DELETE,
		// MessageHeader
		MESSAGE_HEADER_CREATE,
		MESSAGE_HEADER_READ,
		MESSAGE_HEADER_UPDATE,
		MESSAGE_HEADER_DELETE,
		// SafetyReport
		SAFETY_REPORT_CREATE,
		SAFETY_REPORT_READ,
		SAFETY_REPORT_UPDATE,
		SAFETY_REPORT_DELETE,
		// User - full access
		USER_CREATE,
		USER_READ,
		USER_UPDATE,
		USER_DELETE,
		USER_LIST,
		// Organization - full access
		ORG_CREATE,
		ORG_READ,
		ORG_UPDATE,
		ORG_DELETE,
		ORG_LIST,
		// AuditLog
		AUDIT_READ,
		AUDIT_LIST,
		// Terminology
		TERMINOLOGY_READ,
		// XML
		XML_EXPORT,
		XML_IMPORT,
	]
}

/// Returns all permissions for the manager role
fn manager_permissions() -> &'static [Permission] {
	&[
		// Case - full access including approve
		CASE_CREATE,
		CASE_READ,
		CASE_UPDATE,
		CASE_DELETE,
		CASE_LIST,
		CASE_APPROVE,
		// Patient
		PATIENT_CREATE,
		PATIENT_READ,
		PATIENT_UPDATE,
		PATIENT_DELETE,
		// Drug
		DRUG_CREATE,
		DRUG_READ,
		DRUG_UPDATE,
		DRUG_DELETE,
		DRUG_LIST,
		// Reaction
		REACTION_CREATE,
		REACTION_READ,
		REACTION_UPDATE,
		REACTION_DELETE,
		REACTION_LIST,
		// Test Result
		TEST_RESULT_CREATE,
		TEST_RESULT_READ,
		TEST_RESULT_UPDATE,
		TEST_RESULT_DELETE,
		TEST_RESULT_LIST,
		// Narrative
		NARRATIVE_CREATE,
		NARRATIVE_READ,
		NARRATIVE_UPDATE,
		NARRATIVE_DELETE,
		// MessageHeader
		MESSAGE_HEADER_CREATE,
		MESSAGE_HEADER_READ,
		MESSAGE_HEADER_UPDATE,
		MESSAGE_HEADER_DELETE,
		// SafetyReport
		SAFETY_REPORT_CREATE,
		SAFETY_REPORT_READ,
		SAFETY_REPORT_UPDATE,
		SAFETY_REPORT_DELETE,
		// User - read only
		USER_READ,
		USER_LIST,
		// Organization - read own
		ORG_READ,
		// AuditLog - can view
		AUDIT_READ,
		AUDIT_LIST,
		// Terminology
		TERMINOLOGY_READ,
		// XML
		XML_EXPORT,
		XML_IMPORT,
	]
}

/// Returns all permissions for the regular user role
fn user_permissions() -> &'static [Permission] {
	&[
		// Case - CRUD but no delete, no approve
		CASE_CREATE,
		CASE_READ,
		CASE_UPDATE,
		CASE_LIST,
		// Patient
		PATIENT_CREATE,
		PATIENT_READ,
		PATIENT_UPDATE,
		// Drug
		DRUG_CREATE,
		DRUG_READ,
		DRUG_UPDATE,
		DRUG_LIST,
		// Reaction
		REACTION_CREATE,
		REACTION_READ,
		REACTION_UPDATE,
		REACTION_LIST,
		// Test Result
		TEST_RESULT_CREATE,
		TEST_RESULT_READ,
		TEST_RESULT_UPDATE,
		TEST_RESULT_LIST,
		// Narrative
		NARRATIVE_CREATE,
		NARRATIVE_READ,
		NARRATIVE_UPDATE,
		// MessageHeader
		MESSAGE_HEADER_CREATE,
		MESSAGE_HEADER_READ,
		MESSAGE_HEADER_UPDATE,
		// SafetyReport
		SAFETY_REPORT_CREATE,
		SAFETY_REPORT_READ,
		SAFETY_REPORT_UPDATE,
		// User - read self only (handled at endpoint level)
		USER_READ,
		// Organization - read own
		ORG_READ,
		// Terminology
		TERMINOLOGY_READ,
		// XML - export only
		XML_EXPORT,
	]
}

/// Returns all permissions for the viewer role
fn viewer_permissions() -> &'static [Permission] {
	&[
		// Case - read only
		CASE_READ,
		CASE_LIST,
		// Patient
		PATIENT_READ,
		// Drug
		DRUG_READ,
		DRUG_LIST,
		// Reaction
		REACTION_READ,
		REACTION_LIST,
		// Test Result
		TEST_RESULT_READ,
		TEST_RESULT_LIST,
		// Narrative
		NARRATIVE_READ,
		// MessageHeader
		MESSAGE_HEADER_READ,
		// SafetyReport
		SAFETY_REPORT_READ,
		// User - read only
		USER_READ,
		USER_LIST,
		// Organization - read own
		ORG_READ,
		// Terminology
		TERMINOLOGY_READ,
		// XML - export only (viewing)
		XML_EXPORT,
	]
}

// endregion: --- Role Permission Mappings

// region:    --- Permission Checking Functions

/// Returns the permissions for a given role
pub fn role_permissions(role: &str) -> &'static [Permission] {
	match role {
		ROLE_ADMIN => admin_permissions(),
		ROLE_MANAGER => manager_permissions(),
		ROLE_USER => user_permissions(),
		ROLE_VIEWER => viewer_permissions(),
		_ => &[], // Unknown role has no permissions
	}
}

/// Checks if a role has a specific permission
pub fn has_permission(role: &str, permission: Permission) -> bool {
	role_permissions(role).contains(&permission)
}

/// Checks if a role has any of the given permissions
pub fn has_any_permission(role: &str, permissions: &[Permission]) -> bool {
	let role_perms = role_permissions(role);
	permissions.iter().any(|p| role_perms.contains(p))
}

/// Checks if a role has all of the given permissions
pub fn has_all_permissions(role: &str, permissions: &[Permission]) -> bool {
	let role_perms = role_permissions(role);
	permissions.iter().all(|p| role_perms.contains(p))
}

// endregion: --- Permission Checking Functions

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_admin_has_all_permissions() {
		assert!(has_permission(ROLE_ADMIN, CASE_CREATE));
		assert!(has_permission(ROLE_ADMIN, CASE_DELETE));
		assert!(has_permission(ROLE_ADMIN, USER_CREATE));
		assert!(has_permission(ROLE_ADMIN, USER_DELETE));
		assert!(has_permission(ROLE_ADMIN, ORG_CREATE));
		assert!(has_permission(ROLE_ADMIN, AUDIT_LIST));
	}

	#[test]
	fn test_manager_permissions() {
		// Manager can do case operations
		assert!(has_permission(ROLE_MANAGER, CASE_CREATE));
		assert!(has_permission(ROLE_MANAGER, CASE_DELETE));
		assert!(has_permission(ROLE_MANAGER, CASE_APPROVE));
		// Manager can read users but not create
		assert!(has_permission(ROLE_MANAGER, USER_READ));
		assert!(!has_permission(ROLE_MANAGER, USER_CREATE));
		// Manager cannot manage organizations
		assert!(!has_permission(ROLE_MANAGER, ORG_CREATE));
	}

	#[test]
	fn test_user_permissions() {
		// User can create and update cases
		assert!(has_permission(ROLE_USER, CASE_CREATE));
		assert!(has_permission(ROLE_USER, CASE_UPDATE));
		// User cannot delete cases or approve
		assert!(!has_permission(ROLE_USER, CASE_DELETE));
		assert!(!has_permission(ROLE_USER, CASE_APPROVE));
		// User cannot manage users
		assert!(!has_permission(ROLE_USER, USER_CREATE));
		// User cannot import XML
		assert!(!has_permission(ROLE_USER, XML_IMPORT));
	}

	#[test]
	fn test_viewer_permissions() {
		// Viewer can only read
		assert!(has_permission(ROLE_VIEWER, CASE_READ));
		assert!(has_permission(ROLE_VIEWER, CASE_LIST));
		// Viewer cannot create, update, or delete
		assert!(!has_permission(ROLE_VIEWER, CASE_CREATE));
		assert!(!has_permission(ROLE_VIEWER, CASE_UPDATE));
		assert!(!has_permission(ROLE_VIEWER, CASE_DELETE));
	}

	#[test]
	fn test_unknown_role_has_no_permissions() {
		assert!(!has_permission("unknown", CASE_READ));
		assert!(!has_permission("hacker", USER_DELETE));
	}

	#[test]
	fn test_has_any_permission() {
		assert!(has_any_permission(
			ROLE_VIEWER,
			&[CASE_CREATE, CASE_READ]
		));
		assert!(!has_any_permission(
			ROLE_VIEWER,
			&[CASE_CREATE, CASE_DELETE]
		));
	}

	#[test]
	fn test_has_all_permissions() {
		assert!(has_all_permissions(ROLE_ADMIN, &[CASE_CREATE, CASE_DELETE]));
		assert!(!has_all_permissions(
			ROLE_VIEWER,
			&[CASE_READ, CASE_CREATE]
		));
	}
}

// endregion: --- Tests
