# E2BR3 SafetyDB Production Readiness Plan

Complete roadmap for bringing the E2BR3 SafetyDB system (Next.js frontend + Rust backend) to production.

---

## Project Status

### Backend (Rust/Axum) - ✅ COMPLETE

The backend is fully implemented and ready for frontend integration.

**Completed Features:**
- Axum web framework with full REST API
- PostgreSQL database with 28+ tables (all 210 E2B(R3) data elements)
- JWT authentication with httpOnly cookies (15 min expiration)
- RBAC permission system (4 roles: admin, manager, user, viewer)
- Row-Level Security (RLS) for organization isolation
- Complete audit trail with database triggers
- Full CRUD APIs for all E2B(R3) entities
- Nested resource APIs (drugs, reactions, test results, etc.)
- Terminology search APIs (MedDRA, WHODrug, Countries, Code Lists)

### Frontend (Next.js 14) - ⏳ NEEDS BACKEND INTEGRATION

**Completed:**
- Multi-step form wizard with all 8 E2B(R3) sections
- 7 reusable form components (production-ready)
- Complete type system (all 210 E2B(R3) data elements)
- Validation schemas for all sections
- Authentication flow (ready for backend)
- API client (ready for backend)

**Remaining Work:**
- Connect forms to backend APIs
- Connect terminology autocompletes to backend
- Test full end-to-end flow

---

## API Reference (For Frontend Integration)

**Base URL**: `http://localhost:8080`

### Authentication APIs

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `POST` | `/auth/v1/login` | Login with email/password | No |
| `POST` | `/auth/v1/logoff` | Logout and clear session | Yes |
| `POST` | `/auth/v1/refresh` | Refresh authentication token | Yes |

#### `POST /auth/v1/login`
```json
// Request
{ "email": "user@example.com", "pwd": "password123" }

// Response (200 OK) - Sets httpOnly cookie
{ "result": { "success": true } }
```

#### `POST /auth/v1/logoff`
```json
// Request
{ "logoff": true }

// Response (200 OK)
{ "result": { "logged_off": true } }
```

#### `POST /auth/v1/refresh`
```json
// Response (200 OK) - Refreshes httpOnly cookie
{ "data": { "expiresAt": "2026-01-26T15:30:00Z" } }
```

---

### User APIs

| Method | Endpoint | Description | Permission |
|--------|----------|-------------|------------|
| `GET` | `/api/users/me` | Get current user profile | Any authenticated |
| `GET` | `/api/users` | List users | User.List |
| `GET` | `/api/users/{id}` | Get user by ID | User.Read |
| `POST` | `/api/users` | Create user | User.Create (admin) |
| `PUT` | `/api/users/{id}` | Update user | User.Update (admin) |
| `DELETE` | `/api/users/{id}` | Delete user | User.Delete (admin) |

```json
// Response format
{
  "data": {
    "id": "uuid",
    "email": "user@example.com",
    "username": "johndoe",
    "role": "user",
    "organization_id": "uuid"
  }
}
```

---

### Case APIs (E2B(R3) Safety Reports)

| Method | Endpoint | Description | Permission |
|--------|----------|-------------|------------|
| `GET` | `/api/cases` | List cases | Case.List |
| `GET` | `/api/cases/{id}` | Get case | Case.Read |
| `POST` | `/api/cases` | Create case | Case.Create |
| `PUT` | `/api/cases/{id}` | Update case | Case.Update |
| `DELETE` | `/api/cases/{id}` | Delete case | Case.Delete (manager+) |

**Query Parameters:**
```
?filters[status]=draft
&list_options[limit]=20
&list_options[offset]=0
&list_options[order_by]=ctime
&list_options[order_dir]=desc
```

---

### Case Subresource APIs

All nested under `/api/cases/{case_id}/...`

| Resource | Type | Endpoint | E2B Section |
|----------|------|----------|-------------|
| Patient | Singleton | `/patient` | Section D |
| Reactions | Collection | `/reactions` | Section E |
| Drugs | Collection | `/drugs` | Section G |
| Test Results | Collection | `/test-results` | Section F |
| Narrative | Singleton | `/narrative` | Section H |
| Message Header | Singleton | `/message-header` | Section N |
| Safety Report | Singleton | `/safety-report` | Section C |
| Receiver | Singleton | `/receiver` | Section A |
| Other Identifiers | Collection | `/other-identifiers` | C.1.9.r |
| Linked Reports | Collection | `/linked-reports` | C.1.10.r |
| Case Versions | Read-only | `/versions` | Audit |

**Drug Sub-resources** (nested under `/drugs/{drug_id}/...`):
| Resource | Endpoint | E2B Section |
|----------|----------|-------------|
| Reaction Assessments | `/reaction-assessments` | G.k.9.i |
| Recurrences | `/recurrences` | G.k.8.r |

**Parent Sub-resources** (nested under `/patient/parent/{parent_id}/...`):
| Resource | Endpoint | E2B Section |
|----------|----------|-------------|
| Medical History | `/medical-history` | D.10.7.1.r |
| Past Drug History | `/past-drugs` | D.10.8.r |

**CRUD Pattern for Collections:**
- `GET /api/cases/{case_id}/{resource}` - List
- `GET /api/cases/{case_id}/{resource}/{id}` - Get one
- `POST /api/cases/{case_id}/{resource}` - Create
- `PUT /api/cases/{case_id}/{resource}/{id}` - Update
- `DELETE /api/cases/{case_id}/{resource}/{id}` - Delete

