# FDA E2B(R3) Mapping (Draft)

## Sources (authoritative)
- FDA E2B(R3) Core and Regional Data Elements and Business Rules (Excel)
- FDA Regional Implementation Guide (Aug 2024)
- ICH E2B(R3) ICSR Implementation Guide

> This file will be populated by extracting the FDA Core/Regional Excel and mapping each required field to DB/API/exporter/validator coverage.

## Mapping Columns
- Section / Field (e.g., N.1.4 Batch Receiver Identifier)
- FDA Requirement (Required/Conditional/Optional)
- FDA Rule ID (if applicable)
- DB Table.Column (or **MISSING**)
- API Endpoint (or **MISSING**)
- Exporter XPath (or **MISSING**)
- Validator Rule (XSD / custom / **MISSING**)
- Notes / Gaps

## Sections (skeleton)

### N — Batch / Message Header
- N.1.1 Type of Messages in Batch
- N.1.2 Batch Number
- N.1.3 Batch Sender Identifier
- N.1.4 Batch Receiver Identifier
- N.1.5 Date of Batch Transmission
- N.2.r.* Message identification

### C — Safety Report / Sender / Reporter / Receiver
- C.1.* Case identifiers and dates
- C.2.* Primary source
- C.3.* Sender
- C.4.* Literature / documents
- C.5.* Study information

### D — Patient
- D.1.* Patient identifiers
- D.2.* Age / DOB
- D.3.* Weight
- D.4.* Height
- D.5.* Sex
- D.6.* LMP
- D.7.* Medical history
- D.8.* Past drug history
- D.9.* Death info
- D.10.* Parent info

### E — Reaction / Event
- E.i.* Reaction details

### F — Tests
- F.r.* Test results

### G — Drug / Product
- G.k.* Drug info

### H — Narrative
- H.1.* Narrative
- H.2.* Reporter comments
- H.3.* Sender diagnosis
- H.4.* Sender comments
- H.5.* Case summary

## Next Step
- Download FDA Core/Regional Excel and populate this mapping table.

## Extracted FDA Core/Regional (Post-Market) Mapping (auto-extracted)

| Data Element | Name | ICH Conformance | FDA Post-Market Conformance | XPath |
|---|---|---|---|---|
| FDA.C.1.7.1 | Local Criteria Report Type | - | Required |  |
| FDA.C.1.12 | Combination Product Report Indicator | - | Required |  |
| FDA.C.2.r.2.8 | Reporter's Email | - | Required |  |
| C.3.3.1 | Sender’s Department | Optional | Required |  |
| C.3.3.2 | Sender’s Title | Optional | Required |  |
| C.3.3.3 | Sender’s Given Name | Optional | Required |  |
| C.3.3.5 | Sender’s Family Name | Optional | Required |  |
| C.3.4.1 | Sender’s Street Address | Optional | Required |  |
| C.3.4.2 | Sender’s City | Optional | Required |  |

## Gap Mapping (Phase 1) — N Section (Batch/Message Header)

| Data Element | DB Table.Column | API Endpoint | Exporter XPath | Validator Rule | Notes |
|---|---|---|---|---|---|
| N.1.1 Types of Message in Batch | **MISSING (DB)** (constant in export skeleton) | **MISSING (API)** (not exposed) | `/MCCI_IN200100UV01/name[@codeSystem='2.16.840.1.113883.3.989.2.1.1.1']/@code` | XSD | Currently fixed in export skeleton; should be stored or derived if variable. |
| N.1.2 Batch Number | `message_headers.batch_number` | `POST/PUT /api/cases/{case_id}/message-header` | `/MCCI_IN200100UV01/id[@root='2.16.840.1.113883.3.989.2.1.3.22']/@extension` | XSD | Set via message header update. |
| N.1.3 Batch Sender Identifier | `message_headers.batch_sender_identifier` | `POST/PUT /api/cases/{case_id}/message-header` | `/MCCI_IN200100UV01/sender/device/id/@extension` | XSD | Required by FDA; must be populated. |
| N.1.4 Batch Receiver Identifier | `message_headers.batch_receiver_identifier` | `POST/PUT /api/cases/{case_id}/message-header` | `/MCCI_IN200100UV01/receiver/device/id/@extension` | XSD | Required by FDA; must be populated. |
| N.1.5 Date of Batch Transmission | `message_headers.batch_transmission_date` | `POST/PUT /api/cases/{case_id}/message-header` | `/MCCI_IN200100UV01/creationTime/@value` | XSD | FDA requires; currently not set in dev script. |
| N.2.r.1 Message Identifier | `message_headers.message_number` | `POST /api/cases/{case_id}/message-header` | `/MCCI_IN200100UV01/PORR_IN049016UV/id/@extension` | XSD | Stored on create. |
| N.2.r.2 Message Sender Identifier | `message_headers.message_sender_identifier` | `POST /api/cases/{case_id}/message-header` | `/MCCI_IN200100UV01/PORR_IN049016UV/sender/device/id/@extension` | XSD | Stored on create. |
| N.2.r.3 Message Receiver Identifier | `message_headers.message_receiver_identifier` | `POST /api/cases/{case_id}/message-header` | `/MCCI_IN200100UV01/PORR_IN049016UV/receiver/device/id/@extension` | XSD | Stored on create. |
| N.2.r.4 Date of Message Creation | `message_headers.message_date` | `POST /api/cases/{case_id}/message-header` | `/MCCI_IN200100UV01/PORR_IN049016UV/creationTime/@value` | XSD | Stored on create. |

