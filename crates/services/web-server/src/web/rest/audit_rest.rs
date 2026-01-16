use lib_core::model::audit::{AuditLogBmc, AuditLogFilter, AuditLogForCreate};
use lib_rest_core::prelude::*;

// This macro generates CRUD functions for immutable audit logs:
// - create_audit_log
// - get_audit_log
// - list_audit_logs
// - delete_audit_log (admin only)
// Note: No update function - audit logs are immutable
generate_common_rest_fns! {
	Bmc: AuditLogBmc,
	Entity: lib_core::model::audit::AuditLog,
	ForCreate: AuditLogForCreate,
	Filter: AuditLogFilter,
	Suffix: audit_log,
	Id: i64
}
