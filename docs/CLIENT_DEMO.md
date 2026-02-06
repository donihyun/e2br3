# SafetyDB Client Demo (Pharma)

Date: 2026-02-05

## What’s Done (Ready to Show)
- Core SafetyDB data model and API coverage for E2B(R3) case handling.
- Case creation and key subresources: message header, safety report identification, sender, patient, reaction, drug, narrative.
- E2B(R3) XML import validation + case creation.
- E2B(R3) XML export generation.
- Audit trail + RLS context support for regulated environments.

## What’s Not Done Yet (Known Gaps)
- Production hardening (monitoring/alerting, HA, full ops runbook).
- Full UI polish and end‑to‑end UX refinement.
- Complete compliance validation across all edge cases and optional sections.
- Automated migrations/seed pipelines for production environments.

## What I Will Demo (5–7 minutes)
- Login and environment setup.
- Create a new case and add minimum required data.
- Validate and mark a case as ready for export.
- Export the case as E2B(R3) XML.
- (Optional) Import an existing E2B(R3) XML to create a case.

## Demo Flow (Short Script)
1. Show the SafetyDB API surface for cases + safety report subresources.
2. Create a case with basic data (message header, safety report, patient, reaction, drug).
3. Mark the case as `validated`.
4. Export to E2B(R3) XML and show key mapped fields.
5. (Optional) Import an E2B(R3) XML and show the created case.

## Notes
- Audience: Pharma safety team
- Key outcome: “Fast, compliant, auditable E2B(R3) case handling from intake to export.”
