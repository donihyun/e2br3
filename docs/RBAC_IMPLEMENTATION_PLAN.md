# RBAC Implementation Plan for SafetyDB

**Date:** 2026-01-25
**Branch:** feat/RBAC
**Status:** In Progress

---

## Overview

This plan implements a two-layer Role-Based Access Control (RBAC) system:

1. **Application Layer** - Role-based feature access (what actions users can perform)
2. **Database Layer** - Row-Level Security for organization isolation (what data users can see)

---

## Current State

### Application Roles (in `users` table)
- `admin` - Full system access
- `manager` - Management-level access
- `user` - Regular user access
- `viewer` - Read-only access

### Database Roles (PostgreSQL)
- `e2br3_app_role` - API server connections
- `e2br3_auditor_role` - Audit log viewing

### Gaps
- [ ] Role not included in JWT token
- [ ] No role checking on most endpoints
- [ ] No organization isolation (RLS only on audit_logs)
- [ ] No permission framework

---

## Implementation Phases

### Phase 1: JWT & Context Enhancement
**Goal:** Include role in authentication flow

- [ ] 1.1 Add `role` field to `UserForAuth` struct
- [ ] 1.2 Include role in JWT token claims
- [ ] 1.3 Extract role in `mw_ctx_resolver` middleware
- [ ] 1.4 Add `role` and `organization_id` to `Ctx` struct
- [ ] 1.5 Update token validation to include role

### Phase 2: Permission System (PBAC)
**Goal:** Define granular permissions mapped to roles

- [ ] 2.1 Create permission enum for all resources/actions
- [ ] 2.2 Define role-to-permission mappings
- [ ] 2.3 Create `has_permission()` utility function
- [ ] 2.4 Create permission checking middleware/extractor

### Phase 3: Endpoint Protection
**Goal:** Apply RBAC to all REST endpoints

- [ ] 3.1 Create `RequireRole` extractor
- [ ] 3.2 Create `RequirePermission` extractor
- [ ] 3.3 Apply to user management endpoints
- [ ] 3.4 Apply to case management endpoints
- [ ] 3.5 Apply to organization endpoints
- [ ] 3.6 Apply to all other endpoints

### Phase 4: Database Row-Level Security (RLS)
**Goal:** Enforce organization isolation at database level

- [ ] 4.1 Create migration for RLS policies
- [ ] 4.2 Enable RLS on `cases` table
- [ ] 4.3 Enable RLS on `users` table
- [ ] 4.4 Enable RLS on all case-related tables
- [ ] 4.5 Update connection to set user context
- [ ] 4.6 Test RLS policies

### Phase 5: Testing & Validation
**Goal:** Ensure RBAC works correctly

- [ ] 5.1 Unit tests for permission checking
- [ ] 5.2 Integration tests for endpoint protection
- [ ] 5.3 Integration tests for RLS
- [ ] 5.4 Test role transitions

---

## Detailed Implementation

### Phase 1: JWT & Context Enhancement

#### 1.1 Update `UserForAuth` struct

**File:** `crates/libs/lib-core/src/model/user.rs`

```rust
// Before
pub struct UserForAuth {
    pub id: Uuid,
    pub username: String,
    pub pwd: Option<String>,
    pub pwd_salt: Uuid,
    pub token_salt: Uuid,
}

// After
pub struct UserForAuth {
    pub id: Uuid,
    pub username: String,
    pub pwd: Option<String>,
    pub pwd_salt: Uuid,
    pub token_salt: Uuid,
    pub role: String,           // ADD
    pub organization_id: Uuid,  // ADD
}
```

#### 1.2 Update JWT Token Claims

**File:** `crates/libs/lib-auth/src/token/mod.rs`

Add role and organization_id to token payload:
```rust
pub struct TokenClaims {
    pub sub: String,      // user_id
    pub exp: i64,
    pub iat: i64,
    pub role: String,           // ADD
    pub org_id: String,         // ADD
}
```

#### 1.3 Update Context Struct

**File:** `crates/libs/lib-core/src/ctx/mod.rs`

