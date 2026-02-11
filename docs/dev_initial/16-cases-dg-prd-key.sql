-- Add dedicated duplication/product key storage on cases.
ALTER TABLE cases
	ADD COLUMN IF NOT EXISTS dg_prd_key TEXT;

