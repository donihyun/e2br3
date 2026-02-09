# MFDS vs FDA E2B(R3) Differences (Implementation Notes)

## Scope
This note explains how to separate regional logic for:
- MFDS (Korea)
- FDA (US)

ICH E2B(R3) core remains shared and should not be duplicated per region.

## Practical Differences

1. Rollout timelines differ
- MFDS moved to E2B(R3) operations around 2021 guidance updates.
- FDA timeline is explicitly documented with Jan 16, 2024 (postmarket), Apr 1, 2024 (premarket), and transition through Apr 1, 2026.

2. Regional data elements differ
- FDA uses `FDA.*` regional fields and FDA-specific codelists/rules.
- MFDS uses `KR` regional fields (Korea-specific elements).

3. Business-rule catalogs differ
- FDA publishes machine-check style regional/core business rules with reject/warn behavior.
- MFDS has its own local required/conditional interpretation and KR elements, so rule IDs and enforcement matrix are not interchangeable with FDA.

4. Submission channels differ
- FDA: FAERS electronic gateway flow.
- MFDS: Korea domestic reporting channels/systems.

## Codebase Separation Rules

1. Shared
- ICH core mapping paths and parser behavior.
- Generic XML patch engine and round-trip framework.

2. FDA-only
- `crates/libs/lib-core/src/xml/fda/`
- `crates/libs/lib-core/src/xml/mapping/fda/`
- FDA-specific validators/rules.

3. MFDS-only
- `crates/libs/lib-core/src/xml/mfds/`
- `crates/libs/lib-core/src/xml/mapping/mfds/`
- MFDS-specific validators/rules.

## Sources
- FDA Regional Implementation Guide page:
  - https://www.fda.gov/regulatory-information/search-fda-guidance-documents/fda-regional-implementation-guide-e2br3-electronic-transmission-individual-case-safety-reports-drug
- FDA FAERS E2B(R3) standards page:
  - https://www.fda.gov/drugs/fdas-adverse-event-reporting-system-faers/fda-adverse-event-reporting-system-faers-electronic-submissions-e2br3-standards
- FDA FAERS electronic submissions timeline:
  - https://www.fda.gov/drugs/fdas-adverse-event-reporting-system-faers/fda-adverse-event-reporting-system-faers-electronic-submissions
- MFDS E2B(R3) guide notice (민원인안내서):
  - https://www.mfds.go.kr/brd/m_1060/view.do?seq=13056
- MFDS revised notice (민원인안내서):
  - https://www.mfds.go.kr/brd/m_1060/view.do?seq=14869

