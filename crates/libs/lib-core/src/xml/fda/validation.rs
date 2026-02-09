use crate::ctx::Ctx;
use crate::model::reaction::Reaction;
use crate::model::safety_report::{PrimarySource, SafetyReportIdentification};
use crate::model::{ModelManager, Result};
use crate::xml::validate::{
	build_report, has_text, push_issue_by_code, CaseValidationReport, ValidationIssue,
	ValidationProfile,
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
	let ich_report = crate::xml::ich::validation::validate_case(ctx, mm, case_id).await?;

	let report = get_safety_report_optional(mm, case_id).await?;
	let primary_sources = list_primary_sources(mm, case_id).await?;
	let reactions: Vec<Reaction> =
		crate::model::reaction::ReactionBmc::list_by_case(ctx, mm, case_id).await?;
	let mut issues: Vec<ValidationIssue> = ich_report.issues;

	if let Some(report) = report.as_ref() {
		if report.fulfil_expedited_criteria
			&& !has_text(report.local_criteria_report_type.as_deref())
		{
			push_issue_by_code(
				&mut issues,
				"FDA.C.1.7.1.REQUIRED",
				"safetyReportIdentification.localCriteriaReportType",
			);
		}
		if !has_text(report.combination_product_report_indicator.as_deref()) {
			push_issue_by_code(
				&mut issues,
				"FDA.C.1.12.RECOMMENDED",
				"safetyReportIdentification.combinationProductReportIndicator",
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
			if !has_text(source.email.as_deref()) {
				push_issue_by_code(
					&mut issues,
					"FDA.C.2.r.2.EMAIL.REQUIRED",
					&format!("primarySources.{idx}.reporterEmail"),
				);
			}
		});

	reactions.iter().enumerate().for_each(|(idx, reaction)| {
		let needs_required_intervention = reaction.criteria_other_medically_important;
		if needs_required_intervention
			&& !has_text(reaction.required_intervention.as_deref())
		{
			push_issue_by_code(
				&mut issues,
				"FDA.E.i.3.2h.REQUIRED",
				&format!("reactions.{idx}.requiredIntervention"),
			);
		}
	});

	Ok(build_report(ValidationProfile::Fda, case_id, issues))
}
