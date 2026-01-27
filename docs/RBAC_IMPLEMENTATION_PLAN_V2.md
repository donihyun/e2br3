# RBAC Implementation Plan (v2)

Goal: enforce RBAC consistently across **all** web endpoints (cases + all sub-resources), not just users/org.
This plan assumes RLS already enforces org isolation, and RBAC decides *who* can read/write within an org.

---

## 0) Inventory & Permission Matrix

Create a single source of truth for permissions (names + scope + verbs).

Recommended permission set:

- users.read / users.write
- organizations.read / organizations.write
- cases.read / cases.write
- patient.read / patient.write
- reactions.read / reactions.write
- drugs.read / drugs.write
- drug_active_substances.read / drug_active_substances.write
- drug_dosages.read / drug_dosages.write
- drug_indications.read / drug_indications.write
- drug_reaction_assessments.read / drug_reaction_assessments.write
- relatedness_assessments.read / relatedness_assessments.write
- test_results.read / test_results.write
- narrative.read / narrative.write
- sender_diagnoses.read / sender_diagnoses.write
- case_summaries.read / case_summaries.write
- message_header.read / message_header.write
- safety_report.read / safety_report.write
- sender_information.read / sender_information.write
- primary_sources.read / primary_sources.write
- literature_references.read / literature_references.write
- study_information.read / study_information.write
- study_registration_numbers.read / study_registration_numbers.write
- receiver.read / receiver.write
- case_identifiers.read / case_identifiers.write
- audit.read (admin only)
- terminology.read (read-only for all roles)

Define these in one place (e.g., `lib_core::model::acs::permission` or a new `permissions.rs`).

---

## 1) Create a Single RBAC Guard Helper

Add a helper in web layer, used by every REST handler:

- `require_permission(ctx: &Ctx, perm: Permission) -> Result<()>`
- This should map to your existing permission check logic (role -> perms)
- Return `Error::PermissionDenied { required_permission }`

Example signature:

```
pub fn require_permission(ctx: &Ctx, perm: Permission) -> Result<()> {
    if !ctx.perms().contains(&perm) {
        return Err(Error::PermissionDenied { required_permission: perm.to_string() });
    }
    Ok(())
}
```

---

## 2) Enforce RBAC in Handlers (Not in BMC)

RBAC belongs to the API layer. Add checks in handlers so:
- RLS stays data-layer enforcement
- RBAC is consistent across all endpoints

Patterns:

- For **GET/list** -> require `*.read`
- For **POST/PUT/DELETE** -> require `*.write`

Example:

```
require_permission(&ctx, Permission::PatientRead)?; // before get/list
require_permission(&ctx, Permission::PatientWrite)?; // before create/update/delete
```

---

## 3) Apply to All REST Modules

Add guards to these files:

Core:
- `web/rest/case_rest.rs`
- `web/rest/patient_rest.rs`
- `web/rest/reaction_rest.rs`
- `web/rest/drug_rest.rs`
- `web/rest/test_result_rest.rs`
- `web/rest/narrative_rest.rs`
- `web/rest/message_header_rest.rs`
- `web/rest/safety_report_rest.rs`
- `web/rest/receiver_rest.rs`
- `web/rest/case_identifiers_rest.rs`

Sub-resources:
- `web/rest/patient_sub_rest.rs`
- `web/rest/drug_sub_rest.rs`
- `web/rest/narrative_sub_rest.rs`
- `web/rest/safety_report_sub_rest.rs`
- `web/rest/relatedness_assessment_rest.rs`

System:
- `web/rest/user_rest.rs`
- `web/rest/organization_rest.rs`
- `web/rest/audit_rest.rs`
- `web/rest/terminology_rest.rs` (read-only)

---

## 4) Update Role Definitions

Make sure roles map to these permissions:

- Admin: all permissions
- Manager: read/write core case model; no user/org admin; audit read optional
- Viewer: read-only everything except audit

Add/adjust in `lib_core::model::acs::role` or equivalent.

---

## 5) Tests (Web Layer)

Add tests for every REST group:

Pattern:
- Admin can read/write
- Viewer can read only
- Viewer forbidden on write

Test files:
- `rbac_users.rs` (already exists)
- `rbac_cases.rs`
- `rbac_patient.rs`
- `rbac_drug.rs`
- `rbac_narrative.rs`
- `rbac_safety_report.rs`
- `rbac_subresources.rs`
- `rbac_audit.rs`

Keep tests small: create a case, create patient, then attempt a write with viewer → expect 403.

---

## 6) Remove Workarounds

Once RBAC is enforced everywhere, remove ad-hoc guards or test hacks:
- no manual `SET ROLE` in tests
- no special-case exceptions for specific endpoints

---

## 7) Final QA

- Run all web tests
- Verify admin can write; viewer cannot
- Verify RLS still isolates orgs
- Verify audit logging still works

---

## Suggested Order (to reduce churn)

1. Implement `require_permission` helper
2. Patch `case_rest` + core case subresources
3. Patch drug + reaction + test results
4. Patch narrative + safety report
5. Patch remaining sub-resources
6. Add tests

---

If you want, tell me which step you’re starting, and I’ll walk you through it in detail.
