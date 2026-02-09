-- Add explicit validation profile on cases.
-- Safe to run multiple times.

ALTER TABLE cases
  ADD COLUMN IF NOT EXISTS validation_profile VARCHAR(10);

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1
    FROM pg_constraint
    WHERE conname = 'case_validation_profile_valid'
  ) THEN
    ALTER TABLE cases
      ADD CONSTRAINT case_validation_profile_valid
      CHECK (validation_profile IS NULL OR validation_profile IN ('ich', 'fda', 'mfds'));
  END IF;
END$$;

CREATE INDEX IF NOT EXISTS idx_cases_validation_profile
  ON cases(validation_profile);
