# Unified Submission Gateway Architecture (FDA + MFDS + ICH)

## Objective
Build one real submission platform in this backend that supports:
- FDA (ESG NextGen API)
- MFDS (KR gateway/API path)
- ICH receiver integrations (adapter-per-receiver)

No mock submission path in production flow.

---

## 1. Core Design Principle

Implement once, specialize per authority via adapters.

1. Shared core:
- submission persistence
- state machine
- queue/workers
- retry/reconciliation
- API and observability

2. Authority adapters:
- `fda_esg_adapter`
- `mfds_adapter`
- `ich_receiver_adapter::<receiver_code>`

---

## 2. Shared Domain Model

## 2.1 `case_submissions`
- `id` (uuid, pk)
- `case_id` (uuid, fk)
- `authority` (`fda|mfds|ich`)
- `receiver_code` (nullable; required for ICH receiver routing)
- `gateway` (text)
- `environment` (`test|prod`)
- `remote_submission_id` (nullable until accepted)
- `status` (internal state)
- `payload_sha256` (text)
- `xml_bytes` (int)
- `submitted_by` (uuid)
- `submitted_at` (timestamptz)
- `last_status_at` (timestamptz)
- `last_error` (text)
- `retry_count` (int)
- `next_retry_at` (timestamptz, nullable)
- `created_at`, `updated_at`

## 2.2 `submission_events`
- `id` (uuid, pk)
- `submission_id` (uuid, fk)
- `event_type` (`submit_request|submit_response|status_poll|ack_received|state_changed|retry_scheduled|failure`)
- `external_code` (nullable)
- `external_message` (nullable)
- `raw_payload` (jsonb/text)
- `occurred_at` (timestamptz)

## 2.3 `submission_acks`
- `id` (uuid, pk)
- `submission_id` (uuid, fk)
- `ack_level` (smallint; nullable for non-level ack models)
- `success` (bool)
- `ack_code` (nullable)
- `ack_message` (nullable)
- `raw_payload` (jsonb/text)
- `received_at` (timestamptz)
- idempotency unique key on provider-specific identity fields

---

## 3. Unified State Machine

Internal statuses:
- `queued`
- `submission_requested`
- `submitted_pending_ack`
- `ack_received`
- `accepted_final`
- `rejected`
- `failed_retryable`
- `failed_final`

Rules:
1. `accepted_final`, `rejected`, `failed_final` are terminal.
2. Each adapter maps external statuses/ACKs into this model.
3. Duplicate events are idempotent.
4. Out-of-order events are tolerated; final state must remain deterministic.

---

## 4. Adapter Contract

Define trait/interface:

1. `submit(submission_ctx, xml_payload) -> SubmitResult`
2. `poll_status(submission_ctx) -> Vec<ExternalStatusEvent>`
3. `fetch_acks(submission_ctx) -> Vec<ExternalAckEvent>`
4. `classify_error(err) -> retryable|final`

Dispatch key:
- `authority` (+ `receiver_code` for ICH).

---

## 5. Authority Profiles

## 5.1 FDA
- Transport: ESG NextGen API.
- ACK model: ACK1..ACK4.
- Submit preconditions:
  - case `validation_profile=fda`
  - FDA validation passes
  - case status is `validated`

## 5.2 MFDS
- Transport: MFDS-approved gateway/API.
- ACK/status may not be 4-level; map to unified states.
- Preconditions:
  - case `validation_profile=mfds`
  - MFDS regional validations pass
  - case status is `validated`

## 5.3 ICH
- No single global gateway; use `receiver_code` adapters.
- Preconditions:
  - case `validation_profile=ich`
  - receiver-specific policy validation passes
  - case status is `validated`

---

## 6. API Surface

1. `POST /api/cases/{id}/submissions`
- Body:
  - `authority`: `fda|mfds|ich`
  - `environment`: `test|prod` (optional default)
  - `receiver_code`: required for ICH, optional otherwise

2. `GET /api/cases/{id}/submissions`
3. `GET /api/submissions/{id}`
4. `GET /api/submissions/{id}/events`
5. `GET /api/submissions/{id}/acks`

Remove mock endpoints from production path.

---

## 7. Worker Topology

1. Submit worker:
- pulls `queued|failed_retryable` where `next_retry_at <= now`
- calls adapter submit
- persists result + state transition

2. Poll worker:
- polls non-terminal submissions
- ingests status and ACKs
- applies transitions

3. Retry scheduler:
- computes exponential backoff with jitter
- caps attempts; sets `failed_final` on exhaustion

4. Reconciliation worker:
- periodic scan for stuck submissions
- re-sync with provider status APIs

---

## 8. Configuration

Shared:
- `SUBMISSION_ENABLED=1`
- `SUBMISSION_POLL_INTERVAL_SECS`
- `SUBMISSION_MAX_RETRIES`
- `SUBMISSION_TIMEOUT_SECS`

Authority-specific:
- FDA: `FDA_ESG_*`
- MFDS: `MFDS_*`
- ICH receiver-specific: `ICH_<RECEIVER>_*`

Rules:
1. Validate config at startup.
2. Fail fast if enabled authority has missing secrets.
3. Never log sensitive values.

---

## 9. Observability and Audit

Structured events and metrics by `authority` and `receiver_code`:
- submit success/fail rate
- time to final state
- rejection/error code distribution
- retry counts
- stuck submission count

Audit guarantees:
- who submitted
- exact payload hash
- external identifiers
- full event timeline

---

## 10. Implementation Sequence (Single Program)

1. Add shared DB migrations (`case_submissions`, `submission_events`, `submission_acks`).
2. Add `lib-core` models/BMC and query APIs.
3. Implement unified state machine + unit tests.
4. Build adapter trait + dispatcher.
5. Implement FDA real adapter first (highest confidence path).
6. Replace current in-memory submission logic with DB-backed core service.
7. Add generic `/submissions` API endpoints.
8. Implement workers (submit/poll/retry/reconcile).
9. Implement MFDS adapter on same framework.
10. Implement first ICH receiver adapter on same framework.
11. Add metrics, dashboards, alerts.
12. Run UAT by authority and canary rollout.

---

## 11. Rollout Strategy

Feature flags:
- `SUBMISSION_FDA_ENABLED`
- `SUBMISSION_MFDS_ENABLED`
- `SUBMISSION_ICH_ENABLED`
- optional receiver-level flags for ICH

Rollout:
1. Deploy schema + read-only APIs first.
2. Enable FDA test, then FDA prod.
3. Enable MFDS test, then MFDS prod.
4. Enable ICH receivers one-by-one.

Rollback:
- disable authority flag only
- preserve submission records and events

---

## 12. Definition of Done

- One shared submission platform handles FDA, MFDS, and ICH.
- All authorities use real adapters (no mock path in production).
- Submission lifecycle is durable, queryable, idempotent, and auditable.
- New receiver onboarding is adapter-only, without core redesign.