## Gap Mapping (Phase 1) — C Section (Safety Report / Sender / Reporter / Receiver / Studies)

| Data Element | DB Table.Column | API Endpoint | Exporter XPath | Validator Rule | Notes |
|---|---|---|---|---|---|
| C.1.1 Sender’s Safety Report Unique Identifier | `cases.safety_report_id` | `POST /api/cases` | `//id[@root='2.16.840.1.113883.3.989.2.1.3.1']/@extension` | XSD | Set at case creation. |
| C.1.2 Date of Creation | `safety_report_identification.transmission_date` | `POST/PUT /api/cases/{case_id}/safety-report` | `//controlActProcess/effectiveTime/@value` | XSD | Required. |
| C.1.3 Type of Report | `safety_report_identification.report_type` | `POST/PUT /api/cases/{case_id}/safety-report` | `//investigationCharacteristic[code='1']/value/@code` | XSD | Required. |
| C.1.4 Date First Received from Source | `safety_report_identification.date_first_received_from_source` | `POST/PUT /api/cases/{case_id}/safety-report` | `//investigationEvent/effectiveTime/low/@value` | XSD | Required. |
| C.1.5 Date of Most Recent Information | `safety_report_identification.date_of_most_recent_information` | `POST/PUT /api/cases/{case_id}/safety-report` | `//investigationEvent/availabilityTime/@value` | XSD | Required. |
| C.1.6.1 Are Additional Documents Available | derived | `POST/PUT /api/cases/{case_id}/safety-report/documents-held` | `//observationEvent[code='1']/value/@value` | XSD | Derived from documents_held_by_sender presence. |
| C.1.7 Fulfil Expedited Criteria | `safety_report_identification.fulfil_expedited_criteria` | `POST/PUT /api/cases/{case_id}/safety-report` | `//observationEvent[code='23']/value/@value` | XSD | Required (boolean). |
| C.1.8.1 Worldwide Unique Case ID | `safety_report_identification.worldwide_unique_id` | `PUT /api/cases/{case_id}/safety-report` | `//id[@root='2.16.840.1.113883.3.989.2.1.3.2']/@extension` | XSD | Optional. |
| C.1.11.1 Nullification/Amendment Code | `safety_report_identification.nullification_code` | `PUT /api/cases/{case_id}/safety-report` | `//investigationCharacteristic[code='3']/value/@code` | XSD | Optional. |
| C.1.11.2 Nullification Reason | `safety_report_identification.nullification_reason` | `PUT /api/cases/{case_id}/safety-report` | `//investigationCharacteristic[code='4']/value/originalText` | XSD | Optional. |
| C.2.r.* Primary Source | `primary_sources.*` | `POST/PUT /api/cases/{case_id}/safety-report/primary-sources` | `//outboundRelationship[@typeCode='SPRT' and code='2']` | XSD + custom | Required elements within primary source. |
| C.2.r.2 Reporter Address/Contact | `primary_sources.street/city/state/postcode/telephone/email` | same as above | `//assignedEntity/addr/*` + `telecom` | XSD + custom | Email required per FDA post‑market. |
| C.2.r.3 Reporter Country Code | `primary_sources.country_code` | same as above | `//assignedPerson/asLocatedEntity/location/code/@code` | XSD + custom | ISO 3166. |
| C.2.r.4 Qualification | `primary_sources.qualification` | same as above | `//assignedPerson/asQualifiedEntity/code/@code` | XSD | Required within primary source. |
| C.2.r.5 Primary Source Regulatory | `primary_sources.primary_source_regulatory` | same as above | `//priorityNumber/@value` | XSD | Required. |
| C.3.1 Sender Type | `sender_information.sender_type` | `POST/PUT /api/cases/{case_id}/safety-report/senders` | `//assignedEntity/code/@code` | XSD | Required. |
| C.3.2 Sender Organization | `sender_information.organization_name` | same as above | `//representedOrganization/assignedEntity/representedOrganization/name` | XSD | Required. |
| C.3.3 Sender Person Name | `sender_information.person_*` | same as above | `//assignedPerson/name/*` | XSD + custom | FDA requires given/family for post‑market. |
| C.3.4 Sender Address/Contact | `sender_information.street/city/state/postcode/telephone/fax/email/country_code` | same as above | `//addr/*` + `telecom` | XSD + custom | Email required per FDA post‑market. |
| C.4.r.* Literature References | `literature_references.*` | `POST/PUT /api/cases/{case_id}/safety-report/literature` | `//reference[@typeCode='REFR']/document` | XSD | Optional; supports base64 docs. |
| C.4.r.* Documents Held by Sender | `documents_held_by_sender.*` | `POST/PUT /api/cases/{case_id}/safety-report/documents-held` | `//reference[@typeCode='REFR']/document` | XSD | Optional; presence drives C.1.6.1. |
| C.5 Study Information | `study_information.*` | `POST/PUT /api/cases/{case_id}/safety-report/studies` | `//researchStudy/*` | XSD | Optional; includes study name/type. |
| C.5.1.r Study Registration | `study_registration_numbers.*` | `POST/PUT /api/cases/{case_id}/safety-report/studies/{study_id}/registrations` | `//researchStudy/authorization/studyRegistration/*` | XSD | Optional; includes country code. |
| C.3.4.3 | Sender’s State or Province | Optional | Required |  |
| C.3.4.4 | Sender’s Postcode | Optional | Required |  |
| C.3.4.5 | Sender’s Country Code | Optional | Required |  |
| C.3.4.6 | Sender’s Telephone | Optional | Required |  |
| C.3.4.7 | Sender’s Fax | Optional | Required |  |
| C.3.4.8 | Sender’s E-mail Address | Optional | Required |  |
| FDA.C.5.5a | IND Number where AE Occurred | - | Conditional-Required |  |
| FDA.C.5.5b | Pre-ANDA Number where AE Occurred | - | Conditional-Required |  |
| FDA.C.5.6.r | IND number of cross reported IND | - | Conditional-Required |  |
| D.9.1 | Date of Death | Optional | Conditional-Required |  |
| FDA.D.11.r.1 | Patient Race Code | - | Required |  |
| FDA.D.12 | Patient Ethnicity Code | - | Required |  |
| FDA.E.i.3.2h | Required Intervention | - | Required |  |
| G.k.9.i.2.r.1 | Source of Assessment | Optional | Conditional-Required |  |
| G.k.9.i.2.r.2 | Method of Assessment | Optional | Conditional-Required |  |
| G.k.9.i.2.r.3 | Result of Assessment | Optional | Conditional-Required |  |
| FDA.G.k.10a | FDA Additional Information on Drug (coded) | - | Conditional-Required |  |

