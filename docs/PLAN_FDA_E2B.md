# FDA E2B(R3) Production Readiness Plan

## Goal
Produce FDA‑compliant E2B(R3) XML with real‑world values, validated by XSD + FDA business rules.

## Phase 0 — Baseline & Guardrails
- Confirm target schema: FDA uses ICH E2B(R3) HL7 message `MCCI_IN200100UV01` (XML_1.0).
- Ensure `E2BR3_XSD_PATH` points to full schema set (multicacheschemas + coreschemas).
- Lock template version and export path to a single canonical template.

## Phase 1 — Required Field Audit & Mapping (FDA)
**Deliverable:** A mapping table of FDA‑required fields → DB column/API → exporter mapping → validation rule.

1) Enumerate FDA‑required sections and fields (N, C, D, E, F, G, H, plus sender/receiver).
   - Source of truth: FDA E2B(R3) Core and Regional Data Elements & Business Rules (Excel).
   - Regional guidance: FDA Regional Implementation Guide (August 2024).
   - ICH base IG: E2B(R3) ICSR Implementation Guide (FDA/ICH).
2) For each field, map:
   - DB location (table/column) or missing
   - API endpoint used to set it (or missing)
   - Exporter mapping (XPath) or missing
   - Validation status (XSD only / custom rules / missing)
3) Produce a gap list:
   - Fields not stored
   - Fields stored but not exported
   - Fields exported but not validated

## Phase 2 — Data Model & API Completion
**Deliverable:** DB schema + APIs capture all required fields.

1) Add missing DB columns / tables (with migrations).
2) Extend REST endpoints for data capture (create/update).
3) Add terminology enforcement (MedDRA/ISO/WHODrug) where required.

## Phase 3 — Exporter Completion
**Deliverable:** Exporter emits valid FDA XML with required values.

1) Fill all required fields from DB.
2) Use nullFlavor only when permitted.
3) Remove or omit optional sections when empty.
4) Add ordering to match XSD content model.

## Phase 4 — FDA Business Rule Validator
**Deliverable:** Validator rejects FDA‑invalid content before export.

1) Implement FDA‑specific rules in `validate_e2b_xml_rules`.
2) Add rules for required identifiers, dates, codes, and conditional requirements.
3) Add tests for each rule.

## Phase 5 — End‑to‑End Compliance Tests
**Deliverable:** Automated tests that pass FDA validator with real‑world samples.

1) Import a real‑world case set.
2) Export XML and run XSD + FDA rules.
3) CI test to prevent regressions.
4) Remove temporary `clamp_str` truncation once the pipeline passes and decide on long‑value handling (schema expansion vs strict validation).

---

## Current Status (as of 2026‑02‑04)
- XSD validation works when schemas mounted.
- Export pipeline works but still contains template placeholders unless replaced.
- Batch sender/receiver now injected; template updated for proper ordering.

## Immediate Next Actions
1) Build FDA required field checklist from the FDA Core/Regional Excel.
2) Map to DB/API/exporter.
3) Identify missing fields and prioritize.
4) After a green end‑to‑end run, remove `clamp_str` truncation and lock the policy (schema change vs validation error).

## Notes
- FDA validation is stricter than XSD alone.
- Template placeholders must be replaced with real values or removed.
