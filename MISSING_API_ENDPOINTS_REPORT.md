# Missing API Endpoints Report

**Date:** 2026-01-18
**Analysis:** Comparison of Backend Implementation vs Frontend Requirements

---

## Executive Summary

The current backend implements **48 endpoints** across 11 categories. However, the frontend requires several additional endpoints for full E2B(R3) compliance and feature completeness. This report identifies **critical gaps** that must be addressed.

| Status | Category | Count |
|--------|----------|-------|
| 🔴 Critical | Missing core functionality | 2 |
| 🟠 High | Missing E2B(R3) entities | 12 |
---dddd

## 🔴 Critical Missing Endpoints

### 1. XML Export

**Frontend expects:** `GET /api/xml/export/:id`
**Backend has:** None

**Impact:** The `/app/dashboard/submission/page.tsx` submission page cannot export cases as E2B(R3) XML. This is a **core E2B(R3) requirement** - the entire purpose of the system is to generate compliant XML.

**Required implementation:**
```
GET /api/xml/export/{case_id}
Response: E2B(R3) compliant XML document
Content-Type: application/xml
```

---

### 2. XML Import

**Frontend expects:** `POST /api/xml/import`
**Backend has:** None

**Impact:** The `/app/dashboard/import/page.tsx` import page cannot import E2B(R3) XML files. Users cannot load external ICSR reports into the system.

**Required implementation:**
```
POST /api/xml/import
Content-Type: multipart/form-data
Request: { "file": <E2B(R3) XML file> }
Response: { "data": { "caseId": "uuid", "status": "imported" } }
```

---

### 3. Previously listed as missing but implemented

**Authentication - Token Refresh**
- Implemented as `POST /auth/v1/refresh` (cookie-based), not missing.

**MedDRA Terminology Search**
- Implemented as `GET /api/terminology/meddra?q={term}&limit={count}`.

**WHODrug Terminology Search**
- Implemented as `GET /api/terminology/whodrug?q={term}&limit={count}`.

---

## 🟠 High Priority - Missing E2B(R3) Entity Endpoints

Based on the E2B(R3) specification and frontend type system (`lib/types/e2br3.ts`), the following repeating section endpoints are missing:

### Section C - Case Identification

| Entity | E2B(R3) Section | Required Endpoint Pattern |
|--------|-----------------|---------------------------|
| Primary Sources | C.2.r | `/api/cases/{id}/primary-sources` |
| Literature References | C.4.r | `/api/cases/{id}/literature-references` |
| Study Registration | C.5.1.r | `/api/cases/{id}/study-registrations` |

**Primary Sources (C.2.r)** - Contains reporter information:
- C.2.r.1 Reporter's Name
- C.2.r.4 Qualification (1=Physician, 2=Pharmacist, etc.)
- C.2.r.5 Primary Source for Regulatory Purposes

---

### Section D - Patient Information

| Entity | E2B(R3) Section | Required Endpoint Pattern |
|--------|-----------------|---------------------------|
| Medical History | D.7.1.r | `/api/cases/{id}/patient/medical-history` |
| Past Drug Therapy | D.8.r | `/api/cases/{id}/patient/past-drugs` |
| Death Causes (Reported) | D.9.2.r | `/api/cases/{id}/patient/death-causes-reported` |
| Death Causes (Autopsy) | D.9.4.r | `/api/cases/{id}/patient/death-causes-autopsy` |
| Parent Information | D.10 | `/api/cases/{id}/patient/parent` |

---

### Section G - Drug Information (Nested)

| Entity | E2B(R3) Section | Required Endpoint Pattern |
|--------|-----------------|---------------------------|
| Active Substances | G.k.2.3.r | `/api/cases/{id}/drugs/{drug_id}/substances` |
| Dosage Regimens | G.k.4.r | `/api/cases/{id}/drugs/{drug_id}/dosages` |
| Drug Indications | G.k.6.r | `/api/cases/{id}/drugs/{drug_id}/indications` |
| Drug-Reaction Matrix | G.k.9.i | `/api/cases/{id}/drugs/{drug_id}/reaction-assessments` |

