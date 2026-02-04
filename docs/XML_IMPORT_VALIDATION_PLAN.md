# XML Import & Validation Plan

Goal: Add a safe, deterministic XML validation + import pipeline for E2B(R3) payloads, with clear error reporting, audit logging, and RBAC‑guarded endpoints. Store original XML in S3, parsed JSONB in Postgres, and create a case + case_version(1). Also support manual case creation via API (form‑filled JSON).

## Step 1: Scope & Data Mapping
- Target: E2B(R3) XML (confirm 3.0/3.1), support namespaces and canonical root.
- Define the minimal accepted document structure (root + required nodes).
- Create a mapping table from XML nodes → model fields (case, patient, drug, reactions, etc.).
- Import behavior: create new case + case_version(1); define conflict policy for duplicate case_id / safety_report_id.

## Step 2: XML Parsing Strategy
- Parser: `quick-xml` + `serde` (streaming preferred).
- Implement an internal XML DTO model that mirrors the schema.
- Enforce UTF‑8, entity limits, and max size.

## Step 3: Validation Layer
- **XSD validation only (for now)**:
  - bundle schemas, cache/compile,
  - validate namespaces + required fields before parsing.
- **Error policy**:
  - XSD errors → reject request with helpful errors (line/column if available).
  - No extra business‑rule validation yet.

## Step 4: Import Pipeline (Model Integration)
- Transactional importer:
  - begin txn,
  - set full RLS context (`set_full_context_dbx`),
  - insert case + case_version(1) + subresources,
  - commit or rollback.
- Persist:
  - original XML to S3,
  - parsed JSONB to Postgres.
- Ensure audit triggers fire for all inserts.

## Step 5: REST Endpoint Design
- Upload endpoint:
  - `POST /api/import/xml` (multipart file upload).
  - RBAC: `XML_IMPORT` (admin only).
- Validation endpoint:
  - `POST /api/import/xml/validate` (dry‑run).
- Manual creation endpoint:
  - `POST /api/cases` (form JSON to create case without XML).
- Return structured errors (line/column when possible).

## Step 6: Storage & Limits
- Store raw XML in S3 (object key stored in DB).
- Store parsed JSONB in Postgres.
- Implement size limits and streaming read.
- Log import job metadata (user_id, org_id, status, errors).

## Step 7: Tests
- Unit tests:
  - valid XML → DTO mapping,
  - invalid XML → XSD errors,
  - business rule errors.
- Integration tests:
  - upload 3 E2B(R3) samples → stored in S3 + JSONB + case + case_version,
  - failures return helpful errors,
  - list/search by case ID shows imported cases,
  - RBAC forbids non-admin.
- Regression tests for RLS and audit logs.

## Step 8: Observability & Ops
- Emit structured logs per import job.
- Optional: background queue for large imports.
- Add metrics (import duration, fail rate).

## Step 9: CLI or Admin Utilities (Optional)
- Local CLI for validating XML before upload.
- Admin-only list endpoint for import jobs/history.

## Step 10: Rollout
- Ship validation‑only endpoint first.
- Enable import with feature flag.
- Monitor logs and metrics.

---
## Compliance Checklist (Step-by-step to pass validator tests)

**Step 0 — Baseline plumbing (done)**
- XSD validation via `E2BR3_XSD_PATH`
- Root/ITSVersion/schemaLocation checks
- telecom tel/fax/mailto validation + nullFlavor rules
- start/end/duration checks
- test result structural checks (F.r.3.*)
- dosing structural checks (G.k.4.*)
- nullFlavor rules (required when missing + forbidden when value present)

**Step 1 — Add validator tests**
- Add `crates/libs/lib-core/tests/xml_validation.rs`
- Validate 2–3 XMLs from `~/Documents/6_Example Instances`
- Add one negative sample per rule group

**Step 2 — Expand nullFlavor rules (reference instance)**
- Add remaining nullFlavor checks from technical reference table
- Scope to exact XPath nodes in the reference instance

**Step 3 — Reaction rules**
- Reaction node (`code=29`) structural requirements
- Outcome (`code=27`) must include value or nullFlavor
- MedDRA reaction term must be present or nullFlavor

**Step 4 — Drug rules**
- Substance name nullFlavor (G.k.2.3.r.1)
- Dose + period value/unit (G.k.4.*)
- EffectiveTime combos (G.k.4.r.4/5/8)
- Route + dose form require text or nullFlavor

**Step 5 — Patient/report rules**
- Patient demographics required or nullFlavor
- Primary source fields required or nullFlavor
- Privacy/telecom rules per ICH tech info

**Step 6 — Tests on examples**
- All example XMLs must pass
- Add malformed XML per rule group to assert errors

**Step 7 — Import pipeline (optional for compliance tests)**
- Parse XML → JSONB → case + case_version
- Persist original XML to S3

---
### Implementation Order (Recommended)
1) XML DTO + parsing
2) XSD validation + business rules
3) Importer (transaction + S3 + JSONB + case + case_version)
4) API endpoints + RBAC (import + validate + manual case create)
5) Tests + hardening
