use crate::ctx::Ctx;
use crate::model::case::{Case, CaseBmc};
use crate::model::drug::DrugInformation;
use crate::model::message_header::MessageHeader;
use crate::model::narrative::NarrativeInformation;
use crate::model::patient::PatientInformation;
use crate::model::reaction::Reaction;
use crate::model::safety_report::{PrimarySource, SafetyReportIdentification};
use crate::model::{ModelManager, Result};
use crate::xml::validate::{
	build_report, has_any_primary_source_content, has_patient_initials,
	push_issue_by_code, push_issue_if_rule_invalid, should_require_case_narrative,
	should_require_patient_initials, CaseValidationReport, RuleFacts,
	ValidationIssue, ValidationProfile, CASE_RULE_ICH_C13_REQUIRED,
	CASE_RULE_ICH_C1_REQUIRED, CASE_RULE_ICH_C2R4_REQUIRED,
	CASE_RULE_ICH_D1_REQUIRED, CASE_RULE_ICH_EI11A_REQUIRED,
	CASE_RULE_ICH_EI7_REQUIRED, CASE_RULE_ICH_GK1_REQUIRED,
	CASE_RULE_ICH_GK22_REQUIRED, CASE_RULE_ICH_H1_REQUIRED,
	CASE_RULE_ICH_N_REQUIRED,
};
use sqlx::types::Uuid;

async fn get_safety_report_optional(
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Option<SafetyReportIdentification>> {
	let sql = "SELECT * FROM safety_report_identification WHERE case_id = $1";
	mm.dbx()
		.fetch_optional(
			sqlx::query_as::<_, SafetyReportIdentification>(sql).bind(case_id),
		)
		.await
		.map_err(Into::into)
}

async fn get_message_header_optional(
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Option<MessageHeader>> {
	let sql = "SELECT * FROM message_headers WHERE case_id = $1";
	mm.dbx()
		.fetch_optional(sqlx::query_as::<_, MessageHeader>(sql).bind(case_id))
		.await
		.map_err(Into::into)
}

async fn get_patient_optional(
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Option<PatientInformation>> {
	let sql = "SELECT * FROM patient_information WHERE case_id = $1";
	mm.dbx()
		.fetch_optional(sqlx::query_as::<_, PatientInformation>(sql).bind(case_id))
		.await
		.map_err(Into::into)
}

async fn get_narrative_optional(
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Option<NarrativeInformation>> {
	let sql = "SELECT * FROM narrative_information WHERE case_id = $1";
	mm.dbx()
		.fetch_optional(sqlx::query_as::<_, NarrativeInformation>(sql).bind(case_id))
		.await
		.map_err(Into::into)
}

async fn list_primary_sources(
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Vec<PrimarySource>> {
	let sql =
		"SELECT * FROM primary_sources WHERE case_id = $1 ORDER BY sequence_number";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, PrimarySource>(sql).bind(case_id))
		.await
		.map_err(Into::into)
}

pub async fn validate_case(
	ctx: &Ctx,
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<CaseValidationReport> {
	let _case: Case = CaseBmc::get(ctx, mm, case_id).await?;

	let report = get_safety_report_optional(mm, case_id).await?;
	let header = get_message_header_optional(mm, case_id).await?;
	let patient = get_patient_optional(mm, case_id).await?;
	let narrative = get_narrative_optional(mm, case_id).await?;
	let primary_sources = list_primary_sources(mm, case_id).await?;
	let reactions: Vec<Reaction> =
		crate::model::reaction::ReactionBmc::list_by_case(ctx, mm, case_id).await?;
	let drugs: Vec<DrugInformation> =
		crate::model::drug::DrugInformationBmc::list_by_case(ctx, mm, case_id)
			.await?;

	let mut issues: Vec<ValidationIssue> = Vec::new();

	if report.is_none() {
		push_issue_by_code(
			&mut issues,
			CASE_RULE_ICH_C1_REQUIRED,
			"safetyReportIdentification",
		);
	}

	if header.is_none() {
		push_issue_by_code(&mut issues, CASE_RULE_ICH_N_REQUIRED, "messageHeader");
	}

	if let Some(report) = report.as_ref() {
		let _ = push_issue_if_rule_invalid(
			&mut issues,
			CASE_RULE_ICH_C13_REQUIRED,
			"safetyReportIdentification.reportType",
			Some(report.report_type.as_str()),
			None,
			RuleFacts::default(),
		);
	}

	primary_sources
		.iter()
		.enumerate()
		.for_each(|(idx, source)| {
			if !has_any_primary_source_content(source) {
				return;
			}
			let _ = push_issue_if_rule_invalid(
				&mut issues,
				CASE_RULE_ICH_C2R4_REQUIRED,
				format!("primarySources.{idx}.qualification"),
				source.qualification.as_deref(),
				None,
				RuleFacts::default(),
			);
		});

	if let Some(patient) = patient.as_ref() {
		if should_require_patient_initials(patient) && !has_patient_initials(patient)
		{
			push_issue_by_code(
				&mut issues,
				CASE_RULE_ICH_D1_REQUIRED,
				"patientInformation.patientInitials",
			);
		}
	}

	reactions.iter().enumerate().for_each(|(idx, reaction)| {
		let _ = push_issue_if_rule_invalid(
			&mut issues,
			CASE_RULE_ICH_EI11A_REQUIRED,
			format!("reactions.{idx}.primarySourceReaction"),
			Some(reaction.primary_source_reaction.as_str()),
			None,
			RuleFacts::default(),
		);
		let _ = push_issue_if_rule_invalid(
			&mut issues,
			CASE_RULE_ICH_EI7_REQUIRED,
			format!("reactions.{idx}.reactionOutcome"),
			reaction.outcome.as_deref(),
			None,
			RuleFacts::default(),
		);
	});

	drugs.iter().enumerate().for_each(|(idx, drug)| {
		let _ = push_issue_if_rule_invalid(
			&mut issues,
			CASE_RULE_ICH_GK1_REQUIRED,
			format!("drugs.{idx}.drugCharacterization"),
			Some(drug.drug_characterization.as_str()),
			None,
			RuleFacts::default(),
		);
		let _ = push_issue_if_rule_invalid(
			&mut issues,
			CASE_RULE_ICH_GK22_REQUIRED,
			format!("drugs.{idx}.medicinalProduct"),
			Some(drug.medicinal_product.as_str()),
			None,
			RuleFacts::default(),
		);
	});

	if let Some(narrative) = narrative.as_ref() {
		if should_require_case_narrative(narrative) {
			let _ = push_issue_if_rule_invalid(
				&mut issues,
				CASE_RULE_ICH_H1_REQUIRED,
				"narrative.caseNarrative",
				Some(narrative.case_narrative.as_str()),
				None,
				RuleFacts::default(),
			);
		}
	}

	Ok(build_report(ValidationProfile::Ich, case_id, issues))
}