```rust
pub struct Ctx {
    user_id: Uuid,
    role: String,              // ADD
    organization_id: Uuid,     // ADD
}

impl Ctx {
    pub fn role(&self) -> &str { &self.role }
    pub fn organization_id(&self) -> Uuid { self.organization_id }

    pub fn is_admin(&self) -> bool { self.role == "admin" }
    pub fn is_manager(&self) -> bool { self.role == "manager" }
    pub fn is_viewer(&self) -> bool { self.role == "viewer" }
}
```

#### 1.4 Update Auth Middleware

**File:** `crates/libs/lib-web/src/middleware/mw_auth.rs`

Extract role from token and create enhanced Ctx.

---

### Phase 2: Permission System

#### 2.1 Permission Definitions

**File:** `crates/libs/lib-core/src/model/acs/mod.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Case,
    User,
    Organization,
    AuditLog,
    Drug,
    Reaction,
    Patient,
    Narrative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Create,
    Read,
    Update,
    Delete,
    List,
    Export,
    Import,
    Approve,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permission(pub Resource, pub Action);
```

#### 2.2 Role-Permission Mapping

```rust
pub fn role_permissions(role: &str) -> &'static [Permission] {
    match role {
        "admin" => &[
            // All permissions
            Permission(Resource::Case, Action::Create),
            Permission(Resource::Case, Action::Read),
            Permission(Resource::Case, Action::Update),
            Permission(Resource::Case, Action::Delete),
            Permission(Resource::Case, Action::List),
            Permission(Resource::Case, Action::Export),
            Permission(Resource::Case, Action::Import),
            Permission(Resource::User, Action::Create),
            Permission(Resource::User, Action::Read),
            Permission(Resource::User, Action::Update),
            Permission(Resource::User, Action::Delete),
            Permission(Resource::User, Action::List),
            Permission(Resource::Organization, Action::Create),
            Permission(Resource::Organization, Action::Read),
            Permission(Resource::Organization, Action::Update),
            Permission(Resource::Organization, Action::Delete),
            Permission(Resource::AuditLog, Action::Read),
            Permission(Resource::AuditLog, Action::List),
            // ... all other permissions
        ],
        "manager" => &[
            // Case management + read users
            Permission(Resource::Case, Action::Create),
            Permission(Resource::Case, Action::Read),
            Permission(Resource::Case, Action::Update),
            Permission(Resource::Case, Action::Delete),
            Permission(Resource::Case, Action::List),
            Permission(Resource::Case, Action::Export),
            Permission(Resource::Case, Action::Import),
            Permission(Resource::Case, Action::Approve),
            Permission(Resource::User, Action::Read),
            Permission(Resource::User, Action::List),
            // ... nested case resources
        ],
        "user" => &[
            // Case CRUD, no user management
            Permission(Resource::Case, Action::Create),
            Permission(Resource::Case, Action::Read),
            Permission(Resource::Case, Action::Update),
            Permission(Resource::Case, Action::List),
            Permission(Resource::Case, Action::Export),
            // ... nested case resources (drugs, reactions, etc.)
        ],
        "viewer" => &[
            // Read-only
            Permission(Resource::Case, Action::Read),
            Permission(Resource::Case, Action::List),
            Permission(Resource::User, Action::Read),
            // ... read permissions only
        ],
        _ => &[],
    }
}

pub fn has_permission(role: &str, permission: Permission) -> bool {
    role_permissions(role).contains(&permission)
}
```

---

### Phase 3: Endpoint Protection

#### 3.1 RequirePermission Extractor

**File:** `crates/libs/lib-web/src/middleware/mw_permission.rs`

```rust
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};

pub struct RequirePermission<const R: u8, const A: u8>;

#[async_trait]
impl<S, const R: u8, const A: u8> FromRequestParts<S> for RequirePermission<R, A>
where
    S: Send + Sync,
{
    type Rejection = WebError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ctx = parts.extensions.get::<Ctx>()
            .ok_or(WebError::AuthRequired)?;

        let resource = Resource::from_u8(R);
        let action = Action::from_u8(A);
        let permission = Permission(resource, action);

        if !has_permission(ctx.role(), permission) {
            return Err(WebError::PermissionDenied {
                required: format!("{:?}.{:?}", resource, action)
            });
        }

        Ok(Self)
    }
}
```

#### 3.2 Apply to Endpoints

