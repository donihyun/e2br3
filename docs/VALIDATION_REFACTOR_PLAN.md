# Validation Refactor Plan

## Goal
Align validation behavior across frontend/backend, reduce duplicated rule logic, and optionally harden export safety without breaking current workflows.

## Scope
- Single-pass import validation (remove duplicate parse-time validation).
- Shared ICH base validator reused by FDA/MFDS validators.
- Optional export-time XML validation guard behind env flag.
- Rule metadata in `xml/validate/rules.rs` as the source of truth for case-validator issue text/severity/section.
- Preserve existing API contracts and validation response format.

## Current Status
- Completed: single-pass import validation.
- Completed: FDA/MFDS compose from ICH base validator.
- Completed: optional export-time guard via `E2BR3_EXPORT_VALIDATE`.
- Completed: case validators emit issues via metadata lookup (`push_issue_by_code`), not duplicated hardcoded messages.
- Completed: import mapping hardening for HL7 v3 roundtrip reliability.
  - Message header import now fails fast on write errors (no swallowed errors).
  - Message date is normalized to DB format (`YYYYMMDDHHMMSS`) before persistence.
  - Message number collision handling added for import-created headers.
  - Patient initials extraction now supports `primaryRole/player1/name` and compact initials tokens.
  - Patient update errors during import are now propagated.
- Completed (partial): XML validator now resolves stable coded FDA/ICH checks through metadata (`find_validation_rule`) while keeping parser/XSD diagnostics direct.
- Completed (incremental): additional ICH XML checks now metadata-backed (`ICH.C.1.9.1.CONDITIONAL`, `ICH.E.i.4-6.CONDITIONAL`, `ICH.G.k.4.r.4-8.CONDITIONAL`).
- Completed (incremental): structural timing XML checks now metadata-backed (`ICH.XML.EFFECTIVETIME.WIDTH.REQUIRES_BOUND`, `ICH.XML.SXPR_TS.COMP.REQUIRED`, `ICH.XML.PIVL_TS.PERIOD.REQUIRED`, `ICH.XML.PIVL_TS.PERIOD.VALUE_UNIT.REQUIRED`, `ICH.XML.IVL_TS.OPERATOR_A.BOUND_REQUIRED`).
- Completed (incremental): additional structural checks now metadata-backed (`ICH.XML.DOSE_QUANTITY.VALUE_UNIT.REQUIRED`, `ICH.XML.PERIOD.VALUE_UNIT.REQUIRED`, `ICH.D.5.SEX.CONDITIONAL`).
- Completed (incremental): telecom + reaction nullFlavor structural checks now metadata-backed (`ICH.XML.TELECOM.*`, `ICH.E.i.2.NULLFLAVOR.*`, `ICH.E.i.7.NULLFLAVOR.*`).
- Completed (incremental): generic code/form/route structural checks now metadata-backed (`ICH.XML.CODE.NULLFLAVOR.*`, `ICH.G.k.4.r.10.NULLFLAVOR.REQUIRED`, `ICH.G.k.4.r.11.NULLFLAVOR.REQUIRED`).
- Completed (incremental): BL + reaction linkage/country structural checks now metadata-backed (`ICH.XML.BL.*`, `ICH.XML.INV_CHAR_BL.*`, `ICH.E.i.0.RELATIONSHIP.CODE.*`, `ICH.E.i.9.COUNTRY.NULLFLAVOR.REQUIRED`).
- Completed (incremental): person/organization/title/id nullFlavor structural checks now metadata-backed (`ICH.C.2.r.*.NULLFLAVOR.*`, `ICH.D.2.BIRTHTIME.NULLFLAVOR.*`, `ICH.D.PARENT.*.NULLFLAVOR.*`, `ICH.C.5.TITLE.NULLFLAVOR.*`, `ICH.G.k.9.i.2.ID.NULLFLAVOR.*`, `ICH.G.k.2.3.NAME.NULLFLAVOR.*`).
- Completed (incremental): generic text and low/high structural checks now metadata-backed (`ICH.XML.TEXT.NULLFLAVOR.*`, `ICH.XML.LOW_HIGH.NULLFLAVOR.*`, `ICH.E.i.4-5.LOW_HIGH.NULLFLAVOR.REQUIRED`, `ICH.G.k.4.r.4-5.LOW_HIGH.NULLFLAVOR.REQUIRED`, `ICH.D.EFFECTIVETIME.LOW_HIGH.NULLFLAVOR.REQUIRED`).
- Completed (incremental): shared Section-E policy module (`xml/validate/e_reaction_policy.rs`) now drives exporter defaults for `E.i.7` and `FDA.E.i.3.2h`, and FDA case-validator requirement gating for `FDA.E.i.3.2h`.
- Completed (incremental): shared Section-C/D/F/G/H policy modules introduced and wired for C/D/E/G/H validator parity paths (`xml/validate/*_policy.rs`).
- Completed (foundation): canonical rule catalog introduced for migrated parity slice (`xml/validate/catalog.rs`) with validation-metadata cross-check test.
- Completed (cutover): `VALIDATION_RULES` metadata moved into `xml/validate/catalog.rs`; legacy `xml/validate/rules.rs` module removed.
- Completed (incremental): canonical evaluator expanded to condition/value/presence helpers, with FDA/MFDS rule-gating/value checks progressively routed through catalog (`is_rule_condition_satisfied`, `is_rule_value_valid`, `is_rule_presence_valid`).
- Completed (incremental): exporter behavior now consumes catalog directives for key required/default/nullFlavor paths (`ICH.E.i.7`, `FDA.E.i.3.2h`, `ICH.G.k.1`, `FDA.C.1.7.1`, `FDA.C.1.12`).
- Completed (incremental): cross-profile parity matrix integration test added to assert validator/exporter behavior alignment from the same rule source (`tests/rule_source_parity_matrix.rs`).
- Completed: explicit `cases.validation_profile` support added (schema/model/import inference + profile resolution precedence).
- Verified: `cargo test -p web-server --test case_validation_web` passing.
  - Verified: `cargo test -p lib-core --test xml_validation` passing.
  - Verified: `cargo test -p lib-core --test case_crud --test audit_trail --test error_cases` passing.

