# BMCs and Models Summary

Source: `crates/libs/lib-core/src/model`

This document summarizes the current model structs and their BMCs (Backend Model Controllers).

## Modules

**audit**
Entities: CaseVersion, AuditLog
DTOs: CaseVersionForCreate, AuditLogForCreate, AuditLogFilter
BMCs: CaseVersionBmc, AuditLogBmc

**case**
Entities: Case
DTOs: CaseForCreate, CaseForUpdate, CaseFilter
BMCs: CaseBmc

**case_identifiers**
Entities: OtherCaseIdentifier, LinkedReportNumber
DTOs: OtherCaseIdentifierForCreate, OtherCaseIdentifierForUpdate, OtherCaseIdentifierFilter, LinkedReportNumberForCreate, LinkedReportNumberForUpdate, LinkedReportNumberFilter
BMCs: OtherCaseIdentifierBmc, LinkedReportNumberBmc

**drug**
Entities: DrugInformation, DrugActiveSubstance, DosageInformation, DrugIndication, DrugDeviceCharacteristic
DTOs: DrugInformationForCreate, DrugInformationForUpdate, DrugActiveSubstanceForCreate, DrugActiveSubstanceForUpdate, DrugActiveSubstanceFilter, DosageInformationForCreate, DosageInformationForUpdate, DosageInformationFilter, DrugIndicationForCreate, DrugIndicationForUpdate, DrugIndicationFilter, DrugDeviceCharacteristicForCreate, DrugDeviceCharacteristicForUpdate, DrugDeviceCharacteristicFilter
BMCs: DrugInformationBmc, DrugActiveSubstanceBmc, DosageInformationBmc, DrugIndicationBmc, DrugDeviceCharacteristicBmc

**drug_reaction_assessment**
Entities: DrugReactionAssessment, RelatednessAssessment
DTOs: DrugReactionAssessmentForCreate, DrugReactionAssessmentForUpdate, DrugReactionAssessmentFilter, RelatednessAssessmentForCreate, RelatednessAssessmentForUpdate, RelatednessAssessmentFilter
BMCs: DrugReactionAssessmentBmc, RelatednessAssessmentBmc

**drug_recurrence**
Entities: DrugRecurrenceInformation
DTOs: DrugRecurrenceInformationForCreate, DrugRecurrenceInformationForUpdate, DrugRecurrenceInformationFilter
BMCs: DrugRecurrenceInformationBmc

**message_header**
Entities: MessageHeader
DTOs: MessageHeaderForCreate, MessageHeaderForUpdate
BMCs: MessageHeaderBmc

**narrative**
Entities: NarrativeInformation, SenderDiagnosis, CaseSummaryInformation
DTOs: NarrativeInformationForCreate, NarrativeInformationForUpdate, SenderDiagnosisForCreate, SenderDiagnosisForUpdate, SenderDiagnosisFilter, CaseSummaryInformationForCreate, CaseSummaryInformationForUpdate, CaseSummaryInformationFilter
BMCs: NarrativeInformationBmc, SenderDiagnosisBmc, CaseSummaryInformationBmc

**organization**
Entities: Organization
DTOs: OrganizationForCreate, OrganizationForUpdate, OrganizationFilter
BMCs: OrganizationBmc

**parent_history**
Entities: ParentMedicalHistory, ParentPastDrugHistory
DTOs: ParentMedicalHistoryForCreate, ParentMedicalHistoryForUpdate, ParentMedicalHistoryFilter, ParentPastDrugHistoryForCreate, ParentPastDrugHistoryForUpdate, ParentPastDrugHistoryFilter
BMCs: ParentMedicalHistoryBmc, ParentPastDrugHistoryBmc