## Gap Mapping (Phase 1) — D Section (Patient)

| Data Element | DB Table.Column | API Endpoint | Exporter XPath | Validator Rule | Notes |
|---|---|---|---|---|---|
| D.1 Patient (name/initials) | `patient_information.patient_initials` / `patient_given_name` / `patient_family_name` | `POST/PUT /api/cases/{case_id}/patient` | `//primaryRole/player1/name` | XSD | Initials preferred; falls back to given+family. |
| D.1.1.1 Patient Medical Record Number | `patient_identifiers.identifier_value` | `POST/PUT /api/cases/{case_id}/patient/identifiers` | `//primaryRole/player1/asIdentifiedEntity/id/@extension` | XSD | Patient identifiers CRUD exposed. |
| D.1.1.2 Source of Record Number | `patient_identifiers.identifier_type_code` | `POST/PUT /api/cases/{case_id}/patient/identifiers` | `//primaryRole/player1/asIdentifiedEntity/code/@code` | XSD | Root set by `identifier_type_code` mapping. |
| D.2.1 Date of Birth | `patient_information.birth_date` | `PUT /api/cases/{case_id}/patient` | `//primaryRole/player1/birthTime/@value` | XSD | Date format `YYYYMMDD`. |
| D.2.2a Age at Onset (number) | `patient_information.age_at_time_of_onset` | `PUT /api/cases/{case_id}/patient` | `//subjectOf2/observation[code='3']/value/@value` | XSD | Uses code `3`. |
| D.2.2b Age at Onset (unit) | `patient_information.age_unit` | `PUT /api/cases/{case_id}/patient` | `//subjectOf2/observation[code='3']/value/@unit` | XSD | |
| D.2.2.1a Gestation Period (number) | `patient_information.gestation_period` | `PUT /api/cases/{case_id}/patient` | `//subjectOf2/observation[code='16']/value/@value` | XSD | |
| D.2.2.1b Gestation Period (unit) | `patient_information.gestation_period_unit` | `PUT /api/cases/{case_id}/patient` | `//subjectOf2/observation[code='16']/value/@unit` | XSD | |
| D.2.3 Patient Age Group | `patient_information.age_group` | `PUT /api/cases/{case_id}/patient` | `//subjectOf2/observation[code='4']/value/@code` | XSD | |
| D.3 Body Weight | `patient_information.weight_kg` | `PUT /api/cases/{case_id}/patient` | `//subjectOf2/observation[code='7']/value/@value` | XSD | Units assumed kg by export mapping. |
| D.4 Height | `patient_information.height_cm` | `PUT /api/cases/{case_id}/patient` | `//subjectOf2/observation[code='17']/value/@value` | XSD | Units assumed cm by export mapping. |
| D.5 Sex | `patient_information.sex` | `PUT /api/cases/{case_id}/patient` | `//primaryRole/player1/administrativeGenderCode/@code` | XSD | |
| D.6 LMP | `patient_information.last_menstrual_period_date` | `PUT /api/cases/{case_id}/patient` | `//subjectOf2/observation[code='22']/value/@value` | XSD | Removes `nullFlavor` when set. |
| D.7.1.r Medical History (MedDRA) | `medical_history_episodes.meddra_code/meddra_version/start_date/continuing/comments/family_history` | `POST/PUT /api/cases/{case_id}/patient/medical-history` | `//organizer[code='1']/component/observation/*` | XSD | Repeats per episode. |
| D.7.2 Medical History Text | `patient_information.medical_history_text` | `PUT /api/cases/{case_id}/patient` | `//organizer[code='1']/component/observation[code='18']/value` | XSD | |
| D.7.3 Concomitant Therapies | `patient_information.concomitant_therapy` | `PUT /api/cases/{case_id}/patient` | `//organizer[code='1']/component/observation[code='11']/value/@value` | XSD | |
| D.8.r Past Drug History | `past_drug_history.*` | `POST/PUT /api/cases/{case_id}/patient/past-drugs` | `//organizer[code='2']/component/substanceAdministration/*` | XSD | MPID/PhPID, dates, indication, reactions. |
| D.9.1 Date of Death | `patient_death_information.date_of_death` | `POST/PUT /api/cases/{case_id}/patient/death-info` | `//primaryRole/player1/deceasedTime/@value` | XSD | FDA Conditional‑Required. |
| D.9.2.r Reported Cause of Death | `reported_causes_of_death.meddra_code/meddra_version` | `POST/PUT /api/cases/{case_id}/patient/death-info/{death_info_id}/reported-causes` | `//subjectOf2/observation[code='32']/value/@code` | XSD | Repeats per cause. |
| D.9.3 Was Autopsy Done | `patient_death_information.autopsy_performed` | `POST/PUT /api/cases/{case_id}/patient/death-info` | `//subjectOf2/observation[code='5']/value/@value` | XSD | |
| D.9.4.r Autopsy Cause of Death | `autopsy_causes_of_death.meddra_code/meddra_version` | `POST/PUT /api/cases/{case_id}/patient/death-info/{death_info_id}/autopsy-causes` | `//subjectOf2/observation[code='5']/outboundRelationship2/observation/value/@code` | XSD | Repeats per cause. |
| D.10.1 Parent Identification | `parent_information.parent_identification` | `POST/PUT /api/cases/{case_id}/patient/parents` | `//primaryRole/player1/role[@classCode='PRS']/associatedPerson/name` | XSD | |
| D.10.3 Parent Birth Date | `parent_information.parent_birth_date` | `POST/PUT /api/cases/{case_id}/patient/parents` | `//role[@classCode='PRS']/associatedPerson/birthTime/@value` | XSD | |
| D.10.2.1/2 Parent Age (value/unit) | `parent_information.parent_age` / `parent_age_unit` | `POST/PUT /api/cases/{case_id}/patient/parents` | `//role[@classCode='PRS']/subjectOf2/observation[code='3']/value/@value` + `@unit` | XSD | |
| D.10.6 Parent Sex | `parent_information.sex` | `POST/PUT /api/cases/{case_id}/patient/parents` | `//role[@classCode='PRS']/associatedPerson/administrativeGenderCode/@code` | XSD | |
| D.10.4/5 Parent LMP / Weight / Height | `parent_information.last_menstrual_period_date` / `weight_kg` / `height_cm` | `POST/PUT /api/cases/{case_id}/patient/parents` | `//role[@classCode='PRS']/subjectOf2/observation[code='22' or code='7' or code='17']/value` | XSD | Weight/height units set to kg/cm. |
| D.10.7.r Parent Medical History | `parent_medical_history.*` | `POST/PUT /api/cases/{case_id}/patient/parent/{parent_id}/medical-history` | `//role[@classCode='PRS']/subjectOf2/organizer[code='1']/component/observation/*` | XSD | Repeats per episode. |
| D.10.8.r Parent Past Drug History | `parent_past_drug_history.*` | `POST/PUT /api/cases/{case_id}/patient/parent/{parent_id}/past-drugs` | `//role[@classCode='PRS']/subjectOf2/organizer[code='2']/component/substanceAdministration/*` | XSD | Repeats per drug. |
| FDA.D.11.r.1 Patient Race Code | `patient_information.race_code` | `PUT /api/cases/{case_id}/patient` | `//primaryRole/subjectOf2/observation[code='C17049']/value/@code` | custom (FDA.D.11) | Exported via subjectOf2/observation. |
| FDA.D.12 Patient Ethnicity Code | `patient_information.ethnicity_code` | `PUT /api/cases/{case_id}/patient` | `//primaryRole/subjectOf2/observation[code='C16564']/value/@code` | custom (FDA.D.12) | Exported via subjectOf2/observation. |