```rust
// Example: Case endpoints
async fn create_case(
    ctx: CtxW,
    _perm: RequirePermission<{Resource::Case as u8}, {Action::Create as u8}>,
    // ...
) -> Result<Json<Case>> {
    // Handler code
}

// Or use a simpler macro approach
#[require_permission(Case, Create)]
async fn create_case(ctx: CtxW, ...) -> Result<Json<Case>> {
    // Handler code
}
```

---

### Phase 4: Database Row-Level Security

#### 4.1 RLS Migration

**File:** `sql/migrations/XXXX_add_rls_policies.sql`

```sql
-- ============================================
-- Row-Level Security for Organization Isolation
-- ============================================

-- Function to get current organization from session
CREATE OR REPLACE FUNCTION current_organization_id() RETURNS UUID AS $$
BEGIN
    RETURN NULLIF(current_setting('app.current_organization_id', true), '')::UUID;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to check if current user is admin
CREATE OR REPLACE FUNCTION is_current_user_admin() RETURNS BOOLEAN AS $$
BEGIN
    RETURN COALESCE(current_setting('app.current_user_role', true), '') = 'admin';
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================
-- Cases Table RLS
-- ============================================
ALTER TABLE cases ENABLE ROW LEVEL SECURITY;

-- Policy: Users can only see cases in their organization
CREATE POLICY cases_org_isolation ON cases
    FOR ALL
    TO e2br3_app_role
    USING (
        organization_id = current_organization_id()
        OR is_current_user_admin()
    )
    WITH CHECK (
        organization_id = current_organization_id()
        OR is_current_user_admin()
    );

-- ============================================
-- Users Table RLS
-- ============================================
ALTER TABLE users ENABLE ROW LEVEL SECURITY;

-- Policy: Users can see users in their organization (or admins see all)
CREATE POLICY users_org_isolation ON users
    FOR SELECT
    TO e2br3_app_role
    USING (
        organization_id = current_organization_id()
        OR is_current_user_admin()
    );

-- Policy: Only admins can create/update/delete users
CREATE POLICY users_admin_modify ON users
    FOR ALL
    TO e2br3_app_role
    USING (is_current_user_admin())
    WITH CHECK (is_current_user_admin());

-- ============================================
-- Case-Related Tables RLS
-- ============================================

-- Patients
ALTER TABLE patients ENABLE ROW LEVEL SECURITY;
CREATE POLICY patients_via_case ON patients
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = patients.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- Drugs
ALTER TABLE drugs ENABLE ROW LEVEL SECURITY;
CREATE POLICY drugs_via_case ON drugs
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = drugs.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- Reactions
ALTER TABLE reactions ENABLE ROW LEVEL SECURITY;
CREATE POLICY reactions_via_case ON reactions
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = reactions.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- Test Results
ALTER TABLE test_results ENABLE ROW LEVEL SECURITY;
CREATE POLICY test_results_via_case ON test_results
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = test_results.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- Narratives
ALTER TABLE narratives ENABLE ROW LEVEL SECURITY;
CREATE POLICY narratives_via_case ON narratives
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = narratives.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- Message Headers
ALTER TABLE message_headers ENABLE ROW LEVEL SECURITY;
CREATE POLICY message_headers_via_case ON message_headers
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = message_headers.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- Safety Reports
ALTER TABLE safety_reports ENABLE ROW LEVEL SECURITY;
CREATE POLICY safety_reports_via_case ON safety_reports
    FOR ALL TO e2br3_app_role
    USING (
        EXISTS (
            SELECT 1 FROM cases c
            WHERE c.id = safety_reports.case_id
            AND (c.organization_id = current_organization_id() OR is_current_user_admin())
        )
    );

-- ============================================
-- Organizations Table RLS
-- ============================================
ALTER TABLE organizations ENABLE ROW LEVEL SECURITY;

-- Users can see their own organization
CREATE POLICY orgs_own ON organizations
    FOR SELECT TO e2br3_app_role
    USING (
        id = current_organization_id()
        OR is_current_user_admin()
    );

-- Only admins can modify organizations
CREATE POLICY orgs_admin_modify ON organizations
    FOR ALL TO e2br3_app_role
    USING (is_current_user_admin())
    WITH CHECK (is_current_user_admin());

-- ============================================
-- Grant permissions to app role
-- ============================================
GRANT EXECUTE ON FUNCTION current_organization_id() TO e2br3_app_role;
GRANT EXECUTE ON FUNCTION is_current_user_admin() TO e2br3_app_role;
```

