# E2B(R3) Safety Database Audit Trail System Migration Plan

**Document Version:** 1.0
**Date:** 2026-01-14
**Status:** APPROVED FOR IMPLEMENTATION
**Migration Strategy:** Option B - Full Standardization with UUID-Only Audit Trail

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Objectives](#objectives)
3. [Current State Analysis](#current-state-analysis)
4. [Target State Architecture](#target-state-architecture)
5. [Migration Strategy](#migration-strategy)
6. [Step-by-Step Implementation Plan](#step-by-step-implementation-plan)
7. [Validation & Testing](#validation--testing)
8. [Rollback Procedures](#rollback-procedures)
9. [Risk Mitigation](#risk-mitigation)
10. [Compliance Checklist](#compliance-checklist)

---

## Executive Summary

This document outlines the complete migration from a **dual audit ID system (i64 + UUID)** to a **unified UUID-based audit trail system** for the E2B(R3) safety database to achieve full compliance with:

- **21 CFR Part 11** (FDA Electronic Records)
- **EMA GVP Module VI** (EU Pharmacovigilance)
- **ALCOA+ Data Integrity Principles**
- **MFDS Guidelines** (Korea)

**Timeline:** 8-10 weeks
**Effort:** ~120-150 hours
**Risk Level:** Medium (with proper testing)
**Validation Required:** Yes (IQ/OQ/PQ)

---

## Objectives

### Primary Goals

1. ✅ **Eliminate dual user ID system** (i64 `audit_id` + UUID `id`)
2. ✅ **Standardize audit fields** across all 31 Business Model Controllers
3. ✅ **Ensure 100% user attribution** in all audit logs
4. ✅ **Implement tamper-proof audit trail** using PostgreSQL RLS
5. ✅ **Achieve regulatory compliance** for production deployment

### Success Criteria

- All database operations log user UUID to `audit_logs` table
- All 31 models have consistent `created_by` / `updated_by` fields
- No application code uses i64 `user_audit_id`
- Audit trail passes IQ/OQ/PQ validation tests
- System ready for FDA/EMA inspection

---

## Current State Analysis

### Current Architecture Issues

| Component | Current State | Issue |
|-----------|--------------|-------|
| **User Identity** | Dual system: UUID `id` + i64 `audit_id` | Complexity, inconsistency risk |
| **Ctx Structure** | Holds both `user_id: UUID` and `user_audit_id: i64` | Confusing, maintenance burden |
| **Model Audit Fields** | 3 different patterns across 31 models | Inconsistent, hard to maintain |
| **Database Triggers** | Use UUID but depend on session variable | Session variable NOT set in production |
| **Manual CRUD** | 28 models bypass audit field injection | No app-level user tracking |
| **Compliance** | ❌ Non-compliant with 21 CFR Part 11 | Production blocker |

### Current Model Patterns

```
Pattern A (2 models): Organization, User
  - cid: i64, ctime: DateTime, mid: i64, mtime: DateTime

Pattern B (1 model): Case
  - created_by: UUID, updated_by: UUID, created_at: DateTime, updated_at: DateTime

Pattern C (28 models): All E2B entities
  - created_at: DateTime, updated_at: DateTime (NO user tracking)
```

---

## Target State Architecture

### Unified Audit System

```
┌─────────────────────────────────────────────────────────────────┐
│                    APPLICATION LAYER                             │
├─────────────────────────────────────────────────────────────────┤
│  All Models (31 BMCs)                                            │
│    - id: UUID                                                    │
│    - created_at: DateTime    ← Convenience fields                │
│    - updated_at: DateTime    ← for quick queries                 │
│    - created_by: UUID        ← User attribution                  │
│    - updated_by: Option<UUID> ← User attribution                 │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                    DATABASE LAYER                                │
├─────────────────────────────────────────────────────────────────┤
│  PostgreSQL Triggers (26 tables)                                 │
│    → Automatically log to audit_logs table                       │
│    → Capture old_values / new_values (JSONB)                     │
│    → Use current_setting('app.current_user_id')::UUID            │
├─────────────────────────────────────────────────────────────────┤
│  audit_logs Table (Complete Audit Trail)                         │
│    - id: BIGSERIAL                                               │
│    - table_name: VARCHAR                                         │
│    - record_id: UUID                                             │
│    - action: VARCHAR (CREATE/UPDATE/DELETE/SUBMIT/NULLIFY)       │
│    - user_id: UUID  ← ALWAYS populated                           │
│    - old_values: JSONB                                           │
│    - new_values: JSONB                                           │
│    - ip_address: INET                                            │
│    - user_agent: TEXT                                            │
│    - created_at: TIMESTAMPTZ                                     │
│                                                                   │
│  Row-Level Security: Append-only, read via auditor role          │
└─────────────────────────────────────────────────────────────────┘
```

### Key Principles

1. **Single Source of Truth:** PostgreSQL `audit_logs` table is authoritative
2. **Single User ID:** UUID only (no i64 audit_id)
3. **Consistent Model Pattern:** All 31 models use Pattern B (Case model style)
4. **Automatic Logging:** Database triggers ensure no operation is missed
5. **Tamper-Proof:** Row-Level Security prevents unauthorized modifications

---

## Migration Strategy

### Phased Approach

**Phase 1:** Database Foundation (Weeks 1-2)
- Remove i64 audit_id from users table
- Enable session variable setting in production
- Implement Row-Level Security on audit_logs
- Add created_by/updated_by columns to all tables

**Phase 2:** Application Code Refactoring (Weeks 3-5)
- Update Ctx to remove user_audit_id
- Migrate all 31 models to Pattern B
- Refactor base CRUD layer
- Update all manual CRUD implementations

**Phase 3:** Testing & Validation (Weeks 6-8)
- Comprehensive integration testing
- IQ/OQ/PQ validation execution
- Performance testing
- Security testing

**Phase 4:** Documentation & Deployment (Weeks 9-10)
- Complete validation documentation
- Create SOPs
- Production deployment
- Post-deployment verification

---

## Step-by-Step Implementation Plan

## PHASE 1: Database Foundation (Weeks 1-2)

### Step 1.1: Create Migration Scripts Directory

**Duration:** 15 minutes

```bash
mkdir -p sql/migrations/audit_trail_migration
cd sql/migrations/audit_trail_migration
```

**Deliverable:** Migration scripts directory structure

---

### Step 1.2: Backup Production Database

**Duration:** 1 hour

```bash
# Create complete database backup
pg_dump -h localhost -U e2br3_user -d e2br3_db \
  --format=custom \
  --file=backup_pre_audit_migration_$(date +%Y%m%d_%H%M%S).dump

# Verify backup integrity
pg_restore --list backup_pre_audit_migration_*.dump | head -20

# Document backup location and checksum
sha256sum backup_pre_audit_migration_*.dump > backup_checksum.txt
```

**Deliverable:** Database backup file with checksum

---

### Step 1.3: Create Migration Script - Add created_by/updated_by Columns

**Duration:** 2 hours

**File:** `sql/migrations/audit_trail_migration/001_add_audit_columns.sql`

```sql
-- Migration 001: Add created_by and updated_by columns to all tables
-- Date: 2026-01-14
-- Author: System Architect
-- Purpose: Standardize user attribution across all models

-- Start transaction
BEGIN;

-- ============================================================================
-- PART 1: Add columns to users and organizations (replace cid/mid)
-- ============================================================================

-- Users table: Add new columns
ALTER TABLE users
  ADD COLUMN created_by UUID REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Migrate existing cid/mid data to created_by/updated_by
-- Note: This requires mapping audit_id (i64) back to UUID
UPDATE users u1
SET created_by = u2.id
FROM users u2
WHERE u1.cid = u2.audit_id;

UPDATE users u1
SET updated_by = u2.id
FROM users u2
WHERE u1.mid = u2.audit_id;

-- Organizations table: Add new columns
ALTER TABLE organizations
  ADD COLUMN created_by UUID REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Migrate existing cid/mid data
UPDATE organizations o
SET created_by = u.id
FROM users u
WHERE o.cid = u.audit_id;

UPDATE organizations o
SET updated_by = u.id
FROM users u
WHERE o.mid = u.audit_id;

-- ============================================================================
-- PART 2: Add columns to E2B entity tables (28 models)
-- ============================================================================

-- Patient Information
ALTER TABLE patient_information
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Message Header
ALTER TABLE message_headers
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Safety Report Identification
ALTER TABLE safety_report_identification
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Sender Information
ALTER TABLE sender_information
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Literature References
ALTER TABLE literature_references
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Study Information
ALTER TABLE study_information
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Study Registration Numbers
ALTER TABLE study_registration_numbers
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Primary Sources
ALTER TABLE primary_sources
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Medical History Episodes
ALTER TABLE medical_history_episodes
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Past Drug History
ALTER TABLE past_drug_history
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Patient Death Information
ALTER TABLE patient_death_information
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Reported Causes of Death
ALTER TABLE reported_causes_of_death
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Autopsy Causes of Death
ALTER TABLE autopsy_causes_of_death
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Parent Information
ALTER TABLE parent_information
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Reactions
ALTER TABLE reactions
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Test Results
ALTER TABLE test_results
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Drug Information
ALTER TABLE drug_information
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Drug Active Substances
ALTER TABLE drug_active_substances
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Dosage Information
ALTER TABLE dosage_information
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Drug Indications
ALTER TABLE drug_indications
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Narrative Information
ALTER TABLE narrative_information
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Sender Diagnoses
ALTER TABLE sender_diagnoses
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- Case Summary Information
ALTER TABLE case_summary_information
  ADD COLUMN created_by UUID NOT NULL REFERENCES users(id),
  ADD COLUMN updated_by UUID REFERENCES users(id);

-- ============================================================================
-- PART 3: Set default values for existing records
-- ============================================================================

-- Create a system user for historical records if not exists
INSERT INTO users (id, username, email, role, password_hash, audit_id)
VALUES (
  '00000000-0000-0000-0000-000000000001'::UUID,
  'system_migration',
  'system@migration.local',
  'system',
  'MIGRATION_NO_LOGIN',
  1
) ON CONFLICT (id) DO NOTHING;

-- Update all existing records to have created_by = system user
-- This is for records that existed before migration

UPDATE patient_information
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE message_headers
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE safety_report_identification
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE sender_information
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE literature_references
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE study_information
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE study_registration_numbers
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE primary_sources
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE medical_history_episodes
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE past_drug_history
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE patient_death_information
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE reported_causes_of_death
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE autopsy_causes_of_death
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE parent_information
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE reactions
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE test_results
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE drug_information
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE drug_active_substances
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE dosage_information
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE drug_indications
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE narrative_information
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE sender_diagnoses
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

UPDATE case_summary_information
SET created_by = '00000000-0000-0000-0000-000000000001'::UUID
WHERE created_by IS NULL;

-- ============================================================================
-- PART 4: Create verification query
-- ============================================================================

-- Verify all tables have the new columns
SELECT
  table_name,
  column_name,
  data_type,
  is_nullable
FROM information_schema.columns
WHERE table_schema = 'public'
  AND column_name IN ('created_by', 'updated_by')
ORDER BY table_name, column_name;

COMMIT;
```

**Execute:**
```bash
psql -h localhost -U e2br3_user -d e2br3_db -f 001_add_audit_columns.sql
```

**Verification:**
```sql
-- Count tables with audit columns
SELECT COUNT(DISTINCT table_name) as tables_with_audit_columns
FROM information_schema.columns
WHERE table_schema = 'public'
  AND column_name IN ('created_by', 'updated_by');
-- Expected: 31 tables (or 62 rows / 2 columns per table)
```

**Deliverable:** All tables have `created_by` and `updated_by` columns

---

### Step 1.4: Remove Old Audit ID Columns

**Duration:** 1 hour

**File:** `sql/migrations/audit_trail_migration/002_remove_old_audit_columns.sql`

```sql
-- Migration 002: Remove old i64 audit_id columns
-- Date: 2026-01-14
-- Purpose: Clean up dual ID system

BEGIN;

-- Drop old columns from users table
ALTER TABLE users
  DROP COLUMN IF EXISTS audit_id,
  DROP COLUMN IF EXISTS cid,
  DROP COLUMN IF EXISTS ctime,
  DROP COLUMN IF EXISTS mid,
  DROP COLUMN IF EXISTS mtime;

-- Drop old columns from organizations table
ALTER TABLE organizations
  DROP COLUMN IF EXISTS cid,
  DROP COLUMN IF EXISTS ctime,
  DROP COLUMN IF EXISTS mid,
  DROP COLUMN IF EXISTS mtime;

-- Verify columns are removed
SELECT table_name, column_name
FROM information_schema.columns
WHERE table_schema = 'public'
  AND table_name IN ('users', 'organizations')
  AND column_name IN ('audit_id', 'cid', 'ctime', 'mid', 'mtime');
-- Expected: 0 rows

COMMIT;
```

**Execute:**
```bash
psql -h localhost -U e2br3_user -d e2br3_db -f 002_remove_old_audit_columns.sql
```

**Deliverable:** Old i64 audit columns removed

---

### Step 1.5: Implement Row-Level Security on audit_logs

**Duration:** 2 hours

**File:** `sql/migrations/audit_trail_migration/003_audit_logs_rls.sql`

```sql
-- Migration 003: Implement Row-Level Security for audit_logs
-- Date: 2026-01-14
-- Purpose: Make audit_logs tamper-proof (append-only)

BEGIN;

-- Enable Row-Level Security
ALTER TABLE audit_logs ENABLE ROW LEVEL SECURITY;

-- Drop existing policies if any
DROP POLICY IF EXISTS audit_logs_append_only ON audit_logs;
DROP POLICY IF EXISTS audit_logs_read_for_auditors ON audit_logs;

-- Create application role (used by API connections)
DO $$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'e2br3_app_role') THEN
    CREATE ROLE e2br3_app_role;
  END IF;
END
$$;

-- Create auditor role (read-only access to audit logs)
DO $$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'e2br3_auditor_role') THEN
    CREATE ROLE e2br3_auditor_role;
  END IF;
END
$$;

-- Policy 1: Allow INSERT only for application role
CREATE POLICY audit_logs_append_only ON audit_logs
  FOR INSERT
  TO e2br3_app_role
  WITH CHECK (true);

-- Policy 2: Deny UPDATE and DELETE for application role
CREATE POLICY audit_logs_no_modify ON audit_logs
  FOR ALL
  TO e2br3_app_role
  USING (false);

-- Policy 3: Allow SELECT for auditor role
CREATE POLICY audit_logs_read_for_auditors ON audit_logs
  FOR SELECT
  TO e2br3_auditor_role
  USING (true);

-- Grant necessary permissions
GRANT INSERT ON audit_logs TO e2br3_app_role;
GRANT SELECT ON audit_logs TO e2br3_auditor_role;
GRANT USAGE ON SEQUENCE audit_logs_id_seq TO e2br3_app_role;

-- Make current e2br3_user a member of app role
GRANT e2br3_app_role TO e2br3_user;

-- Test: Try to modify audit log (should fail)
-- Uncomment to test:
-- SET ROLE e2br3_app_role;
-- DELETE FROM audit_logs WHERE id = 1;  -- Should fail
-- RESET ROLE;

-- Verify RLS is enabled
SELECT schemaname, tablename, rowsecurity
FROM pg_tables
WHERE tablename = 'audit_logs';

-- Verify policies exist
SELECT schemaname, tablename, policyname, cmd
FROM pg_policies
WHERE tablename = 'audit_logs';

COMMIT;
```

**Execute:**
```bash
psql -h localhost -U e2br3_user -d e2br3_db -f 003_audit_logs_rls.sql
```

**Verification:**
```sql
-- Test append-only behavior
SET ROLE e2br3_app_role;

-- This should succeed
INSERT INTO audit_logs (table_name, record_id, action, user_id, new_values)
VALUES ('test_table', gen_random_uuid(), 'TEST',
        (SELECT id FROM users LIMIT 1), '{"test": true}'::jsonb);

-- This should fail
DELETE FROM audit_logs WHERE table_name = 'test_table';  -- ERROR

-- This should fail
UPDATE audit_logs SET action = 'MODIFIED' WHERE table_name = 'test_table';  -- ERROR

RESET ROLE;

-- Clean up test data
DELETE FROM audit_logs WHERE table_name = 'test_table';
```

**Deliverable:** Tamper-proof audit_logs table with RLS

---

### Step 1.6: Create Helper Functions for User Context

**Duration:** 2 hours

**File:** `sql/migrations/audit_trail_migration/004_user_context_functions.sql`

```sql
-- Migration 004: Helper functions for user context management
-- Date: 2026-01-14
-- Purpose: Manage PostgreSQL session variables for audit trail

BEGIN;

-- Function to set current user context for transaction
CREATE OR REPLACE FUNCTION set_current_user_context(p_user_id UUID)
RETURNS void
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
  -- Set session variable (transaction-scoped)
  PERFORM set_config('app.current_user_id', p_user_id::text, true);
END;
$$;

-- Function to get current user context
CREATE OR REPLACE FUNCTION get_current_user_context()
RETURNS UUID
LANGUAGE plpgsql
STABLE
AS $$
DECLARE
  v_user_id TEXT;
BEGIN
  v_user_id := current_setting('app.current_user_id', true);

  IF v_user_id IS NULL OR v_user_id = '' THEN
    RAISE EXCEPTION 'No user context set. Call set_current_user_context() first.';
  END IF;

  RETURN v_user_id::UUID;
EXCEPTION
  WHEN OTHERS THEN
    RAISE EXCEPTION 'Invalid user context: %', SQLERRM;
END;
$$;

-- Function to validate user context is set
CREATE OR REPLACE FUNCTION validate_user_context()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  -- Ensure user context is set before any DML operation
  PERFORM get_current_user_context();
  RETURN NEW;
EXCEPTION
  WHEN OTHERS THEN
    RAISE EXCEPTION 'User context validation failed: %. Ensure set_current_user_context() is called.', SQLERRM;
END;
$$;

-- Grant execute permissions
GRANT EXECUTE ON FUNCTION set_current_user_context(UUID) TO e2br3_app_role;
GRANT EXECUTE ON FUNCTION get_current_user_context() TO e2br3_app_role;
GRANT EXECUTE ON FUNCTION validate_user_context() TO e2br3_app_role;

COMMIT;
```

**Execute:**
```bash
psql -h localhost -U e2br3_user -d e2br3_db -f 004_user_context_functions.sql
```

**Test:**
```sql
-- Test setting and getting user context
BEGIN;

-- Set user context
SELECT set_current_user_context((SELECT id FROM users LIMIT 1));

-- Get user context (should return the same UUID)
SELECT get_current_user_context();

ROLLBACK;
```

**Deliverable:** Helper functions for user context management

---

### Step 1.7: Update Audit Triggers to Use Helper Function

**Duration:** 1 hour

**File:** `sql/migrations/audit_trail_migration/005_update_audit_triggers.sql`

```sql
-- Migration 005: Update audit triggers to use helper function
-- Date: 2026-01-14
-- Purpose: Ensure audit triggers always capture user_id

BEGIN;

-- Drop existing audit trigger function
DROP FUNCTION IF EXISTS audit_trigger_function() CASCADE;

-- Create improved audit trigger function
CREATE OR REPLACE FUNCTION audit_trigger_function()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
  v_user_id UUID;
BEGIN
  -- Get user from context (will fail if not set)
  v_user_id := get_current_user_context();

  IF TG_OP = 'INSERT' THEN
    INSERT INTO audit_logs (table_name, record_id, action, user_id, new_values)
    VALUES (TG_TABLE_NAME, NEW.id, 'CREATE', v_user_id, to_jsonb(NEW));
    RETURN NEW;

  ELSIF TG_OP = 'UPDATE' THEN
    INSERT INTO audit_logs (table_name, record_id, action, user_id, old_values, new_values)
    VALUES (TG_TABLE_NAME, NEW.id, 'UPDATE', v_user_id, to_jsonb(OLD), to_jsonb(NEW));
    RETURN NEW;

  ELSIF TG_OP = 'DELETE' THEN
    INSERT INTO audit_logs (table_name, record_id, action, user_id, old_values)
    VALUES (TG_TABLE_NAME, OLD.id, 'DELETE', v_user_id, to_jsonb(OLD));
    RETURN OLD;
  END IF;

EXCEPTION
  WHEN OTHERS THEN
    RAISE EXCEPTION 'Audit trail logging failed: %. User context may not be set.', SQLERRM;
END;
$$;

-- Recreate all audit triggers (they were dropped with CASCADE)
-- Cases
CREATE TRIGGER audit_cases AFTER INSERT OR UPDATE OR DELETE ON cases
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Patient Information
CREATE TRIGGER audit_patient_info AFTER INSERT OR UPDATE OR DELETE ON patient_information
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Message Headers
CREATE TRIGGER audit_message_headers AFTER INSERT OR UPDATE OR DELETE ON message_headers
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Safety Report Identification
CREATE TRIGGER audit_safety_report_identification AFTER INSERT OR UPDATE OR DELETE ON safety_report_identification
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Sender Information
CREATE TRIGGER audit_sender_information AFTER INSERT OR UPDATE OR DELETE ON sender_information
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Literature References
CREATE TRIGGER audit_literature_references AFTER INSERT OR UPDATE OR DELETE ON literature_references
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Study Information
CREATE TRIGGER audit_study_information AFTER INSERT OR UPDATE OR DELETE ON study_information
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Study Registration Numbers
CREATE TRIGGER audit_study_registration_numbers AFTER INSERT OR UPDATE OR DELETE ON study_registration_numbers
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Primary Sources
CREATE TRIGGER audit_primary_sources AFTER INSERT OR UPDATE OR DELETE ON primary_sources
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Medical History Episodes
CREATE TRIGGER audit_medical_history_episodes AFTER INSERT OR UPDATE OR DELETE ON medical_history_episodes
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Past Drug History
CREATE TRIGGER audit_past_drug_history AFTER INSERT OR UPDATE OR DELETE ON past_drug_history
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Patient Death Information
CREATE TRIGGER audit_patient_death_information AFTER INSERT OR UPDATE OR DELETE ON patient_death_information
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Reported Causes of Death
CREATE TRIGGER audit_reported_causes_of_death AFTER INSERT OR UPDATE OR DELETE ON reported_causes_of_death
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Autopsy Causes of Death
CREATE TRIGGER audit_autopsy_causes_of_death AFTER INSERT OR UPDATE OR DELETE ON autopsy_causes_of_death
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Parent Information
CREATE TRIGGER audit_parent_information AFTER INSERT OR UPDATE OR DELETE ON parent_information
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Reactions
CREATE TRIGGER audit_reactions AFTER INSERT OR UPDATE OR DELETE ON reactions
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Test Results
CREATE TRIGGER audit_test_results AFTER INSERT OR UPDATE OR DELETE ON test_results
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Drug Information
CREATE TRIGGER audit_drug_information AFTER INSERT OR UPDATE OR DELETE ON drug_information
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Drug Active Substances
CREATE TRIGGER audit_drug_active_substances AFTER INSERT OR UPDATE OR DELETE ON drug_active_substances
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Dosage Information
CREATE TRIGGER audit_dosage_information AFTER INSERT OR UPDATE OR DELETE ON dosage_information
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Drug Indications
CREATE TRIGGER audit_drug_indications AFTER INSERT OR UPDATE OR DELETE ON drug_indications
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Narrative Information
CREATE TRIGGER audit_narrative_information AFTER INSERT OR UPDATE OR DELETE ON narrative_information
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Sender Diagnoses
CREATE TRIGGER audit_sender_diagnoses AFTER INSERT OR UPDATE OR DELETE ON sender_diagnoses
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Case Summary Information
CREATE TRIGGER audit_case_summary_information AFTER INSERT OR UPDATE OR DELETE ON case_summary_information
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Add triggers for users and organizations
CREATE TRIGGER audit_users AFTER INSERT OR UPDATE OR DELETE ON users
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_organizations AFTER INSERT OR UPDATE OR DELETE ON organizations
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Verify all triggers are created
SELECT
  event_object_table AS table_name,
  trigger_name,
  event_manipulation AS event
FROM information_schema.triggers
WHERE trigger_schema = 'public'
  AND trigger_name LIKE 'audit_%'
ORDER BY event_object_table;

COMMIT;
```

**Execute:**
```bash
psql -h localhost -U e2br3_user -d e2br3_db -f 005_update_audit_triggers.sql
```

**Deliverable:** All 26+ tables have updated audit triggers

---

## PHASE 2: Application Code Refactoring (Weeks 3-5)

### Step 2.1: Update Ctx Structure

**Duration:** 1 hour

**File:** `crates/libs/lib-core/src/ctx/mod.rs`

```rust
// Remove old implementation and replace with:

use crate::model::Error;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ctx {
    user_id: Uuid,
}

impl Ctx {
    pub fn new(user_id: Uuid) -> Result<Self, Error> {
        if user_id == Uuid::nil() {
            return Err(Error::InvalidUserId);
        }
        Ok(Self { user_id })
    }

    pub fn root_ctx() -> Self {
        // System user for background jobs, migrations, etc.
        Self {
            user_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                .expect("Invalid system UUID"),
        }
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctx_creation() {
        let user_id = Uuid::new_v4();
        let ctx = Ctx::new(user_id).unwrap();
        assert_eq!(ctx.user_id(), user_id);
    }

    #[test]
    fn test_ctx_nil_uuid_rejected() {
        let result = Ctx::new(Uuid::nil());
        assert!(result.is_err());
    }

    #[test]
    fn test_root_ctx() {
        let ctx = Ctx::root_ctx();
        assert_eq!(
            ctx.user_id(),
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
        );
    }
}
```

**Verification:**
```bash
cargo test --package lib-core ctx::tests
```

**Deliverable:** Simplified Ctx with UUID-only user identification

---

### Step 2.2: Add User Context Helper to Database Layer

**Duration:** 2 hours

**File:** `crates/libs/lib-core/src/model/store/mod.rs`

```rust
// Update existing file to add:

use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

// ... existing code ...

/// Sets the current user context for PostgreSQL session
/// This enables audit triggers to capture user_id
pub async fn set_user_context(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> Result<()> {
    sqlx::query("SELECT set_current_user_context($1)")
        .bind(user_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| Error::Store(format!("Failed to set user context: {}", e)))?;

    Ok(())
}

/// Gets the current user context from PostgreSQL session
pub async fn get_user_context(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<Uuid> {
    let row: (Uuid,) = sqlx::query_as("SELECT get_current_user_context()")
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| Error::Store(format!("Failed to get user context: {}", e)))?;

    Ok(row.0)
}

// Update new_db_pool to remove old test user ID logic
pub async fn new_db_pool() -> sqlx::Result<Db> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&core_config().DB_URL)
        .await
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_user_context_setting() {
        let db = new_db_pool().await.unwrap();
        let mut tx = db.begin().await.unwrap();

        let user_id = Uuid::new_v4();
        set_user_context(&mut tx, user_id).await.unwrap();

        let retrieved_id = get_user_context(&mut tx).await.unwrap();
        assert_eq!(user_id, retrieved_id);

        tx.rollback().await.unwrap();
    }
}
```

**Verification:**
```bash
cargo test --package lib-core store::tests::test_user_context_setting
```

**Deliverable:** Helper functions for PostgreSQL user context

---

### Step 2.3: Update Base CRUD Layer

**Duration:** 4 hours

**File:** `crates/libs/lib-core/src/model/base/base_uuid.rs`

```rust
// Replace existing implementation with:

use crate::ctx::Ctx;
use crate::model::store::{set_user_context, Db};
use crate::model::{Error, Result};
use modql::filter::FilterGroups;
use modql::field::{Fields, HasSeaFields};
use sea_query::{Condition, Expr, PostgresQueryBuilder, Query, TableRef};
use sea_query_binder::SqlxBinder;
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use uuid::Uuid;

// Trait for BMC configuration
pub trait DbBmc {
    const TABLE: &'static str;
}

// ============================================================================
// CREATE
// ============================================================================

pub async fn create<MC, E>(ctx: &Ctx, mm: &ModelManager, data: E) -> Result<Uuid>
where
    MC: DbBmc,
    E: HasSeaFields,
{
    let db = mm.db();
    let mut tx = db.begin().await?;

    // CRITICAL: Set user context for audit triggers
    set_user_context(&mut tx, ctx.user_id()).await?;

    let fields = data.not_none_sea_fields();
    let (columns, values) = fields.for_sea_insert();

    let table = TableRef::SchemaTable("public".into(), MC::TABLE.into());

    let mut query = Query::insert();
    query
        .into_table(table)
        .columns(columns)
        .values(values)?
        .returning(Query::returning().column(Expr::col("id")));

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    let (id,): (Uuid,) = sqlx::query_as_with(&sql, values)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(id)
}

pub async fn create_many<MC, E>(
    ctx: &Ctx,
    mm: &ModelManager,
    data: Vec<E>,
) -> Result<Vec<Uuid>>
where
    MC: DbBmc,
    E: HasSeaFields,
{
    let db = mm.db();
    let mut tx = db.begin().await?;

    // CRITICAL: Set user context for audit triggers
    set_user_context(&mut tx, ctx.user_id()).await?;

    let table = TableRef::SchemaTable("public".into(), MC::TABLE.into());
    let mut query = Query::insert();
    query.into_table(table);

    for item in data {
        let fields = item.not_none_sea_fields();
        let (columns, values) = fields.for_sea_insert();

        if query.is_empty_columns() {
            query.columns(columns);
        }
        query.values(values)?;
    }

    query.returning(Query::returning().column(Expr::col("id")));

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    let ids: Vec<(Uuid,)> = sqlx::query_as_with(&sql, values)
        .fetch_all(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(ids.into_iter().map(|(id,)| id).collect())
}

// ============================================================================
// READ
// ============================================================================

pub async fn get<MC, E>(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<E>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
{
    let db = mm.db();

    let sql = format!("SELECT * FROM {} WHERE id = $1", MC::TABLE);

    let entity = sqlx::query_as::<_, E>(&sql)
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })?;

    Ok(entity)
}

pub async fn list<MC, E, F>(
    ctx: &Ctx,
    mm: &ModelManager,
    filter: Option<Vec<F>>,
    list_options: Option<ListOptions>,
) -> Result<Vec<E>>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    F: Into<FilterGroups>,
{
    let db = mm.db();

    let mut query = Query::select();
    query.from(TableRef::SchemaTable("public".into(), MC::TABLE.into()));
    query.columns(vec![Expr::col("*")]);

    // Apply filters if provided
    if let Some(filter) = filter {
        let filter_groups: Vec<FilterGroups> = filter.into_iter().map(Into::into).collect();
        for fg in filter_groups {
            let condition = fg.into_sea_condition()?;
            query.cond_where(condition);
        }
    }

    // Apply list options (pagination, sorting)
    if let Some(opts) = list_options {
        if let Some(limit) = opts.limit {
            query.limit(limit);
        }
        if let Some(offset) = opts.offset {
            query.offset(offset);
        }
        // Add order_by support as needed
    }

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    let entities = sqlx::query_as_with::<_, E, _>(&sql, values)
        .fetch_all(db)
        .await?;

    Ok(entities)
}

// ============================================================================
// UPDATE
// ============================================================================

pub async fn update<MC, E>(
    ctx: &Ctx,
    mm: &ModelManager,
    id: Uuid,
    data: E,
) -> Result<()>
where
    MC: DbBmc,
    E: HasSeaFields,
{
    let db = mm.db();
    let mut tx = db.begin().await?;

    // CRITICAL: Set user context for audit triggers
    set_user_context(&mut tx, ctx.user_id()).await?;

    let fields = data.not_none_sea_fields();

    let table = TableRef::SchemaTable("public".into(), MC::TABLE.into());
    let mut query = Query::update();
    query.table(table);

    for field in fields.iter() {
        query.value(field.name.clone(), field.value.clone());
    }

    // Add updated_by and updated_at
    query.value(Expr::col("updated_by"), ctx.user_id());
    query.value(Expr::col("updated_at"), Expr::current_timestamp());

    query.and_where(Expr::col("id").eq(id));

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    let count = sqlx::query_with(&sql, values)
        .execute(&mut *tx)
        .await?
        .rows_affected();

    if count == 0 {
        return Err(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        });
    }

    tx.commit().await?;

    Ok(())
}

// ============================================================================
// DELETE
// ============================================================================

pub async fn delete<MC>(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()>
where
    MC: DbBmc,
{
    let db = mm.db();
    let mut tx = db.begin().await?;

    // CRITICAL: Set user context for audit triggers
    set_user_context(&mut tx, ctx.user_id()).await?;

    let sql = format!("DELETE FROM {} WHERE id = $1", MC::TABLE);

    let count = sqlx::query(&sql)
        .bind(id)
        .execute(&mut *tx)
        .await?
        .rows_affected();

    if count == 0 {
        return Err(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        });
    }

    tx.commit().await?;

    Ok(())
}

pub async fn delete_many<MC>(ctx: &Ctx, mm: &ModelManager, ids: Vec<Uuid>) -> Result<u64>
where
    MC: DbBmc,
{
    let db = mm.db();
    let mut tx = db.begin().await?;

    // CRITICAL: Set user context for audit triggers
    set_user_context(&mut tx, ctx.user_id()).await?;

    let placeholders: Vec<String> = (1..=ids.len())
        .map(|i| format!("${}", i))
        .collect();

    let sql = format!(
        "DELETE FROM {} WHERE id = ANY(ARRAY[{}]::UUID[])",
        MC::TABLE,
        placeholders.join(", ")
    );

    let mut query = sqlx::query(&sql);
    for id in ids {
        query = query.bind(id);
    }

    let count = query.execute(&mut *tx).await?.rows_affected();

    tx.commit().await?;

    Ok(count)
}
```

**Verification:**
```bash
cargo build --package lib-core
cargo test --package lib-core base::
```

**Deliverable:** Base CRUD layer with automatic user context setting

---

### Step 2.4: Update Model Definitions - Organization Example

**Duration:** 1 hour per model × 31 models = 31 hours

**File:** `crates/libs/lib-core/src/model/organization.rs`

```rust
// Replace existing implementation

use crate::ctx::Ctx;
use crate::model::base::{self, DbBmc};
use crate::model::{Error, ModelManager, Result};
use modql::field::{Fields, HasFields};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

// ============================================================================
// Entity
// ============================================================================

#[derive(Debug, Clone, Fields, FromRow, Serialize, Deserialize)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub organization_type: String,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub is_active: bool,

    // Audit fields (standardized)
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
}

// ============================================================================
// ForCreate
// ============================================================================

#[derive(Debug, Clone, Fields, Deserialize)]
pub struct OrganizationForCreate {
    pub name: String,
    pub organization_type: String,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
}

// ============================================================================
// ForUpdate
// ============================================================================

#[derive(Debug, Clone, Fields, Deserialize)]
pub struct OrganizationForUpdate {
    pub name: Option<String>,
    pub organization_type: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub is_active: Option<bool>,
}

// ============================================================================
// Filter
// ============================================================================

#[derive(Debug, Clone, Fields, Deserialize)]
pub struct OrganizationFilter {
    pub id: Option<Uuid>,
    pub name: Option<String>,
    pub organization_type: Option<String>,
    pub country: Option<String>,
    pub is_active: Option<bool>,
}

// ============================================================================
// BMC
// ============================================================================

pub struct OrganizationBmc;

impl DbBmc for OrganizationBmc {
    const TABLE: &'static str = "organizations";
}

impl OrganizationBmc {
    pub async fn create(
        ctx: &Ctx,
        mm: &ModelManager,
        org_c: OrganizationForCreate,
    ) -> Result<Uuid> {
        // Add audit fields before creation
        let mut fields = org_c.not_none_fields();
        fields.push(("created_by".to_string(), ctx.user_id().into()));
        fields.push(("created_at".to_string(), OffsetDateTime::now_utc().into()));
        fields.push(("updated_at".to_string(), OffsetDateTime::now_utc().into()));
        fields.push(("is_active".to_string(), true.into()));

        base::create::<Self, _>(ctx, mm, fields).await
    }

    pub async fn get(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<Organization> {
        base::get::<Self, _>(ctx, mm, id).await
    }

    pub async fn list(
        ctx: &Ctx,
        mm: &ModelManager,
        filter: Option<Vec<OrganizationFilter>>,
        list_options: Option<ListOptions>,
    ) -> Result<Vec<Organization>> {
        base::list::<Self, _, _>(ctx, mm, filter, list_options).await
    }

    pub async fn update(
        ctx: &Ctx,
        mm: &ModelManager,
        id: Uuid,
        org_u: OrganizationForUpdate,
    ) -> Result<()> {
        base::update::<Self, _>(ctx, mm, id, org_u).await
    }

    pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
        base::delete::<Self>(ctx, mm, id).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::_dev_utils;

    #[tokio::test]
    async fn test_create_org() {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();

        let org_c = OrganizationForCreate {
            name: "Test Org".to_string(),
            organization_type: "Hospital".to_string(),
            country: "US".to_string(),
            address: None,
            city: None,
            state: None,
            postal_code: None,
            phone: None,
            email: None,
            website: None,
        };

        let id = OrganizationBmc::create(&ctx, &mm, org_c).await.unwrap();

        let org = OrganizationBmc::get(&ctx, &mm, id).await.unwrap();
        assert_eq!(org.name, "Test Org");
        assert_eq!(org.created_by, ctx.user_id());
        assert!(org.updated_by.is_none());

        // Cleanup
        OrganizationBmc::delete(&ctx, &mm, id).await.unwrap();
    }
}
```

**Repeat for all 31 models:**
- User
- Case
- PatientInformation
- MessageHeader
- SafetyReportIdentification
- ... (all E2B entities)

**Strategy for mass migration:**
1. Create a template from Organization
2. Use find/replace for each model name
3. Adjust fields specific to each model
4. Keep audit fields consistent

**Deliverable:** All 31 models standardized with created_by/updated_by fields

---

### Step 2.5: Update Manual CRUD Implementations

**Duration:** 2 hours per model × 28 models = 56 hours

**File Example:** `crates/libs/lib-core/src/model/patient.rs`

```rust
// BEFORE (manual SQL):
pub async fn create(_ctx: &Ctx, mm: &ModelManager, data: PatientInformationForCreate) -> Result<Uuid> {
    let sql = format!(
        "INSERT INTO {} (case_id, patient_initials, sex, created_at, updated_at)
         VALUES ($1, $2, $3, now(), now())
         RETURNING id",
        Self::TABLE
    );
    // _ctx ignored!
}

// AFTER (using base layer):
impl PatientInformationBmc {
    pub async fn create(
        ctx: &Ctx,
        mm: &ModelManager,
        patient_c: PatientInformationForCreate,
    ) -> Result<Uuid> {
        // Add audit fields
        let mut fields = patient_c.not_none_fields();
        fields.push(("created_by".to_string(), ctx.user_id().into()));
        fields.push(("created_at".to_string(), OffsetDateTime::now_utc().into()));
        fields.push(("updated_at".to_string(), OffsetDateTime::now_utc().into()));

        base::create::<Self, _>(ctx, mm, fields).await
    }

    // Similar for update, delete, etc.
}
```

**Migration checklist for each model:**
- [ ] Remove manual SQL INSERT statements
- [ ] Remove manual SQL UPDATE statements
- [ ] Remove manual SQL DELETE statements
- [ ] Use `base::create()` with audit fields
- [ ] Use `base::update()` (automatically sets updated_by)
- [ ] Use `base::delete()`
- [ ] Update tests

**Deliverable:** All 28 manual CRUD models migrated to base layer

---

### Step 2.6: Update Error Types

**Duration:** 1 hour

**File:** `crates/libs/lib-core/src/model/error.rs`

```rust
// Add new error variants

#[derive(Debug, Serialize)]
pub enum Error {
    // ... existing variants ...

    InvalidUserId,
    UserContextNotSet,
    AuditTrailFailed { source: String },

    // ... other variants ...
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::InvalidUserId => write!(f, "Invalid user ID"),
            Error::UserContextNotSet => write!(f, "User context not set for operation"),
            Error::AuditTrailFailed { source } => {
                write!(f, "Audit trail logging failed: {}", source)
            }
            // ... other variants ...
        }
    }
}

impl std::error::Error for Error {}
```

**Deliverable:** Updated error types for audit system

---

### Step 2.7: Remove Old Utility Functions

**Duration:** 1 hour

**File:** `crates/libs/lib-core/src/model/base/utils.rs`

Remove old functions:
- `prep_fields_for_create()` - No longer needed (handled in each BMC)
- `prep_fields_for_update()` - No longer needed (handled in base_uuid)
- `add_timestamps_for_create()` - No longer needed
- `add_timestamps_for_update()` - No longer needed

**Deliverable:** Cleaned up utility module

---

## PHASE 3: Testing & Validation (Weeks 6-8)

### Step 3.1: Create Integration Tests

**Duration:** 1 week

**File:** `tests/integration/audit_trail_tests.rs`

```rust
use lib_core::ctx::Ctx;
use lib_core::model::{
    organization::{OrganizationBmc, OrganizationForCreate},
    patient::{PatientInformationBmc, PatientInformationForCreate},
    ModelManager,
};
use uuid::Uuid;

#[tokio::test]
async fn test_audit_trail_user_attribution() {
    let mm = init_test_db().await;

    // Create two different users
    let user1_id = create_test_user(&mm, "user1@test.com").await;
    let user2_id = create_test_user(&mm, "user2@test.com").await;

    let ctx1 = Ctx::new(user1_id).unwrap();
    let ctx2 = Ctx::new(user2_id).unwrap();

    // User 1 creates an organization
    let org_c = OrganizationForCreate {
        name: "Test Hospital".to_string(),
        organization_type: "Hospital".to_string(),
        country: "US".to_string(),
        // ... other fields
    };

    let org_id = OrganizationBmc::create(&ctx1, &mm, org_c).await.unwrap();

    // Verify created_by is user1
    let org = OrganizationBmc::get(&ctx1, &mm, org_id).await.unwrap();
    assert_eq!(org.created_by, user1_id);
    assert_eq!(org.updated_by, None);

    // Verify audit log has user1
    let audit_logs = get_audit_logs(&mm, "organizations", org_id).await;
    assert_eq!(audit_logs.len(), 1);
    assert_eq!(audit_logs[0].action, "CREATE");
    assert_eq!(audit_logs[0].user_id, user1_id);

    // User 2 updates the organization
    let org_u = OrganizationForUpdate {
        name: Some("Updated Hospital".to_string()),
        ..Default::default()
    };

    OrganizationBmc::update(&ctx2, &mm, org_id, org_u).await.unwrap();

    // Verify updated_by is user2
    let org = OrganizationBmc::get(&ctx2, &mm, org_id).await.unwrap();
    assert_eq!(org.created_by, user1_id); // Still user1
    assert_eq!(org.updated_by, Some(user2_id));

    // Verify audit log has user2 for update
    let audit_logs = get_audit_logs(&mm, "organizations", org_id).await;
    assert_eq!(audit_logs.len(), 2);
    assert_eq!(audit_logs[1].action, "UPDATE");
    assert_eq!(audit_logs[1].user_id, user2_id);

    // Verify old_values and new_values captured
    let update_log = &audit_logs[1];
    let old_name = update_log.old_values["name"].as_str().unwrap();
    let new_name = update_log.new_values["name"].as_str().unwrap();
    assert_eq!(old_name, "Test Hospital");
    assert_eq!(new_name, "Updated Hospital");

    // Cleanup
    OrganizationBmc::delete(&ctx1, &mm, org_id).await.unwrap();

    // Verify delete is logged
    let audit_logs = get_audit_logs(&mm, "organizations", org_id).await;
    assert_eq!(audit_logs.len(), 3);
    assert_eq!(audit_logs[2].action, "DELETE");
    assert_eq!(audit_logs[2].user_id, user1_id);
}

#[tokio::test]
async fn test_audit_trail_tamper_resistance() {
    let mm = init_test_db().await;
    let ctx = Ctx::root_ctx();

    // Create a record
    let org_c = OrganizationForCreate { /* ... */ };
    let org_id = OrganizationBmc::create(&ctx, &mm, org_c).await.unwrap();

    // Try to modify audit log (should fail due to RLS)
    let db = mm.db();
    let result = sqlx::query("UPDATE audit_logs SET user_id = $1 WHERE record_id = $2")
        .bind(Uuid::new_v4())
        .bind(org_id)
        .execute(db)
        .await;

    assert!(result.is_err() || result.unwrap().rows_affected() == 0);

    // Try to delete audit log (should fail due to RLS)
    let result = sqlx::query("DELETE FROM audit_logs WHERE record_id = $1")
        .bind(org_id)
        .execute(db)
        .await;

    assert!(result.is_err() || result.unwrap().rows_affected() == 0);

    // Verify audit log is still intact
    let audit_logs = get_audit_logs(&mm, "organizations", org_id).await;
    assert_eq!(audit_logs.len(), 1);
}

#[tokio::test]
async fn test_all_models_have_audit_fields() {
    let mm = init_test_db().await;

    // Query database schema to verify all tables have audit columns
    let db = mm.db();
    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT table_name
         FROM information_schema.columns
         WHERE table_schema = 'public'
           AND column_name IN ('created_by', 'updated_by')
         ORDER BY table_name"
    )
    .fetch_all(db)
    .await
    .unwrap();

    // Should be 31 tables (all BMCs)
    assert!(tables.len() >= 31, "Expected at least 31 tables with audit fields, found {}", tables.len());

    // Verify specific tables
    let table_names: Vec<String> = tables.into_iter().map(|(name,)| name).collect();
    assert!(table_names.contains(&"users".to_string()));
    assert!(table_names.contains(&"organizations".to_string()));
    assert!(table_names.contains(&"cases".to_string()));
    assert!(table_names.contains(&"patient_information".to_string()));
    assert!(table_names.contains(&"reactions".to_string()));
    assert!(table_names.contains(&"drug_information".to_string()));
    // ... check all 31
}

// Helper functions
async fn get_audit_logs(mm: &ModelManager, table_name: &str, record_id: Uuid) -> Vec<AuditLog> {
    let db = mm.db();
    sqlx::query_as(
        "SELECT * FROM audit_logs
         WHERE table_name = $1 AND record_id = $2
         ORDER BY created_at"
    )
    .bind(table_name)
    .bind(record_id)
    .fetch_all(db)
    .await
    .unwrap()
}
```

**Run tests:**
```bash
cargo test --test audit_trail_tests
```

**Deliverable:** Comprehensive integration test suite

---

### Step 3.2: Operational Qualification (OQ) Tests

**Duration:** 1 week

**File:** `docs/validation/OQ_Audit_Trail_Tests.md`

```markdown
# Operational Qualification (OQ) - Audit Trail System

## Test ID: OQ-AUDIT-001
**Title:** User Attribution - CREATE Operation
**Objective:** Verify user_id is captured in audit_logs for INSERT operations
**Prerequisite:** Database migrated, application deployed

**Test Steps:**
1. Login as User A (UUID: xxx)
2. Create a new Patient record via API
3. Query audit_logs table for the patient record_id
4. Verify user_id = User A UUID
5. Verify action = 'CREATE'
6. Verify new_values JSONB contains patient data

**Expected Result:**
- audit_logs.user_id matches User A UUID
- Timestamp within 1 second of operation

**Status:** [ ] PASS / [ ] FAIL
**Tested By:** ___________
**Date:** ___________

---

## Test ID: OQ-AUDIT-002
**Title:** User Attribution - UPDATE Operation
**Objective:** Verify user_id is captured for UPDATE operations

**Test Steps:**
1. Login as User B (UUID: yyy)
2. Update existing Patient record created by User A
3. Query audit_logs for UPDATE action
4. Verify user_id = User B UUID
5. Verify old_values contains original data
6. Verify new_values contains updated data

**Expected Result:**
- audit_logs shows UPDATE with User B UUID
- old_values != new_values
- Record's updated_by field = User B UUID

**Status:** [ ] PASS / [ ] FAIL
**Tested By:** ___________
**Date:** ___________

---

## Test ID: OQ-AUDIT-003
**Title:** User Attribution - DELETE Operation

... (continue for all test cases)

---

## Test ID: OQ-AUDIT-010
**Title:** Tamper Resistance - Unauthorized Modification

**Test Steps:**
1. Login as regular application user
2. Attempt to UPDATE audit_logs table via SQL
3. Attempt to DELETE from audit_logs table
4. Verify operations are blocked

**Expected Result:**
- UPDATE returns 0 rows affected or permission denied
- DELETE returns 0 rows affected or permission denied
- Audit logs remain unchanged

**Status:** [ ] PASS / [ ] FAIL
**Tested By:** ___________
**Date:** ___________
```

**Create 30+ test cases covering:**
- All CRUD operations
- All 31 models
- Edge cases (NULL values, concurrent operations)
- Performance (1000+ records)
- Security (RLS, tampering attempts)

**Deliverable:** Complete OQ test protocol with sign-off

---

### Step 3.3: Performance Testing

**Duration:** 3 days

**File:** `tests/performance/audit_trail_bench.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lib_core::ctx::Ctx;
use lib_core::model::organization::{OrganizationBmc, OrganizationForCreate};

fn benchmark_create_with_audit(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mm = rt.block_on(init_test_db());
    let ctx = Ctx::root_ctx();

    c.bench_function("create_organization_with_audit", |b| {
        b.iter(|| {
            rt.block_on(async {
                let org_c = OrganizationForCreate {
                    name: "Bench Test".to_string(),
                    organization_type: "Hospital".to_string(),
                    country: "US".to_string(),
                    // ...
                };

                let id = OrganizationBmc::create(&ctx, &mm, org_c).await.unwrap();
                OrganizationBmc::delete(&ctx, &mm, id).await.unwrap();
            });
        });
    });
}

fn benchmark_bulk_create(c: &mut Criterion) {
    // Test creating 1000 records with audit trail
    // Target: < 5 seconds for 1000 records
}

criterion_group!(benches, benchmark_create_with_audit, benchmark_bulk_create);
criterion_main!(benches);
```

**Performance targets:**
- Single CREATE: < 50ms
- Single UPDATE: < 50ms
- Bulk CREATE (1000): < 5s
- Audit log query (10,000 records): < 500ms

**Deliverable:** Performance benchmark results

---

## PHASE 4: Documentation & Deployment (Weeks 9-10)

### Step 4.1: System Validation Documentation

**Duration:** 1 week

**Files to create:**

1. **`docs/validation/URS_Audit_Trail.md`** - User Requirements Specification
   ```markdown
   # User Requirements Specification (URS)
   # Audit Trail System

   ## UR-001: User Attribution
   **Requirement:** The system SHALL record the user ID for all create, update, and delete operations.

   **Rationale:** 21 CFR 11.10(e) requires audit trails to record operator entries.

   **Verification:** OQ-AUDIT-001, OQ-AUDIT-002, OQ-AUDIT-003
   ```

2. **`docs/validation/FS_Audit_Trail.md`** - Functional Specification
3. **`docs/validation/DS_Audit_Trail.md`** - Design Specification
4. **`docs/validation/IQ_Audit_Trail.md`** - Installation Qualification
5. **`docs/validation/PQ_Audit_Trail.md`** - Performance Qualification
6. **`docs/validation/Traceability_Matrix.xlsx`** - URS → Tests mapping

**Deliverable:** Complete validation package

---

### Step 4.2: Standard Operating Procedures (SOPs)

**Duration:** 3 days

**Files to create:**

1. **`docs/sops/SOP_001_Audit_Trail_Review.md`**
   ```markdown
   # SOP-001: Audit Trail Review Procedure

   ## Purpose
   Define the process for periodic review of audit trails to ensure data integrity.

   ## Scope
   All audit_logs entries for safety database operations.

   ## Frequency
   - Monthly for routine operations
   - Immediately upon detection of anomaly

   ## Procedure
   1. Generate audit trail report for review period
   2. Review for unusual patterns (mass deletions, off-hours access)
   3. Verify user attributions are valid
   4. Document findings in audit review log
   5. Escalate anomalies to QA manager

   ## Responsibilities
   - Database Administrator: Execute queries
   - QA Manager: Review findings
   - Compliance Officer: Sign off
   ```

2. **`docs/sops/SOP_002_Data_Integrity_Incident.md`**
3. **`docs/sops/SOP_003_Audit_Log_Retention.md`**

**Deliverable:** Complete SOP package

---

### Step 4.3: Production Deployment

**Duration:** 2 days

**Checklist:**

```markdown
# Production Deployment Checklist

## Pre-Deployment
- [ ] All OQ tests passed (30/30)
- [ ] Performance benchmarks met
- [ ] Validation documents signed off
- [ ] SOPs approved and trained
- [ ] Database backup completed
- [ ] Rollback plan documented

## Deployment Steps
- [ ] Schedule maintenance window
- [ ] Notify users of downtime
- [ ] Execute database migrations (001-005)
- [ ] Deploy application code
- [ ] Run smoke tests
- [ ] Verify audit trail functionality
- [ ] Monitor for 24 hours

## Post-Deployment
- [ ] Verify all models logging to audit_logs
- [ ] Check performance metrics
- [ ] Review error logs
- [ ] Conduct user acceptance testing
- [ ] Archive deployment artifacts
- [ ] Update system documentation
```

**Deployment command:**
```bash
# Execute migrations in order
psql -f sql/migrations/audit_trail_migration/001_add_audit_columns.sql
psql -f sql/migrations/audit_trail_migration/002_remove_old_audit_columns.sql
psql -f sql/migrations/audit_trail_migration/003_audit_logs_rls.sql
psql -f sql/migrations/audit_trail_migration/004_user_context_functions.sql
psql -f sql/migrations/audit_trail_migration/005_update_audit_triggers.sql

# Deploy application
cargo build --release
systemctl restart e2br3-api
```

**Deliverable:** Successfully deployed production system

---

## Rollback Procedures

### Emergency Rollback Plan

**If deployment fails, execute rollback within 1 hour:**

**Step 1: Restore Database**
```bash
# Stop application
systemctl stop e2br3-api

# Restore from backup
pg_restore -h localhost -U e2br3_user -d e2br3_db \
  --clean --if-exists \
  backup_pre_audit_migration_*.dump

# Verify restore
psql -c "SELECT COUNT(*) FROM users"
```

**Step 2: Revert Application Code**
```bash
git checkout [previous_commit_hash]
cargo build --release
systemctl start e2br3-api
```

**Step 3: Verify System**
```bash
# Run health check
curl http://localhost:8080/health

# Verify database operations
psql -c "SELECT * FROM users LIMIT 1"
```

**Step 4: Document Incident**
- Root cause analysis
- Lessons learned
- Update deployment plan

---

## Risk Mitigation

### Risk Matrix

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Data loss during migration | Low | Critical | Full backup before migration |
| Downtime exceeds window | Medium | High | Practice run in staging environment |
| Performance degradation | Medium | Medium | Load testing beforevalidation |
| User context not set | Low | Critical | Comprehensive integration tests |
| RLS blocks legitimate operations | Low | High | Thorough RLS policy testing |

### Mitigation Strategies

1. **Staging Environment Testing**
   - Complete full migration in staging first
   - Run load tests with production-like data volume
   - Verify all edge cases

2. **Phased Rollout**
   - Week 1: Database migrations only (read-only changes)
   - Week 2: Deploy app code to subset of users
   - Week 3: Full production rollout

3. **Monitoring**
   - Set up alerts for failed audit trail writes
   - Monitor query performance
   - Track user_id NULL occurrences (should be 0)

---

## Compliance Checklist

### 21 CFR Part 11 Requirements

| Requirement | Implementation | Verified |
|-------------|----------------|----------|
| **11.10(e)** Secure, computer-generated audit trails | PostgreSQL triggers + RLS | [ ] |
| **11.10(e)** Time-stamped | created_at TIMESTAMPTZ | [ ] |
| **11.10(e)** Record operator entries | user_id UUID captured | [ ] |
| **11.10(e)** Record create/modify/delete | INSERT/UPDATE/DELETE triggers | [ ] |
| **11.10(e)** Changes don't obscure previous | old_values/new_values JSONB | [ ] |
| **11.10(e)** Retention period | Per safety data retention policy | [ ] |
| **11.10(e)** Available for review | audit_logs queryable | [ ] |

### EMA GVP Module VI Requirements

| Requirement | Implementation | Verified |
|-------------|----------------|----------|
| Traceability of data entered/modified | All 31 models have created_by/updated_by | [ ] |
| Dates and sources | Timestamps + user_id | [ ] |
| Quality management | SOPs in place | [ ] |

### ALCOA+ Principles

| Principle | Implementation | Verified |
|-----------|----------------|----------|
| Attributable | user_id UUID | [ ] |
| Legible | JSONB human-readable | [ ] |
| Contemporaneous | NOW() timestamps | [ ] |
| Original | PostgreSQL is source of truth | [ ] |
| Accurate | Database constraints | [ ] |
| Complete | All fields logged | [ ] |
| Consistent | Standardized across 31 models | [ ] |
| Enduring | Backup + retention policy | [ ] |
| Available | Query interface | [ ] |
| Traceable | audit_logs with old/new values | [ ] |

---

## Timeline Summary

| Phase | Duration | Key Deliverables |
|-------|----------|------------------|
| **Phase 1: Database** | 2 weeks | Migrations, RLS, triggers updated |
| **Phase 2: Application** | 3 weeks | All 31 models refactored, Ctx updated |
| **Phase 3: Testing** | 3 weeks | Integration tests, OQ, performance |
| **Phase 4: Documentation** | 2 weeks | Validation docs, SOPs, deployment |
| **TOTAL** | **10 weeks** | Production-ready audit system |

---

## Success Metrics

**Post-Implementation Verification:**

1. **100% User Attribution**
   ```sql
   -- Should return 0 rows
   SELECT COUNT(*) FROM audit_logs WHERE user_id IS NULL;
   ```

2. **All Models Covered**
   ```sql
   -- Should return 31+ tables
   SELECT COUNT(DISTINCT table_name) FROM audit_logs;
   ```

3. **Tamper Resistance**
   ```sql
   -- Should fail or return 0
   UPDATE audit_logs SET user_id = 'xxx' WHERE id = 1;
   ```

4. **Performance Targets Met**
   - CREATE operations: < 50ms p95
   - UPDATE operations: < 50ms p95
   - Audit log queries: < 500ms for 10k records

---

## Appendices

### Appendix A: SQL Migration Scripts

All migration scripts are located in:
```
sql/migrations/audit_trail_migration/
  001_add_audit_columns.sql
  002_remove_old_audit_columns.sql
  003_audit_logs_rls.sql
  004_user_context_functions.sql
  005_update_audit_triggers.sql
```

### Appendix B: Model Migration Checklist

For each of 31 models:
- [ ] Add created_by/updated_by to struct
- [ ] Update ForCreate to not include audit fields
- [ ] Update ForUpdate to not include audit fields
- [ ] Migrate create() to use base::create()
- [ ] Migrate update() to use base::update()
- [ ] Migrate delete() to use base::delete()
- [ ] Add/update tests
- [ ] Document in changelog

### Appendix C: References

**Regulatory:**
- 21 CFR Part 11: https://www.ecfr.gov/current/title-21/chapter-I/subchapter-A/part-11
- EMA GVP Module VI: https://www.ema.europa.eu/en/gvp-module-vi

**Technical:**
- PostgreSQL Row-Level Security: https://www.postgresql.org/docs/current/ddl-rowsecurity.html
- SQLx Documentation: https://docs.rs/sqlx/latest/sqlx/

---

## Document Control

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-14 | System Architect | Initial version |

**Approval:**

- [ ] Technical Lead: _________________ Date: _______
- [ ] QA Manager: _________________ Date: _______
- [ ] Compliance Officer: _________________ Date: _______

---

**END OF DOCUMENT**
