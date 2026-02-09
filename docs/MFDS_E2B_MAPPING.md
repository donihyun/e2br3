# MFDS E2B(R3) Mapping (Draft)

## Goal
- Keep ICH core mapping shared.
- Keep MFDS regional (`KR`) mapping separate from FDA regional mapping.
- Support lossless XML patching with regional rule packs.

## Source Status
- This file is implementation scaffolding.
- Regional field IDs below were consolidated from widely-used implementer references.
- Before production go-live, verify each row against the latest official MFDS package.

## Mapping Columns
- Section / Field
- MFDS Conformance (Required / Conditional / Optional)
- Rule ID
- DB Table.Column
- API Endpoint
- Exporter XPath
- Importer XPath
- Validator Rule
- Notes

## Sections

### C — Safety Report / Sender / Reporter
- `C.2.r.4.KR.1`
- `C.3.1.KR.1`
- `C.5.4.KR.1`

### D — Patient
- `D.8.r.1.KR.1a`
- `D.8.r.1.KR.1b`
- `D.10.8.r.1.KR.1a`
- `D.10.8.r.1.KR.1b`

### E — Reaction
- No confirmed KR field IDs in current extracted set.

### F — Test
- No confirmed KR field IDs in current extracted set.

### G — Drug
- `G.k.2.1.KR.1a`
- `G.k.2.1.KR.1b`
- `G.k.2.3.r.1.KR.1a`
- `G.k.2.3.r.1.KR.1b`
- `G.k.9.i.2.r.2.KR.1`
- `G.k.9.i.2.r.3.KR.1`
- `G.k.9.i.2.r.3.KR.2`

### H — Narrative
- No confirmed KR field IDs in current extracted set.

## Known Conditional Rules (to encode in validator)
- `C.5.4.KR.1` is required when `C.5.4 = 3` (Other studies).
- `C.2.r.4.KR.1` applies when reporter qualification indicates Other health professional.
- `C.3.1.KR.1` applies when sender type is health professional.
- For domestic post-marketed cases, KR product/ingredient local coding fields become conditionally required.
- For foreign-use products, WHO MPID pathway may be used for KR product/ingredient fields.
- `G.k.9.i.2.r.3.KR.1/.KR.2` allowed values depend on source-of-assessment categories.

## Implementation Pointers
- Rust module for MFDS region constants: `crates/libs/lib-core/src/xml/mfds/codes.rs`
- Rust module for MFDS rules: `crates/libs/lib-core/src/xml/mfds/business_rules.rs`
- Rust MFDS mapping modules: `crates/libs/lib-core/src/xml/mapping/mfds/`

## External References Used For Bootstrap
- Oracle Argus MFDS profile summary:
  - https://docs.oracle.com/en/industries/health-sciences/argus-safety/8.2.2/mfdbp/introduction.html
- Veeva Vault Safety MFDS profile:
  - https://safety.veevavault.help/en/lr/86927/