## Gap Mapping (Phase 1) — E Section (Reaction/Event)

| Data Element | DB Table.Column | API Endpoint | Exporter XPath | Validator Rule | Notes |
|---|---|---|---|---|---|
| E.i.1.1 Reaction/Event as Reported | `reactions.primary_source_reaction` | `POST/PUT /api/cases/{case_id}/reactions` | `//subjectOf2/observation[code='29']/value/originalText` | XSD | Required. |
| E.i.1.1b Reaction Language | `reactions.reaction_language` | `PUT /api/cases/{case_id}/reactions/{id}` | `//subjectOf2/observation[code='29']/value/originalText/@language` | XSD | REST update exists via reactions endpoint. |
| E.i.1.2 Reaction for Translation | derived | derived | `//subjectOf2/observation[code='29']/outboundRelationship2/observation[code='30']/value` | XSD | Exporter uses same text as E.i.1.1. |
| E.i.2.1 MedDRA Version | `reactions.reaction_meddra_version` | `PUT /api/cases/{case_id}/reactions/{id}` | `//subjectOf2/observation[code='29']/value/@codeSystemVersion` | XSD | |
| E.i.2.2 Reaction/Event (MedDRA code) | `reactions.reaction_meddra_code` | `PUT /api/cases/{case_id}/reactions/{id}` | `//subjectOf2/observation[code='29']/value/@code` | XSD | |
| E.i.3 Term Highlighted by Reporter | `reactions.term_highlighted` | `PUT /api/cases/{case_id}/reactions/{id}` | `//subjectOf2/observation[code='29']/outboundRelationship2/observation[code='37']/value/@code` | XSD | REST update exists via reactions endpoint. |
| E.i.3.1 Term Highlighted by Reporter | `reactions.term_highlighted` | `PUT /api/cases/{case_id}/reactions/{id}` | `//subjectOf2/observation[code='29']/outboundRelationship2/observation[code='37']/value/@code` | XSD | Boolean mapped to 1/2. |
| E.i.3.2a Results in Death | `reactions.criteria_death` | `PUT /api/cases/{case_id}/reactions/{id}` | `//subjectOf2/observation[code='29']/outboundRelationship2/observation[code='34']/value/@value` | XSD | Exported via bool flag. |
| E.i.3.2b Life Threatening | `reactions.criteria_life_threatening` | `PUT /api/cases/{case_id}/reactions/{id}` | `//subjectOf2/observation[code='29']/outboundRelationship2/observation[code='21']/value/@value` | XSD | Exported via bool flag. |
| E.i.3.2c Hospitalization | `reactions.criteria_hospitalization` | `PUT /api/cases/{case_id}/reactions/{id}` | `//subjectOf2/observation[code='29']/outboundRelationship2/observation[code='33']/value/@value` | XSD | Exported via bool flag. |
| E.i.3.2d Disabling | `reactions.criteria_disabling` | `PUT /api/cases/{case_id}/reactions/{id}` | `//subjectOf2/observation[code='29']/outboundRelationship2/observation[code='35']/value/@value` | XSD | REST update exists via reactions endpoint. |
| E.i.3.2e Congenital Anomaly | `reactions.criteria_congenital_anomaly` | `PUT /api/cases/{case_id}/reactions/{id}` | `//subjectOf2/observation[code='29']/outboundRelationship2/observation[code='12']/value/@value` | XSD | REST update exists via reactions endpoint. |
| E.i.3.2f Other Medically Important | `reactions.criteria_other_medically_important` | `PUT /api/cases/{case_id}/reactions/{id}` | `//subjectOf2/observation[code='29']/outboundRelationship2/observation[code='26']/value/@value` | XSD | REST update exists via reactions endpoint. |
| FDA.E.i.3.2h Required Intervention | `reactions.required_intervention` | `PUT /api/cases/{case_id}/reactions/{id}` | `//subjectOf2/observation[code='29']/outboundRelationship2/observation[code='726']/value/@value` | custom (FDA.E.i.3.2h) | Exported when present; nullFlavor otherwise. |
| E.i.4 Start Date | `reactions.start_date` | `PUT /api/cases/{case_id}/reactions/{id}` | `//subjectOf2/observation[code='29']/effectiveTime/comp[@xsi:type='IVL_TS']/low/@value` | XSD | |
| E.i.5 End Date | `reactions.end_date` | `PUT /api/cases/{case_id}/reactions/{id}` | `//subjectOf2/observation[code='29']/effectiveTime/comp[@xsi:type='IVL_TS']/high/@value` | XSD | |
| E.i.6a Duration (number) | `reactions.duration_value` | `PUT /api/cases/{case_id}/reactions/{id}` | `//subjectOf2/observation[code='29']/effectiveTime/comp[@operator='A']/width/@value` | XSD | REST update exists via reactions endpoint. |
| E.i.6b Duration (unit) | `reactions.duration_unit` | `PUT /api/cases/{case_id}/reactions/{id}` | `//subjectOf2/observation[code='29']/effectiveTime/comp[@operator='A']/width/@unit` | XSD | REST update exists via reactions endpoint. |
| E.i.7 Outcome | `reactions.outcome` | `PUT /api/cases/{case_id}/reactions/{id}` | `//subjectOf2/observation[code='29']/outboundRelationship2/observation[code='27']/value/@code` | XSD | |
| E.i.8 Medical Confirmation | `reactions.medical_confirmation` | `PUT /api/cases/{case_id}/reactions/{id}` | **MISSING (Exporter)** | XSD | DB/API present; exporter not wired. |
| E.i.9 Country | `reactions.country_code` | `PUT /api/cases/{case_id}/reactions/{id}` | `//subjectOf2/observation[code='29']/location/locatedEntity/locatedPlace/code/@code` | XSD | REST update exists via reactions endpoint. |