#### 4.2 Set Context on Each Request

**File:** `crates/libs/lib-core/src/model/base.rs` (or connection setup)

```rust
// Before each database operation, set the session context
pub async fn set_user_context(
    db: &Pool<Postgres>,
    user_id: Uuid,
    organization_id: Uuid,
    role: &str,
) -> Result<()> {
    sqlx::query("SELECT set_config('app.current_user_id', $1, true)")
        .bind(user_id.to_string())
        .execute(db)
        .await?;

    sqlx::query("SELECT set_config('app.current_organization_id', $1, true)")
        .bind(organization_id.to_string())
        .execute(db)
        .await?;

    sqlx::query("SELECT set_config('app.current_user_role', $1, true)")
        .bind(role)
        .execute(db)
        .await?;

    Ok(())
}
```

---

## Permission Matrix

| Resource | Action | Admin | Manager | User | Viewer |
|----------|--------|-------|---------|------|--------|
| **Case** | Create | ✅ | ✅ | ✅ | ❌ |
| | Read | ✅ | ✅ | ✅ | ✅ |
| | Update | ✅ | ✅ | ✅ | ❌ |
| | Delete | ✅ | ✅ | ❌ | ❌ |
| | List | ✅ | ✅ | ✅ | ✅ |
| | Export | ✅ | ✅ | ✅ | ✅ |
| | Import | ✅ | ✅ | ❌ | ❌ |
| | Approve | ✅ | ✅ | ❌ | ❌ |
| **User** | Create | ✅ | ❌ | ❌ | ❌ |
| | Read | ✅ | ✅ | ✅ | ✅ |
| | Update | ✅ | ❌ | ❌ | ❌ |
| | Delete | ✅ | ❌ | ❌ | ❌ |
| | List | ✅ | ✅ | ✅ | ✅ |
| **Organization** | Create | ✅ | ❌ | ❌ | ❌ |
| | Read | ✅ | ✅ | ✅ | ✅ |
| | Update | ✅ | ❌ | ❌ | ❌ |
| | Delete | ✅ | ❌ | ❌ | ❌ |
| **AuditLog** | Read | ✅ | ✅ | ❌ | ❌ |
| | List | ✅ | ✅ | ❌ | ❌ |
| **Drug** | CRUD | ✅ | ✅ | ✅ | Read only |
| **Reaction** | CRUD | ✅ | ✅ | ✅ | Read only |
| **Patient** | CRUD | ✅ | ✅ | ✅ | Read only |
| **Narrative** | CRUD | ✅ | ✅ | ✅ | Read only |

---

## Files to Modify

### Phase 1
- [ ] `crates/libs/lib-core/src/model/user.rs`
- [ ] `crates/libs/lib-core/src/ctx/mod.rs`
- [ ] `crates/libs/lib-auth/src/token/mod.rs`
- [ ] `crates/libs/lib-web/src/middleware/mw_auth.rs`

### Phase 2
- [ ] `crates/libs/lib-core/src/model/acs/mod.rs` (implement PBAC)
- [ ] `crates/libs/lib-core/src/model/acs/permission.rs` (new file)
- [ ] `crates/libs/lib-core/src/model/acs/role.rs` (new file)

### Phase 3
- [ ] `crates/libs/lib-web/src/middleware/mw_permission.rs` (new file)
- [ ] `crates/libs/lib-web/src/middleware/mod.rs`
- [ ] `crates/services/web-server/src/web/rest/user_rest.rs`
- [ ] `crates/services/web-server/src/web/rest/case_rest.rs`
- [ ] `crates/services/web-server/src/web/rest/organization_rest.rs`
- [ ] All other `*_rest.rs` files

### Phase 4
- [ ] `sql/migrations/XXXX_add_rls_policies.sql` (new file)
- [ ] `crates/libs/lib-core/src/model/store/mod.rs` (set context)

---

## Testing Checklist