**CRUD Pattern for Singletons:**
- `GET /api/cases/{case_id}/{resource}` - Get
- `POST /api/cases/{case_id}/{resource}` - Create
- `PUT /api/cases/{case_id}/{resource}` - Update
- `DELETE /api/cases/{case_id}/{resource}` - Delete

---

### Organization APIs

| Method | Endpoint | Description | Permission |
|--------|----------|-------------|------------|
| `GET` | `/api/organizations` | List | Org.List |
| `GET` | `/api/organizations/{id}` | Get | Org.Read |
| `POST` | `/api/organizations` | Create | Org.Create (admin) |
| `PUT` | `/api/organizations/{id}` | Update | Org.Update (admin) |
| `DELETE` | `/api/organizations/{id}` | Delete | Org.Delete (admin) |

---

### Terminology APIs

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/terminology/meddra?q={query}` | Search MedDRA terms |
| `GET` | `/api/terminology/whodrug?q={query}` | Search WHODrug products |
| `GET` | `/api/terminology/countries` | List ISO country codes |
| `GET` | `/api/terminology/code-lists?list={name}` | Get E2B(R3) code list |

**Available Code Lists:**
- `reaction_outcome` (E.i.7)
- `seriousness_criteria` (E.i.3.2)
- `drug_characterization` (G.k.1)
- `action_taken` (G.k.7)
- `dosage_form` (G.k.4.r.9.1)
- `route_of_administration` (G.k.4.r.10.1)
- `age_group` (D.2.3)
- `sex` (D.5)
- `report_type` (C.1.3)
- `sender_type` (C.3.1)

---

### Audit APIs

| Method | Endpoint | Description | Permission |
|--------|----------|-------------|------------|
| `GET` | `/api/audit-logs` | List all logs | AuditLog.List (admin/manager) |
| `GET` | `/api/audit-logs/by-record/{table}/{id}` | Logs for record | AuditLog.List |

---

### Response Formats

**Success:**
```json
{ "data": { ... } }       // Single item
{ "data": [ ... ] }       // List
```

**Error:**
```json
{
  "error": {
    "type": "PERMISSION_DENIED",
    "message": "User.Create permission required",
    "req_uuid": "uuid"
  }
}
```

**Error Types:**
| Type | HTTP | Description |
|------|------|-------------|
| `LOGIN_FAIL` | 401 | Invalid credentials |
| `AUTH_REQUIRED` | 401 | Not authenticated |
| `PERMISSION_DENIED` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `VALIDATION_ERROR` | 400 | Invalid request data |

---

### RBAC Permission Matrix

| Resource | Action | Admin | Manager | User | Viewer |
|----------|--------|:-----:|:-------:|:----:|:------:|
| **Case** | Create | ✅ | ✅ | ✅ | ❌ |
| | Read | ✅ | ✅ | ✅ | ✅ |
| | Update | ✅ | ✅ | ✅ | ❌ |
| | Delete | ✅ | ✅ | ❌ | ❌ |
| **User** | Create | ✅ | ❌ | ❌ | ❌ |
| | Read | ✅ | ✅ | ✅ | ✅ |
| | Update | ✅ | ❌ | ❌ | ❌ |
| | Delete | ✅ | ❌ | ❌ | ❌ |
| **Organization** | CRUD | ✅ | ❌ | ❌ | ❌ |
| **AuditLog** | Read/List | ✅ | ✅ | ❌ | ❌ |
| **Subresources** | CRUD | ✅ | ✅ | ✅ | Read |

**Note:** Row-Level Security (RLS) filters data by organization. Non-admins only see their org's data.

---

## Frontend Integration Checklist

### Phase 1: Authentication
- [ ] Login form → `POST /auth/v1/login`
- [ ] Token refresh (14-min intervals) → `POST /auth/v1/refresh`
- [ ] Logout → `POST /auth/v1/logoff`
- [ ] Get current user on load → `GET /api/users/me`

### Phase 2: Case Management
- [ ] Case list → `GET /api/cases`
- [ ] Create case → `POST /api/cases`
- [ ] Case detail → `GET /api/cases/{id}`
- [ ] Update case → `PUT /api/cases/{id}`
- [ ] Connect CaseFormWizard to APIs

### Phase 3: Form Sections
- [ ] Patient → `/api/cases/{id}/patient`
- [ ] Reactions → `/api/cases/{id}/reactions`
- [ ] Drugs → `/api/cases/{id}/drugs`
- [ ] Test Results → `/api/cases/{id}/test-results`
- [ ] Narrative → `/api/cases/{id}/narrative`
- [ ] Safety Report → `/api/cases/{id}/safety-report`
- [ ] Message Header → `/api/cases/{id}/message-header`

### Phase 4: Terminology
- [ ] MedDRA autocomplete → `GET /api/terminology/meddra?q=`
- [ ] WHODrug autocomplete → `GET /api/terminology/whodrug?q=`
- [ ] Country list → `GET /api/terminology/countries`
- [ ] Code lists → `GET /api/terminology/code-lists?list=`

### Phase 5: Admin Features
- [ ] User management (admin only)
- [ ] Organization management (admin only)
- [ ] Audit log viewer (admin/manager)

---

## Future Phases (Not Yet Implemented)

### XML Processing
- XML import/export endpoints
- XSD validation
- ICH example support

### Production Hardening
- CSRF protection
- Rate limiting
- Performance optimization
- Deployment configuration

---

*Last Updated: 2026-01-26*