## Step-by-Step Fix Plan

1. Baseline and Safety Checks
- Confirm current call paths for `import`, `case validation`, and `export`.
- Keep env-guarded behavior backward compatible.

2. Single-Pass Import Validation
- Keep validation in `import_e2b_xml` as the authoritative import gate.
- Remove validation call from `parse_e2b_xml` so parse step is parse-only.
- Ensure `E2BR3_SKIP_XML_VALIDATE` still bypasses import validation exactly once.

3. Share ICH Base Rules for FDA/MFDS
- Reuse ICH case-level checks by composing FDA/MFDS validators on top of ICH issues.
- Keep profile-specific FDA/MFDS rules in their own modules.
- Maintain output profile labels (`fda`, `mfds`) and issue format.

4. Optional Export-Time Guard
- Add env flag (planned: `E2BR3_EXPORT_VALIDATE`) to enable XML validation after export generation.
- When enabled, run XML validator before returning export response.
- If validation fails, return an error with summary context.
- Default remains disabled to avoid behavior changes in existing environments.

5. Regression Tests
- Run targeted tests for case validation and import/export paths.
- Verify no API signature changes.
- Verify FDA/MFDS profile behavior remains correct.
- Added regression assertions in `xml_import_export` to verify imported `message_header.message_number` and `patient.patient_initials` are present.

6. Source-of-Truth Expansion (Next Execution Phase)
- Expand metadata-driven issue generation from current coded XML checks to broader XML checks where stable rule codes exist.
- Minimize remaining hardcoded error strings in `xml/xml_validation.rs` by mapping additional checks to metadata.
- Keep transport/XSD parser diagnostics as direct messages when they do not map to rule catalog entries.

## Rollout Notes
- Start with env flag OFF in non-critical environments.
- Turn ON export guard in staging first, then production.
- Monitor validation failure rates for false positives.

## Follow-Up (Next Iteration)
- Remove frontend rule duplication that overlaps backend business rules.
- Gate submit/export with authoritative backend validation report + XML validation.
- Completed: frontend case flow now persists explicit `validation_profile` during create/update and initializes from loaded case value.
