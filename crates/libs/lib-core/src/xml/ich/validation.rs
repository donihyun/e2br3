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
	build_report, has_text, push_issue_by_code, CaseValidationReport,
	ValidationIssue, ValidationProfile,
};
use sqlx::types::Uuid;

fn has_any_primary_source_content(source: &PrimarySource) -> bool {
	has_text(source.reporter_title.as_deref())
		|| has_text(source.reporter_given_name.as_deref())
		|| has_text(source.reporter_middle_name.as_deref())
		|| has_text(source.reporter_family_name.as_deref())
		|| has_text(source.organization.as_deref())
		|| has_text(source.department.as_deref())
		|| has_text(source.street.as_deref())
		|| has_text(source.city.as_deref())
		|| has_text(source.state.as_deref())
		|| has_text(source.postcode.as_deref())
		|| has_text(source.telephone.as_deref())
		|| has_text(source.country_code.as_deref())
		|| has_text(source.email.as_deref())
		|| has_text(source.qualification.as_deref())
		|| has_text(source.primary_source_regulatory.as_deref())
}

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
		push_issue_by_code(&mut issues, "ICH.C.1.REQUIRED", "safetyReportIdentification");
	}

	if header.is_none() {
		push_issue_by_code(&mut issues, "ICH.N.REQUIRED", "messageHeader");
	}

	if let Some(report) = report.as_ref() {
		if report.report_type.trim().is_empty() {
			push_issue_by_code(
				&mut issues,
				"ICH.C.1.3.REQUIRED",
				"safetyReportIdentification.reportType",
			);
		}
	}

	primary_sources
		.iter()
		.enumerate()
		.for_each(|(idx, source)| {
			if !has_any_primary_source_content(source) {
				return;
			}
			if !has_text(source.qualification.as_deref()) {
				push_issue_by_code(
					&mut issues,
					"ICH.C.2.r.4.REQUIRED",
					format!("primarySources.{idx}.qualification"),
				);
			}
		});

	if let Some(patient) = patient.as_ref() {
		let has_patient_payload = has_text(patient.patient_given_name.as_deref())
			|| has_text(patient.patient_family_name.as_deref())
			|| patient.birth_date.is_some()
			|| patient.age_at_time_of_onset.is_some()
			|| patient.sex.is_some();
		if has_patient_payload && !has_text(patient.patient_initials.as_deref()) {
			push_issue_by_code(
				&mut issues,
				"ICH.D.1.REQUIRED",
				"patientInformation.patientInitials",
			);
		}
	}

	reactions.iter().enumerate().for_each(|(idx, reaction)| {
		if reaction.primary_source_reaction.trim().is_empty() {
			push_issue_by_code(
				&mut issues,
				"ICH.E.i.1.1a.REQUIRED",
				format!("reactions.{idx}.primarySourceReaction"),
			);
		}
		if !has_text(reaction.outcome.as_deref()) {
			push_issue_by_code(
				&mut issues,
				"ICH.E.i.7.REQUIRED",
				format!("reactions.{idx}.reactionOutcome"),
			);
		}
	});

	drugs.iter().enumerate().for_each(|(idx, drug)| {
		if drug.drug_characterization.trim().is_empty() {
			push_issue_by_code(
				&mut issues,
				"ICH.G.k.1.REQUIRED",
				format!("drugs.{idx}.drugCharacterization"),
			);
		}
		if drug.medicinal_product.trim().is_empty() {
			push_issue_by_code(
				&mut issues,
				"ICH.G.k.2.2.REQUIRED",
				format!("drugs.{idx}.medicinalProduct"),
			);
		}
	});

	if let Some(narrative) = narrative.as_ref() {
		let has_narrative_payload = has_text(narrative.reporter_comments.as_deref())
			|| has_text(narrative.sender_comments.as_deref())
			|| !narrative.case_narrative.trim().is_empty();
		if has_narrative_payload && narrative.case_narrative.trim().is_empty() {
			push_issue_by_code(
				&mut issues,
				"ICH.H.1.REQUIRED",
				"narrative.caseNarrative",
			);
		}
	}

	Ok(build_report(ValidationProfile::Ich, case_id, issues))
}
