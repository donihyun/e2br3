# FDA Real Submission Gateway Implementation Plan (No Mock)

## Objective
Implement real FDA E2B(R3) submission from this backend to FDA ESG NextGen, with durable state tracking, ACK lifecycle handling, retries, and auditability.

This plan assumes:
- XML generation/validation already exists.
- We are adding real transport + operational workflow.

---

## Phase 0: Confirm FDA Transport Profile

1. Pick one transport for v1:
- Recommended: ESG NextGen API (not USP, not AS2).

2. Confirm operational credentials with FDA:
- Test and production account(s).
- Client credentials/certificates required by ESG NextGen API.
- Endpoint base URLs for test/prod.

3. Document external contract:
- Submit endpoint.
- Status endpoint.
- ACK retrieval endpoint.
- Auth/token acquisition flow.

Exit criteria:
- We can authenticate against ESG test endpoint with non-production credentials.

---

## Phase 1: Data Model for Real Submissions

Add durable tables (Postgres), not in-memory.

1. `case_submissions`
- `id` (uuid, pk)
- `case_id` (uuid, fk -> cases)
- `gateway` (text) e.g. `fda_esg_nextgen_api`
- `environment` (text) `test|prod`
- `remote_submission_id` (text, nullable until returned)
- `status` (text enum-like)
- `payload_sha256` (text)
- `xml_bytes` (int)
- `submitted_by` (uuid)
- `submitted_at` (timestamptz)
- `last_status_at` (timestamptz)
- `last_error` (text, nullable)
- `created_at`, `updated_at`

2. `submission_acks`
- `id` (uuid, pk)
- `submission_id` (uuid, fk -> case_submissions)
- `ack_level` (smallint: 1..4)
- `success` (bool)
- `ack_code` (text, nullable)
- `ack_message` (text, nullable)
- `raw_payload` (jsonb or text)
- `received_at` (timestamptz)
- unique(`submission_id`, `ack_level`, `ack_code`, `received_at`) for idempotency.

3. Add indexes:
- `case_submissions(case_id, submitted_at desc)`
- `case_submissions(status, updated_at)`
- `submission_acks(submission_id, ack_level, received_at desc)`

Exit criteria:
- Submission state survives restarts.
- ACK history is queryable per submission.

---

## Phase 2: Domain State Machine

Define explicit statuses:
- `queued`
- `submission_requested`
- `submitted_ack1_pending`
- `ack1_received`
- `ack2_received`
- `ack3_received`
- `ack4_received`
- `rejected`
- `failed_retryable`
- `failed_final`

Rules:
1. `rejected` is terminal.
2. `ack4_received` is terminal success.
3. Duplicate ACK ingestion must be idempotent.
4. Out-of-order ACKs are allowed but normalized to latest terminal meaning.

Exit criteria:
- State transitions are deterministic and covered by unit tests.

---

## Phase 3: ESG Client (Real HTTP Adapter)

Create module:
- `crates/services/web-server/src/submission/fda_esg_client.rs`

Responsibilities:
1. Obtain/refresh auth token.
2. Submit XML payload.
3. Poll submission status.
4. Pull ACK documents/messages.
5. Parse and normalize ACKs into internal model.

Implementation notes:
- Use `reqwest` with strict timeout/retry policy.
- Use exponential backoff with jitter for 429/5xx.
- Separate transient vs permanent errors.
- Persist every request/response metadata needed for audit.

Exit criteria:
- Can submit a real test case and receive remote submission id from ESG test.

---

## Phase 4: Secure Config and Secrets

Add config keys (env):
- `FDA_ESG_ENABLED` (`0|1`)
- `FDA_ESG_ENV` (`test|prod`)
- `FDA_ESG_BASE_URL`
- `FDA_ESG_AUTH_URL`
- `FDA_ESG_CLIENT_ID`
- `FDA_ESG_CLIENT_SECRET` (or cert/key references)
- `FDA_ESG_TIMEOUT_SECS`
- `FDA_ESG_MAX_RETRIES`
- `FDA_ESG_POLL_INTERVAL_SECS`

Security controls:
1. Never log secrets or full tokens.
2. Redact sensitive headers in logs.
3. Validate config at startup (fail fast).

Exit criteria:
- Service boots only with valid real-submission configuration when feature enabled.

