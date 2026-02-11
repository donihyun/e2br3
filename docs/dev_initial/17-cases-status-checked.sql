-- Allow intermediate review status before system-level validation/export readiness.
ALTER TABLE cases
DROP CONSTRAINT IF EXISTS case_status_valid;

ALTER TABLE cases
ADD CONSTRAINT case_status_valid
CHECK (status IN ('draft', 'checked', 'validated', 'submitted', 'archived', 'nullified'));