---

### Section H - Case Narrative

| Entity | E2B(R3) Section | Required Endpoint Pattern |
|--------|-----------------|---------------------------|
| Sender Diagnoses | H.3.r | `/api/cases/{id}/sender-diagnoses` |
| Case Summary | H.5.r | `/api/cases/{id}/case-summaries` |



## 🟢 Implemented Endpoints (Working)

| Category | Endpoints | Status |
|----------|-----------|--------|
| Cases | GET, POST, PUT, DELETE `/api/cases` | ✅ |
| Patient | GET, POST, PUT, DELETE `/api/cases/{id}/patient` | ✅ |
| Reactions | CRUD `/api/cases/{id}/reactions` | ✅ |
| Drugs | CRUD `/api/cases/{id}/drugs` | ✅ |
| Test Results | CRUD `/api/cases/{id}/test-results` | ✅ |
| Narrative | CRUD `/api/cases/{id}/narrative` | ✅ |
| Message Header | CRUD `/api/cases/{id}/message-header` | ✅ |
| Safety Report | CRUD `/api/cases/{id}/safety-report` | ✅ |
| Organizations | CRUD `/api/organizations` | ✅ |
| Users | CRUD `/api/users` | ✅ |

---

## Implementation Priority Recommendation

### Phase 1 - Critical (Blocks Core Functionality)

| Priority | Endpoint | Effort | Impact |
|----------|----------|--------|--------|
| P0 | `GET /api/xml/export/{id}` | High | E2B(R3) XML generation |
| P0 | `POST /api/xml/import` | High | E2B(R3) XML parsing |

### Phase 2 - E2B(R3) Compliance (Repeating Sections)

| Priority | Endpoint | E2B(R3) Section |
|----------|----------|-----------------|
| P1 | `/api/cases/{id}/primary-sources` | C.2.r |
| P1 | `/api/cases/{id}/drugs/{drug_id}/dosages` | G.k.4.r |
| P1 | `/api/cases/{id}/drugs/{drug_id}/indications` | G.k.6.r |
| P1 | `/api/cases/{id}/sender-diagnoses` | H.3.r |

### Phase 3 - Full Compliance (Optional Repeating Sections)

| Priority | Endpoint | E2B(R3) Section |
|----------|----------|-----------------|
| P2 | `/api/cases/{id}/literature-references` | C.4.r |
| P2 | `/api/cases/{id}/study-registrations` | C.5.1.r |
| P2 | `/api/cases/{id}/patient/medical-history` | D.7.1.r |
| P2 | `/api/cases/{id}/patient/past-drugs` | D.8.r |
| P2 | `/api/cases/{id}/patient/death-causes-*` | D.9.2.r, D.9.4.r |
| P2 | `/api/cases/{id}/drugs/{drug_id}/substances` | G.k.2.3.r |
| P2 | `/api/cases/{id}/case-summaries` | H.5.r |

---

## Summary Statistics

| Category | Count |
|----------|-------|
| Currently Implemented | 48 |
| Critical Missing | 5 |
| E2B(R3) Entity Endpoints Missing | 12 |
| Total Endpoints Needed | ~65 |
| Completion Rate | **74%** |

---

## Next Steps

1. **Immediate:** Implement token refresh endpoint to prevent session issues
2. **Week 1:** Implement terminology endpoints (MedDRA, WHODrug) - can use mock data initially
3. **Week 2:** Implement XML export/import - core E2B(R3) functionality
4. **Week 3-4:** Implement missing repeating section endpoints per Phase 2/3

---

*Report generated by analyzing `API_ENDPOINTS_REPORT.md` against frontend requirements in `CLAUDE.md` and `lib/types/e2br3.ts`*