---

## Phase 5: REST/API Contract

Replace mock-facing endpoints with real behavior.

1. `POST /api/cases/{id}/submissions/fda`
- Validates case readiness.
- Exports XML.
- Inserts `case_submissions` row in `queued`.
- Dispatches submission job.
- Returns submission record.

2. `GET /api/cases/{id}/submissions`
- Returns durable records from DB.

3. `GET /api/submissions/{id}`
- Returns one durable record + latest ACK summary.

4. Remove/disable mock endpoint:
- `POST /api/submissions/{id}/acks/mock`

Exit criteria:
- API no longer depends on in-memory mock storage.

---

## Phase 6: Background Workers

Implement async workers (in-process first, queue later if needed):

1. Submit worker
- Picks `queued|failed_retryable` submissions.
- Calls ESG submit.
- Updates remote id + status.

2. ACK poll worker
- Polls ESG for submissions not terminal.
- Ingests new ACKs.
- Advances state machine.

3. Retry worker
- Applies bounded retries for transient failures.
- Moves to `failed_final` after max attempts.

4. Reconciliation job
- Daily scan for long-running non-terminal submissions.
- Re-query ESG status and repair drift.

Exit criteria:
- End-to-end asynchronous lifecycle works without manual API calls after initial submit.

---

## Phase 7: Case Workflow Integration

Submission preconditions:
1. Case status must be `validated`.
2. Validation profile must be FDA (or explicit override policy).
3. XML validation must pass before queueing.

Post-submission updates:
1. Set case `status=submitted` when submission is accepted for transport.
2. Add new case status option `submission_rejected` or keep rejection on submission record only (choose one policy and document it).
3. Keep immutable snapshot hash for the submitted XML (`payload_sha256`).

Exit criteria:
- Case workflow and submission workflow are consistent and traceable.

---

## Phase 8: Observability and Audit

Add structured events:
- `submission.created`
- `submission.sent`
- `submission.ack.received`
- `submission.state.changed`
- `submission.retry.scheduled`
- `submission.failed.final`

Metrics:
- Submission success rate.
- Mean time to ACK1/ACK4.
- Retry count distribution.
- Rejection code frequency.

Audit:
- Who triggered submission.
- Which payload hash was sent.
- Exact ACK payload references.

Exit criteria:
- Operators can answer “what happened to submission X?” in minutes.

---

## Phase 9: Testing Strategy

1. Unit tests
- State transitions.
- ACK parser with real FDA ACK fixtures.
- Retry classification.

2. Integration tests (mock HTTP server at test layer only)
- Auth failure.
- Submit success.
- Poll + ACK ingestion.
- Duplicate ACK idempotency.

3. Staging/UAT (real ESG test account)
- Submit known-good FDA case.
- Verify ACK1->ACK4 lifecycle.
- Submit known-bad case and verify rejection handling.

Exit criteria:
- Automated tests pass and UAT evidence is captured.

---

## Phase 10: Rollout Plan

1. Feature flag OFF in production.
2. Deploy DB migrations + read paths.
3. Enable for internal org only (canary).
4. Monitor metrics/logs for 1 week.
5. Expand to all orgs.

Rollback:
- Disable `FDA_ESG_ENABLED`.
- Keep read-only visibility for existing records.
- No data loss in submission/ack tables.

---

## Concrete Task Breakdown (Execution Order)

1. Create DB migrations for `case_submissions` and `submission_acks`.
2. Add `lib-core` models + BMCs for new tables.
3. Implement submission state machine module + tests.
4. Implement real `fda_esg_client` with auth + submit + status + ack retrieval.
5. Replace in-memory store with DB persistence.
6. Replace mock ACK endpoint with real poll worker ingestion.
7. Wire REST handlers to DB-backed submission service.
8. Add config parsing/validation and secret redaction.
9. Add structured logs/metrics.
10. Run ESG test UAT and record evidence.
11. Enable production with feature flag/canary.

---

## Definition of Done

- No mock endpoints or in-memory submission state used in production path.
- A validated FDA case can be submitted to real ESG test endpoint from API.
- ACK lifecycle is persisted durably and visible via API.
- Retries, failures, and terminal states are deterministic and audited.
- Ops has dashboards/alerts for stuck or rejected submissions.