### Application RBAC Tests
- [ ] Admin can perform all operations
- [ ] Manager can manage cases but not users
- [ ] User can CRUD cases but not delete
- [ ] Viewer can only read
- [ ] Unauthenticated requests are rejected
- [ ] Invalid role returns error

### RLS Tests
- [ ] User A cannot see User B's organization cases
- [ ] Admin can see all organizations
- [ ] Creating case in wrong org fails
- [ ] Nested resources respect parent case org

---

## Rollback Plan

If issues arise:
1. Disable RLS: `ALTER TABLE <table> DISABLE ROW LEVEL SECURITY;`
2. Revert middleware changes
3. Remove permission checks from endpoints

---

## Progress Tracking

| Phase | Status | Started | Completed |
|-------|--------|---------|-----------|
| Phase 1 | ✅ Complete | 2026-01-25 | 2026-01-25 |
| Phase 2 | ✅ Complete | 2026-01-25 | 2026-01-25 |
| Phase 3 | ✅ Complete | 2026-01-25 | 2026-01-25 |
| Phase 4 | ✅ Complete | 2026-01-25 | 2026-01-25 |
| Phase 5 | ⏳ Pending (Testing) | | |

---

## Implementation Summary

### Completed Changes

**Phase 1: JWT & Context Enhancement**
- Updated `UserForAuth` to include `role` field
- Updated `UserForLogin` to include `role` field
- Expanded `Ctx` struct with `organization_id` and `role` fields
- Added role helper methods to `Ctx` (`is_admin()`, `is_manager()`, etc.)
- Updated auth middleware to pass role and org to context

**Phase 2: Permission System (PBAC)**
- Created `lib-core/src/model/acs/permission.rs` with full permission framework
- Defined `Resource`, `Action`, and `Permission` types
- Implemented role-to-permission mappings for all 4 roles
- Created permission constants for all resources

**Phase 3: Endpoint Protection**
- Created `lib-web/src/middleware/mw_permission.rs` with extractors
- Added `PermissionDenied` and `OrganizationAccessDenied` error types
- Updated `user_rest.rs` with permission checks (admin-only for CUD operations)
- Updated `audit_rest.rs` to use new permission system
- Added `/api/users/me` endpoint for current user profile

**Phase 4: Database Row-Level Security**
- Added RLS context functions to `03-safetydb-schema.sql`:
  - `current_organization_id()` - gets current org from session
  - `is_current_user_admin()` - checks if current user is admin
  - `set_org_context()` - sets org and role for session
- Added RLS policies for core tables in `03-safetydb-schema.sql`:
  - cases, case_versions, users, organizations
- Added RLS policies for case-related tables in `12-triggers.sql`:
  - patient_information, drug_information, reactions, test_results
  - message_headers, safety_report_identification, narrative_information
  - drug_reaction_assessments, other_case_identifiers, linked_report_numbers
  - primary_sources, literature_references, study_information
  - sender_information, receiver_information
- Added `set_org_context()` functions in Rust code

### Files Modified
- `crates/libs/lib-core/src/ctx/mod.rs`
- `crates/libs/lib-core/src/model/user.rs`
- `crates/libs/lib-core/src/model/mod.rs`
- `crates/libs/lib-core/src/model/acs/mod.rs`
- `crates/libs/lib-core/src/model/acs/permission.rs` (new)
- `crates/libs/lib-core/src/model/store/mod.rs`
- `crates/libs/lib-web/src/middleware/mod.rs`
- `crates/libs/lib-web/src/middleware/mw_auth.rs`
- `crates/libs/lib-web/src/middleware/mw_permission.rs` (new)
- `crates/libs/lib-web/src/error.rs`
- `crates/libs/lib-web/src/handlers/handlers_login.rs`
- `crates/services/web-server/src/web/rest/audit_rest.rs`
- `crates/services/web-server/src/web/rest/user_rest.rs`
- `crates/services/web-server/src/web/rest/mod.rs`
- `docs/dev_initial/03-safetydb-schema.sql` (RLS for core tables)
- `docs/dev_initial/12-triggers.sql` (RLS for case-related tables)
- Test files updated for new Ctx signature

---

*Last Updated: 2026-01-25*
