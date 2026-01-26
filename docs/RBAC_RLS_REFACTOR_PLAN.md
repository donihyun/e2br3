# RBAC/RLS Refactor Plan

Goal: make RBAC+RLS reliable without per-test hacks, and keep the web-server replaceable.

## Problems to solve
- RLS context is connection-scoped; pooled queries can miss session vars.
- Auth lookup happens before DB context exists (login/refresh).
- REST errors from lib-rest-core were not mapped, causing 500s.
- Tests need heavy boilerplate (env vars, SET ROLE, raw SQL inserts, extra grants).

## Plan (step-by-step)

1) Centralize DB context per request
   - Enforce a request-scoped transaction for all HTTP routes that hit the DB.
   - In one place, set:
     - set_current_user_context(user_id)
     - set_org_context(org_id, role)
   - Apply the same pattern to login/refresh routes.

   Solves:
   - RLS context mismatch (request vs connection)
   - Random 403/500 from missing session vars

2) Centralize auth lookup
   - Implement a single helper for “lookup user by email for auth”.
   - It should open a short transaction, set app.auth_email, read user, and commit.
   - Remove ad-hoc set_config calls from random handlers/middleware.

   Solves:
   - Login/refresh failures due to RLS blocking users lookup

3) Normalize error mapping
   - Ensure lib-rest-core errors are mapped to client errors (400 vs 500).
   - Keep the mapping in mw_res_map only.

   Solves:
   - UUID not-found returning 500

4) Reduce test boilerplate
   - Create test helpers to:
     - seed org/user/case via a single DB setup transaction
     - set context for tests (single function)
   - Remove per-test SET ROLE, row_security, and raw SQL setup where possible.

   Solves:
   - Repeated env/role/context hacks in every test

5) Verify boundaries for web-server independence
   - Keep all HTTP wiring inside lib-web (or lib-web-app).
   - web-server binary only creates config + app + serve.

   Solves:
   - Allows replacing web-server without touching model/libs

## Deliverables
- Consistent DB context middleware for all web routes
- Single auth lookup helper with predictable RLS behavior
- Clean error mapping in mw_res_map
- Shared test helpers for web tests and RLS tests
- Web-server remains a thin binary

## Expected outcome
- No per-test manual SET ROLE / set_config boilerplate
- Login/refresh and RLS tests pass consistently
- Clear, minimal interface between web-server and libraries