## Gap Mapping (Phase 1) — F Section (Tests and Procedures)

| Data Element | DB Table.Column | API Endpoint | Exporter XPath | Validator Rule | Notes |
|---|---|---|---|---|---|
| F.r.1 Test Date | `test_results.test_date` | `PUT /api/cases/{case_id}/test-results/{id}` | `//subjectOf2/organizer[code='3']/component/observation/effectiveTime/@value` | XSD | |
| F.r.2.1 Test Name (MedDRA code) | `test_results.test_meddra_code` | `PUT /api/cases/{case_id}/test-results/{id}` | `//subjectOf2/organizer[code='3']/component/observation/code/@code` | XSD | REST update exists via test-results endpoint. |
| F.r.2.1a MedDRA Version | `test_results.test_meddra_version` | `PUT /api/cases/{case_id}/test-results/{id}` | `//subjectOf2/organizer[code='3']/component/observation/code/@codeSystemVersion` | XSD | REST update exists via test-results endpoint. |
| F.r.2.1 Test Name (free text) | `test_results.test_name` | `POST/PUT /api/cases/{case_id}/test-results` | `//organizer[code='3']/component/observation/code/originalText` | XSD | Exported in structured and unstructured nodes. |
| F.r.3.1 Test Result (coded) | `test_results.test_result_code` | `PUT /api/cases/{case_id}/test-results/{id}` | `//organizer[code='3']/component/observation/value[@xsi:type='CE']/@code` | XSD | Exported when coded result present. |
| F.r.3.2 Test Result (value) | `test_results.test_result_value` | `PUT /api/cases/{case_id}/test-results/{id}` | `//subjectOf2/organizer[code='3']/component/observation/value[@xsi:type='IVL_PQ']/center/@value` | XSD | |
| F.r.3.3 Test Result (unit) | `test_results.test_result_unit` | `PUT /api/cases/{case_id}/test-results/{id}` | `//subjectOf2/organizer[code='3']/component/observation/value[@xsi:type='IVL_PQ']/center/@unit` | XSD | |
| F.r.3.4 Result Unstructured Data | `test_results.result_unstructured` | `PUT /api/cases/{case_id}/test-results/{id}` | `//organizer[code='3']/component/observation/value[@xsi:type='ED']` | XSD | Exported for unstructured results. |
| F.r.4 Normal Low Value | `test_results.normal_low_value` | `PUT /api/cases/{case_id}/test-results/{id}` | `//organizer[code='3']/component/observation/referenceRange/observationRange/value/@value` | XSD | Exported when high value is absent. |
| F.r.5 Normal High Value | `test_results.normal_high_value` | `PUT /api/cases/{case_id}/test-results/{id}` | `//subjectOf2/organizer[code='3']/component/observation/referenceRange/observationRange/value/@value` | XSD | Exporter only sets high value. |
| F.r.6 Comments | `test_results.comments` | `PUT /api/cases/{case_id}/test-results/{id}` | `//subjectOf2/organizer[code='3']/component/observation/outboundRelationship2/observation/value` | XSD | |
| F.r.7 More Information Available | `test_results.more_info_available` | `PUT /api/cases/{case_id}/test-results/{id}` | `//subjectOf2/organizer[code='3']/component/observation/outboundRelationship2/observation[code='25']/value/@value` | XSD | REST update exists via test-results endpoint. |

