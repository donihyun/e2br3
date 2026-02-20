# ICH Real Submission Gateway Plan (No Mock)

## Objective
Implement real ICH E2B(R3) submission using a pluggable transport model, because ICH itself defines data/message standards but submission transport is receiver-specific (authority or partner gateway dependent).

---

## Key Constraint

There is no single global “ICH gateway” equivalent to FDA ESG.
You must support one or more receiver adapters, for example:
- authority-specific API/gateway
- AS2 endpoint
- partner hub/provider API

So implementation should be **adapter-driven** under one common submission framework.

---

## Phase 0: Receiver Matrix and Contracts

1. Build receiver matrix:
- receiver code/name
- environment endpoints (test/prod)
- auth method
- transport type (API/AS2/SFTP/etc.)
- status/ACK retrieval pattern

2. Freeze contracts per receiver:
- submit method
- expected response/remote id
- ACK/status model
- error code catalog

Exit criteria:
- At least one concrete receiver contract is fully specified for v1.

---

## Phase 1: Shared Durable Schema

Use shared submission tables with receiver-level metadata:

1. `case_submissions`
- `id`, `case_id`
- `authority` (`ich`)
- `receiver_code`
- `gateway`, `environment`
- `remote_submission_id`
- `status`
- `payload_sha256`, `xml_bytes`
- `submitted_by`, `submitted_at`
- `last_status_at`, `last_error`
- timestamps

2. `submission_acks`
- `submission_id`
- `ack_level` or receiver status stage
- `success`, `ack_code`, `ack_message`
- `raw_payload`
- `received_at`

Exit criteria:
- All receivers can store state in one schema.

---

## Phase 2: Pluggable State Machine

Define generic statuses:
- `queued`
- `submission_requested`
- `submitted_pending_ack`
- `ack_received`
- `accepted_final`
- `rejected`
- `failed_retryable`
- `failed_final`

Add mapping per receiver:
- external code -> internal status

Rules:
1. receiver mapping tables must be explicit and versioned.
2. duplicate ACK/status events must be idempotent.

Exit criteria:
- Receiver A can be added without changing core state logic.

---

## Phase 3: Adapter Interface and First Real Adapter

Create trait/interface:
- `SubmissionAdapter`
  - `submit(payload) -> remote_submission_id`
  - `poll_status(remote_id) -> status events`
  - `fetch_acks(remote_id) -> ack events`

Files:
- `crates/services/web-server/src/submission/adapter.rs`
- `crates/services/web-server/src/submission/receivers/<receiver>.rs`

Implement one real adapter first (highest-priority receiver).

Exit criteria:
- One non-mock receiver end-to-end flow works.

---

## Phase 4: ICH Validation and Receiver Policies

Before queueing:
1. Case profile must be ICH.
2. XML must pass ICH rules/XSD.
3. Receiver-specific constraints (if any) must pass.

Add receiver policy layer:
- sender/receiver IDs formatting
- allowed code systems
- envelope/header nuances

Exit criteria:
- Fail fast on receiver-specific incompatibilities.

---

## Phase 5: REST/API Contract

1. `POST /api/cases/{id}/submissions/ich`
- body includes `receiver_code` and optional `environment`.

2. `GET /api/cases/{id}/submissions`
3. `GET /api/submissions/{id}`

No mock ACK API in production path.

Exit criteria:
- API can submit to chosen receiver using real adapter.

---

## Phase 6: Worker Architecture

1. submit worker:
- dispatches by `receiver_code` to appropriate adapter.

2. poll/ack worker:
- per receiver polling cadence and rate limits.

3. retry worker:
- receiver-specific retryable conditions.

4. reconciliation worker:
- detect and recover long-running “stuck” submissions.

Exit criteria:
- Multiple receivers can run concurrently without cross-impact.

---

## Phase 7: Security and Operations

1. Secrets/config namespaced by receiver:
- `ICH_<RECEIVER>_BASE_URL`
- `ICH_<RECEIVER>_CLIENT_ID`
- `ICH_<RECEIVER>_CLIENT_SECRET`
- cert/key paths if required

2. Security controls:
- redact credentials
- TLS/cert validation
- least-privilege network access

3. Observability:
- metrics by receiver
- rejection code distribution by receiver
- latency by receiver

Exit criteria:
- Receiver-specific ops visibility is available.

---

## Phase 8: Testing and Rollout

1. Unit tests:
- status mapping per receiver
- adapter selection
- idempotency

2. Integration tests:
- simulated receiver APIs for each adapter

3. UAT:
- real receiver test account(s) with positive/negative cases

4. Rollout:
- feature flag by receiver
- canary receiver first

Rollback:
- disable receiver flag only; keep others active.

---

## Execution Checklist (Order)

1. Add shared submission persistence schema.
2. Implement adapter trait and dispatcher.
3. Implement first real receiver adapter.
4. Add ICH submission endpoint with receiver selection.
5. Implement workers (submit/poll/retry/reconcile).
6. Add metrics/logging dashboards by receiver.
7. Complete UAT for first receiver.
8. Add additional receivers iteratively.

---

## Definition of Done

- No mock path in ICH production flow.
- At least one real receiver adapter is live end-to-end.
- Submission/ACK state is durable and queryable.
- New receiver onboarding requires adapter + mapping only (no core rewrite).
