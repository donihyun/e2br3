use crate::model::base::{CommonIden, DbBmc, TimestampIden};
use lib_utils::time::now_utc;
use modql::field::{SeaField, SeaFields};
use sea_query::IntoIden;
use uuid::Uuid;

/// This method must be called when a model controller intends to create its entity.
/// Adds audit trail fields: created_by, created_at, updated_at
pub fn prep_fields_for_create<MC>(fields: &mut SeaFields, user_id: Uuid)
where
	MC: DbBmc,
{
	if MC::has_owner_id() {
		fields.push(SeaField::new(CommonIden::OwnerId.into_iden(), user_id));
	}
	if MC::has_timestamps() {
		add_timestamps_for_create(fields, user_id);
	}
}

/// This method must be called when a Model Controller plans to update its entity.
/// Adds audit trail fields: updated_by, updated_at
pub fn prep_fields_for_update<MC>(fields: &mut SeaFields, user_id: Uuid)
where
	MC: DbBmc,
{
	if MC::has_timestamps() {
		add_timestamps_for_update(fields, user_id);
	}
}

/// Update the audit trail fields for create.
/// Sets: created_by, created_at, updated_at
/// Note: updated_by is left NULL on create (will be set on first update)
fn add_timestamps_for_create(fields: &mut SeaFields, user_id: Uuid) {
	let now = now_utc();
	fields.push(SeaField::new(TimestampIden::CreatedBy, user_id));
	fields.push(SeaField::new(TimestampIden::CreatedAt, now));
	fields.push(SeaField::new(TimestampIden::UpdatedAt, now));
}

/// Update the audit trail fields for update.
/// Sets: updated_by, updated_at
fn add_timestamps_for_update(fields: &mut SeaFields, user_id: Uuid) {
	let now = now_utc();
	fields.push(SeaField::new(TimestampIden::UpdatedBy, user_id));
	fields.push(SeaField::new(TimestampIden::UpdatedAt, now));
}