## Gap Mapping (Phase 1) — G Section (Drug/Product)

| Data Element | DB Table.Column | API Endpoint | Exporter XPath | Validator Rule | Notes |
|---|---|---|---|---|---|
| G.k.1 Drug Role | `drug_information.drug_characterization` | `POST/PUT /api/cases/{case_id}/drugs/{id}` | `//component/causalityAssessment[code='20']/value/@code` | XSD | Required. |
| G.k.2.2 Medicinal Product | `drug_information.medicinal_product` | `POST/PUT /api/cases/{case_id}/drugs/{id}` | `//consumable/instanceOfKind/kindOfProduct/name` | XSD | Required. |
| G.k.2.3.r Substance Name | `drug_active_substances.substance_name` | `POST/PUT /api/cases/{case_id}/drugs/{drug_id}/active-substances` | `//ingredient/ingredientSubstance/name` | XSD | Repeats. |
| G.k.2.3.r Substance TermID | `drug_active_substances.substance_termid` | `PUT /api/cases/{case_id}/drugs/{drug_id}/active-substances/{id}` | `//ingredient/ingredientSubstance/code/@code` | XSD | |
| G.k.2.3.r TermID Version | `drug_active_substances.substance_termid_version` | `PUT /api/cases/{case_id}/drugs/{drug_id}/active-substances/{id}` | `//ingredient/ingredientSubstance/code/@codeSystemVersion` | XSD | |
| G.k.2.3.r Strength (value/unit) | `drug_active_substances.strength_value` / `strength_unit` | `PUT /api/cases/{case_id}/drugs/{drug_id}/active-substances/{id}` | `//ingredient/quantity/numerator/@value` + `@unit` | XSD | |
| G.k.2.4/2.5 MPID/PhPID | `drug_information.mpid/phpid` | `PUT /api/cases/{case_id}/drugs/{id}` | `//kindOfProduct/code/@code` | XSD | REST update exists via drugs endpoint. |
| G.k.2.4/2.5 MPID/PhPID Version | `drug_information.mpid_version/phpid_version` | `PUT /api/cases/{case_id}/drugs/{id}` | `//kindOfProduct/code/@codeSystemVersion` | XSD | REST update exists via drugs endpoint. |
| G.k.2.5 Investigational Product Blinded | `drug_information.investigational_product_blinded` | `PUT /api/cases/{case_id}/drugs/{id}` | `//kindOfProduct/subjectOf/observation[code='G.k.2.5']/value/@value` | XSD | |
| G.k.3.1 Obtain Drug Country | `drug_information.obtain_drug_country` | `PUT /api/cases/{case_id}/drugs/{id}` | `//productEvent/performer/assignedEntity/representedOrganization/addr/country` | XSD | REST update exists via drugs endpoint. |
| G.k.3.2 Brand Name | `drug_information.brand_name` | `PUT /api/cases/{case_id}/drugs/{id}` | `//kindOfProduct/name[2]` | XSD | Exported as second product name. |
| G.k.3.3 Manufacturer Name | `drug_information.manufacturer_name` | `PUT /api/cases/{case_id}/drugs/{id}` | `//asManufacturedProduct/subjectOf/approval/holder/role/playingOrganization/name` | XSD | |
| G.k.3.3 Manufacturer Country | `drug_information.manufacturer_country` | `PUT /api/cases/{case_id}/drugs/{id}` | `//asManufacturedProduct/subjectOf/approval/author/territorialAuthority/territory/code/@code` | XSD | REST update exists via drugs endpoint. |
| G.k.3.4 Batch/Lot Number | `drug_information.batch_lot_number` | `PUT /api/cases/{case_id}/drugs/{id}` | `//productInstanceInstance/lotNumberText` | XSD | Exported on dosage consumable. |
| G.k.4.r Dosage (dose/value/unit) | `dosage_information.dose_value` / `dose_unit` | `POST/PUT /api/cases/{case_id}/drugs/{drug_id}/dosages` | `//outboundRelationship2[@typeCode='COMP']/substanceAdministration/doseQuantity/@value` + `@unit` | XSD | |
| G.k.4.r Frequency | `dosage_information.frequency_value` / `frequency_unit` | `PUT /api/cases/{case_id}/drugs/{drug_id}/dosages/{id}` | `//effectiveTime/comp[@xsi:type='PIVL_TS']/period/@value` + `@unit` | XSD | |
| G.k.4.r Start/End Date | `dosage_information.first_administration_date` / `last_administration_date` | `PUT /api/cases/{case_id}/drugs/{drug_id}/dosages/{id}` | `//effectiveTime/comp[@operator='A']/low|high/@value` | XSD | |
| G.k.4.r Duration | `dosage_information.duration_value` / `duration_unit` | `PUT /api/cases/{case_id}/drugs/{drug_id}/dosages/{id}` | `//effectiveTime/comp[@operator='A']/width/@value` + `@unit` | XSD | |
| G.k.4.r Dosage Text | `dosage_information.dosage_text` / `drug_information.dosage_text` | `PUT /api/cases/{case_id}/drugs/{drug_id}/dosages/{id}` | `//substanceAdministration/text` | XSD | Falls back to drug-level dosage text. |
| G.k.4.r Dose Form | `dosage_information.dose_form` | `PUT /api/cases/{case_id}/drugs/{drug_id}/dosages/{id}` | `//kindOfProduct/formCode/originalText` | XSD | |
| G.k.4.r Dose Form TermID/Version | `dosage_information.dose_form_termid` / `dose_form_termid_version` | `PUT /api/cases/{case_id}/drugs/{drug_id}/dosages/{id}` | `//kindOfProduct/formCode/@code` + `@codeSystemVersion` | XSD | REST update exists via dosages endpoint. |
| G.k.4.r Route of Administration | `dosage_information.route_of_administration` | `PUT /api/cases/{case_id}/drugs/{drug_id}/dosages/{id}` | `//routeCode/@code` | XSD | |
| G.k.4.r Parent Route | `dosage_information.parent_route` / `parent_route_termid` / `parent_route_termid_version` | `PUT /api/cases/{case_id}/drugs/{drug_id}/dosages/{id}` | `//outboundRelationship2/observation[code='G.k.4.r.11']/value` | XSD | TermID/version not in update struct. |
| G.k.5 Dosage Text (drug-level) | `drug_information.dosage_text` | `PUT /api/cases/{case_id}/drugs/{id}` | `//substanceAdministration/text` | XSD | REST update exists via drugs endpoint. |
| G.k.6.r Indication | `drug_indications.indication_meddra_code/version/text` | `POST/PUT /api/cases/{case_id}/drugs/{drug_id}/indications` | `//inboundRelationship[@typeCode='RSON']/observation/value` | XSD | Repeats. |
| G.k.7 Action Taken | `drug_information.action_taken` | `PUT /api/cases/{case_id}/drugs/{id}` | `//inboundRelationship[@typeCode='CAUS']/act/code/@code` | XSD | |
| G.k.8 Rechallenge | `drug_information.rechallenge` | `PUT /api/cases/{case_id}/drugs/{id}` | `//outboundRelationship2/observation[code='31']/value/@code` | XSD | REST update exists via drugs endpoint. |
| G.k.8.r Recurrence (structured) | `drug_recurrence_information.*` | `POST/PUT /api/cases/{case_id}/drugs/{drug_id}/recurrences` | `//outboundRelationship2[@typeCode='PERT']/observation[code='31']/value/@code` | XSD | Exporter only uses `reaction_recurred`; ignores action/MedDRA. |
| G.k.9.i Drug‑Reaction Assessment (link) | `drug_reaction_assessments.*` | `POST/PUT /api/cases/{case_id}/drugs/{drug_id}/reaction-assessments` | `//outboundRelationship2[@typeCode='PERT']/observation[code='31']/outboundRelationship1/actReference/id` | XSD | Links drug↔reaction via actReference id root. |
| G.k.9.i.2.r Relatedness Assessment | `relatedness_assessments.*` | `POST/PUT /api/cases/{case_id}/drugs/{drug_id}/reaction-assessments/{assessment_id}/relatedness` | `//component/causalityAssessment[code='39']/*` | XSD | Exports source/method/result and links drug/reaction ids. |
| FDA.G.k.10a Additional Info on Drug (coded) | `drug_information.fda_additional_info_coded` | `PUT /api/cases/{case_id}/drugs/{id}` | `//outboundRelationship2[@typeCode='REFR']/observation[code='9']/value/@code` | custom (FDA.G.k.10a) | Exported when present; validator enforces when Pre‑ANDA present. |

