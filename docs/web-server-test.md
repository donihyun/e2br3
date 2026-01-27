# Web Server Test Plan

Goal: systematic, repeatable coverage of auth, RBAC, RLS, audit, error mapping,
and middleware behavior at the web layer.

Conventions
- All tests use `crates/services/web-server/tests/common` helpers.
- Seed data via `seed_org_with_users` and `seed_two_orgs_users_cases`.
- Use table-driven tests where possible (roles/actions).

## 1) Auth Flow
1. login_success_admin
2. login_wrong_password
3. login_unknown_email
4. refresh_success
5. refresh_missing_token
6. refresh_invalid_token
7. logoff_success

## 2) Middleware
1. require_auth_missing_cookie -> 403
2. ctx_resolve_sets_user_org_role (validate current user in response or via a small echo endpoint)
3. db_context_per_request_no_leakage (two sequential requests with different users)

## 3) RBAC (Permissions)
User endpoints:
1. admin_can_create_user -> 201
2. viewer_cannot_create_user -> 403
3. admin_can_update_user -> 200
4. viewer_cannot_update_user -> 403
5. admin_can_delete_user -> 204
6. viewer_cannot_delete_user -> 403

Case endpoints:
7. admin_can_create_case -> 201
8. viewer_cannot_create_case -> 403
9. admin_can_update_case -> 200
10. viewer_cannot_update_case -> 403
11. admin_can_delete_case -> 204
12. viewer_cannot_delete_case -> 403

## 4) RLS (Org Isolation)
Users:
1. list_users_filters_by_org
2. get_user_foreign_org -> 400
3. update_user_foreign_org -> 400/403
4. delete_user_foreign_org -> 400/403

Cases:
5. list_cases_filters_by_org
6. get_case_foreign_org -> 400
7. update_case_foreign_org -> 400/403
8. delete_case_foreign_org -> 400/403

Nested resources (sample):
9. list_patient_info_filters_by_org
10. get_patient_info_foreign_org -> 400

## 5) Audit Trail
1. case_create_write_audit_log
2. case_update_write_audit_log
3. case_delete_write_audit_log
4. audit_log_read_requires_auditor_or_admin

## 6) Error Mapping
1. bad_uuid_returns_400
2. not_found_returns_400
3. model_validation_error_returns_400 (if applicable)

## 7) Terminology (Read-Only)
1. meddra_search_success
2. whodrug_search_success
3. iso_country_list_success

Implementation Order (section by section)
1) Auth Flow
2) Middleware
3) RBAC (Users + Cases)
4) RLS (Users + Cases)
5) Audit Trail
6) Error Mapping
7) Terminology
