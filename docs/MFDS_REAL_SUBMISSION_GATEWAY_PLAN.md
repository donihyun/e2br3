# MFDS Real Submission Gateway Implementation Plan (No Mock)

## Objective
Implement real MFDS E2B(R3) submission from this backend with durable tracking, ACK/result handling, retries, and operational audit trails.

---

## Phase 0: Confirm MFDS Submission Channel Contract

1. Confirm actual MFDS transport channel used by your organization:
- MFDS portal/API gateway, AS2 gateway, or delegated provider endpoint.

2. Confirm environment details:
- test/UAT endpoint(s)
- production endpoint(s)
- auth method (OAuth2, client cert, VPN, IP allowlist, etc.)

3. Freeze protocol contract:
- submit API/message format
- status lookup API
- ACK/error payload format

Exit criteria:
- We can authenticate and perform a non-business test call in MFDS test/UAT.

---

## Phase 1: Durable Submission Schema

Create DB tables (shared shape with FDA for reuse):

1. `case_submissions`
- `id`, `case_id`, `authority` (`mfds`)
- `gateway`, `environment`
- `remote_submission_id`
- `status`
- `payload_sha256`, `xml_bytes`
- `submitted_by`, `submitted_at`
- `last_status_at`, `last_error`
- `created_at`, `updated_at`

2. `submission_acks`
- `id`, `submission_id`
- `ack_level` (or provider status stage)
- `success`, `ack_code`, `ack_message`
- `raw_payload`
- `received_at`

3. Add proper indexes and idempotency unique key(s).

Exit criteria:
- Submission and ACK state survive restarts and are queryable per case.

---

## Phase 2: MFDS State Machine

Define deterministic statuses:
- `queued`
- `submission_requested`
- `submitted_pending_ack`
- `ack_received`
- `accepted_final`
- `rejected`
- `failed_retryable`
- `failed_final`

Rules:
1. Treat rejection/final-failure as terminal.
2. Handle duplicate or delayed ACKs idempotently.
3. Track exact external status code transitions.

Exit criteria:
- Transition matrix is unit-tested.

---

## Phase 3: MFDS Adapter Implementation

Add real adapter module:
- `crates/services/web-server/src/submission/mfds_client.rs`

Responsibilities:
1. Auth/token/certificate handling.
2. Submit E2B(R3) payload.
3. Poll/fetch processing status and ACK.
4. Normalize external statuses to internal state machine.

Implementation requirements:
- strict timeouts
- exponential backoff + jitter
- transient/permanent error classification
- redacted structured logging

Exit criteria:
- Real MFDS test/UAT submission returns a remote id and later status.

---

## Phase 4: MFDS Profile and Validation Gate

Before queueing:
1. Case profile must be MFDS.
2. XML must pass XSD + MFDS-specific rules.
3. KR regional fields required by business context must be present.

After queueing:
1. Freeze payload hash for audit.
2. Mark case workflow status policy (`submitted` or authority-specific variant).

Exit criteria:
- Invalid MFDS cases are blocked before transport.

---

## Phase 5: REST API and Worker Flow

API:
1. `POST /api/cases/{id}/submissions/mfds`
2. `GET /api/cases/{id}/submissions`
3. `GET /api/submissions/{id}`

Workers:
1. submit worker for `queued`
2. status/ack poll worker for non-terminal submissions
3. retry worker
4. reconciliation worker for stuck records

Exit criteria:
- End-to-end async lifecycle works with no manual mock endpoints.

---

## Phase 6: Observability, Security, Operations

1. Emit events:
- `submission.created`
- `submission.sent`
- `submission.status.changed`
- `submission.rejected`
- `submission.failed.final`

2. Metrics:
- success rate
- time to final acceptance
- rejection code distribution
- retry volume

3. Security:
- secrets only in env/secret manager
- redact auth headers/tokens
- audit who initiated submission and payload hash

Exit criteria:
- Operators can diagnose any failed MFDS submission quickly.

---

## Phase 7: Test and Rollout

1. Unit tests: state machine + status mapping + idempotency.
2. Integration tests: adapter with simulated MFDS responses.
3. UAT: real MFDS test/UAT account with known good and known bad cases.
4. Canary rollout behind feature flag.

Rollback:
- disable `MFDS_SUBMISSION_ENABLED`
- keep read-only visibility of existing records

---

## Execution Checklist (Order)

1. DB migrations for shared submission tables.
2. Add `lib-core` models/BMC for submissions + ACKs.
3. Implement MFDS state machine + tests.
4. Implement real `mfds_client`.
5. Wire REST endpoints for MFDS.
6. Implement submit/poll/retry/reconcile workers.
7. Add metrics/logging/alerts.
8. Run UAT and collect evidence.
9. Enable production via canary.

---

## Definition of Done

- No mock storage/endpoints in MFDS production path.
- Valid MFDS case submits to real MFDS test/UAT and reaches final status.
- ACK/status history is durable, queryable, and audited.
- Retry and failure handling are deterministic.