## Gap Mapping (Phase 1) — H Section (Narrative and Other Information)

| Data Element | DB Table.Column | API Endpoint | Exporter XPath | Validator Rule | Notes |
|---|---|---|---|---|---|
| H.1 Case Narrative | `narrative_information.case_narrative` | `POST/PUT /api/cases/{case_id}/narrative` | `//investigationEvent/text` | XSD | Required. |
| H.2 Reporter's Comments | `narrative_information.reporter_comments` | `PUT /api/cases/{case_id}/narrative` | `//component1/observationEvent[author/assignedEntity/code='3']/value` | XSD | |
| H.3.r.1a MedDRA Version | `sender_diagnoses.diagnosis_meddra_version` | `PUT /api/cases/{case_id}/narrative/sender-diagnoses/{id}` | `//component1/observationEvent[code='15']/value/@codeSystemVersion` | XSD | Repeats. |
| H.3.r.1b Diagnosis (MedDRA code) | `sender_diagnoses.diagnosis_meddra_code` | `POST/PUT /api/cases/{case_id}/narrative/sender-diagnoses` | `//component1/observationEvent[code='15']/value/@code` | XSD | Repeats. |
| H.4 Sender's Comments | `narrative_information.sender_comments` | `PUT /api/cases/{case_id}/narrative` | `//component1/observationEvent[author/assignedEntity/code='1']/value` | XSD | |
| H.5.r.1 Case Summary Type | `case_summary_information.summary_type` | `PUT /api/cases/{case_id}/narrative/summaries/{id}` | `//component/observationEvent[code='36']/author/assignedEntity/code/@code` | XSD | Exported for each summary. |
| H.5.r.2 Case Summary Language | `case_summary_information.language_code` | `PUT /api/cases/{case_id}/narrative/summaries/{id}` | `//component/observationEvent[code='36']/value/@language` | XSD | |
| H.5.r.3 Case Summary Text | `case_summary_information.summary_text` | `POST/PUT /api/cases/{case_id}/narrative/summaries` | `//component/observationEvent[code='36']/value` | XSD | Exporter only takes first summary. |