**patient**
Entities: PatientInformation, PatientIdentifier, MedicalHistoryEpisode, PastDrugHistory, PatientDeathInformation, ReportedCauseOfDeath, AutopsyCauseOfDeath, ParentInformation
DTOs: PatientInformationForCreate, PatientInformationForUpdate, PatientInformationFilter, PatientIdentifierForCreate, PatientIdentifierForUpdate, PatientIdentifierFilter, MedicalHistoryEpisodeForCreate, MedicalHistoryEpisodeForUpdate, MedicalHistoryEpisodeFilter, PastDrugHistoryForCreate, PastDrugHistoryForUpdate, PastDrugHistoryFilter, PatientDeathInformationForCreate, PatientDeathInformationForUpdate, PatientDeathInformationFilter, ReportedCauseOfDeathForCreate, ReportedCauseOfDeathForUpdate, ReportedCauseOfDeathFilter, AutopsyCauseOfDeathForCreate, AutopsyCauseOfDeathForUpdate, AutopsyCauseOfDeathFilter, ParentInformationForCreate, ParentInformationForUpdate, ParentInformationFilter
BMCs: PatientInformationBmc, PatientIdentifierBmc, MedicalHistoryEpisodeBmc, PastDrugHistoryBmc, PatientDeathInformationBmc, ReportedCauseOfDeathBmc, AutopsyCauseOfDeathBmc, ParentInformationBmc

**reaction**
Entities: Reaction
DTOs: ReactionForCreate, ReactionForUpdate, ReactionFilter
BMCs: ReactionBmc

**receiver**
Entities: ReceiverInformation
DTOs: ReceiverInformationForCreate, ReceiverInformationForUpdate
BMCs: ReceiverInformationBmc

**safety_report**
Entities: SafetyReportIdentification, SenderInformation, PrimarySource, LiteratureReference, DocumentsHeldBySender, StudyInformation, StudyRegistrationNumber
DTOs: SafetyReportIdentificationForCreate, SafetyReportIdentificationForUpdate, SenderInformationForCreate, SenderInformationForUpdate, SenderInformationFilter, PrimarySourceForCreate, PrimarySourceForUpdate, PrimarySourceFilter, LiteratureReferenceForCreate, LiteratureReferenceForUpdate, LiteratureReferenceFilter, DocumentsHeldBySenderForCreate, DocumentsHeldBySenderForUpdate, DocumentsHeldBySenderFilter, StudyInformationForCreate, StudyInformationForUpdate, StudyInformationFilter, StudyRegistrationNumberForCreate, StudyRegistrationNumberForUpdate, StudyRegistrationNumberFilter
BMCs: SafetyReportIdentificationBmc, SenderInformationBmc, PrimarySourceBmc, LiteratureReferenceBmc, DocumentsHeldBySenderBmc, StudyInformationBmc, StudyRegistrationNumberBmc

**terminology**
Entities: MeddraTerm, WhodrugProduct, IsoCountry, E2bCodeList
DTOs: MeddraTermForCreate, MeddraTermFilter, WhodrugProductForCreate, WhodrugProductFilter, IsoCountryForCreate, E2bCodeListForCreate, E2bCodeListFilter
BMCs: MeddraTermBmc, WhodrugProductBmc, IsoCountryBmc, E2bCodeListBmc

**test_result**
Entities: TestResult
DTOs: TestResultForCreate, TestResultForUpdate
BMCs: TestResultBmc

**user**
Entities: User
DTOs: UserForCreate, UserForInsert, UserForLogin, UserForAuth, UserForUpdate, UserFilter
BMCs: UserBmc

## Presave UI Mapping (Suggested)
Use existing models where possible to avoid duplicating schema and validation.

1. Sender: `safety_report::SenderInformation` via `SenderInformationBmc`
2. Receiver: `receiver::ReceiverInformation` via `ReceiverInformationBmc`
3. Product: `drug::DrugInformation` via `DrugInformationBmc` (plus submodels for active substances, dosage, indications)
4. Study: `safety_report::StudyInformation` and `StudyRegistrationNumber` via their BMCs
5. Reporter: `safety_report::PrimarySource` via `PrimarySourceBmc`
6. Narrative: `narrative::NarrativeInformation` via `NarrativeInformationBmc` (plus optional `SenderDiagnosis` and `CaseSummaryInformation`)

If the presave section needs to store partial drafts independently of cases, consider a new lightweight draft table that references these entities or stores JSON snapshots, but only if the existing tables cannot support incomplete data.
