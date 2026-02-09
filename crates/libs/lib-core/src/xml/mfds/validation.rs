use crate::ctx::Ctx;
use crate::model::drug::{DrugActiveSubstance, DrugInformation};
use crate::model::safety_report::SenderInformation;
use crate::model::{ModelManager, Result};
use crate::xml::validate::{
	build_report, has_text, push_issue_by_code, CaseValidationReport,
	ValidationIssue, ValidationProfile,
};
use sqlx::types::Uuid;

async fn list_active_substances_by_case(
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Vec<DrugActiveSubstance>> {
	let sql = r#"
SELECT das.*
FROM drug_active_substances das
JOIN drug_information di ON di.id = das.drug_id
WHERE di.case_id = $1
ORDER BY di.sequence_number, das.sequence_number
"#;
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, DrugActiveSubstance>(sql).bind(case_id))
		.await
		.map_err(Into::into)
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct RelatednessWithDrug {
	pub drug_id: Uuid,
	pub relatedness_sequence_number: i32,
	pub source_of_assessment: Option<String>,
	pub method_of_assessment: Option<String>,
	pub result_of_assessment: Option<String>,
}

async fn list_relatedness_by_case(
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Vec<RelatednessWithDrug>> {
	let sql = r#"
SELECT di.id as drug_id
     , ra.sequence_number as relatedness_sequence_number
     , ra.source_of_assessment
     , ra.method_of_assessment
     , ra.result_of_assessment
FROM relatedness_assessments ra
JOIN drug_reaction_assessments dra ON dra.id = ra.drug_reaction_assessment_id
JOIN drug_information di ON di.id = dra.drug_id
WHERE di.case_id = $1
ORDER BY di.sequence_number, ra.sequence_number
"#;
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, RelatednessWithDrug>(sql).bind(case_id))
		.await
		.map_err(Into::into)
}

async fn list_senders_by_case(
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Vec<SenderInformation>> {
	let sql = "SELECT * FROM sender_information WHERE case_id = $1 ORDER BY created_at";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, SenderInformation>(sql).bind(case_id))
		.await
		.map_err(Into::into)
}

pub async fn validate_case(
	ctx: &Ctx,
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<CaseValidationReport> {
	let ich_report = crate::xml::ich::validation::validate_case(ctx, mm, case_id).await?;
	let drugs: Vec<DrugInformation> =
		crate::model::drug::DrugInformationBmc::list_by_case(ctx, mm, case_id)
			.await?;
	let senders = list_senders_by_case(mm, case_id).await?;
	let active_substances = list_active_substances_by_case(mm, case_id).await?;
	let relatedness = list_relatedness_by_case(mm, case_id).await?;

	let mut issues: Vec<ValidationIssue> = ich_report.issues;

	// MFDS-specific checks (KR profile): only rules backed by persisted fields.
	senders.iter().enumerate().for_each(|(idx, sender)| {
		if sender.sender_type.trim() == "3" {
			push_issue_by_code(
				&mut issues,
				"MFDS.C.3.1.KR.1.REQUIRED",
				format!("senderInformation.{idx}.senderType"),
			);
		}
	});

	let mut domestic_drug_ids = std::collections::HashSet::new();
	let mut drug_index_by_id = std::collections::HashMap::new();

	drugs.iter().enumerate().for_each(|(idx, drug)| {
		drug_index_by_id.insert(drug.id, idx);
		let country = drug.obtain_drug_country.as_deref().map(str::trim);
		match country {
			Some("KR") => {
				domestic_drug_ids.insert(drug.id);
				if !has_text(drug.mpid.as_deref()) {
					push_issue_by_code(
						&mut issues,
						"MFDS.KR.DOMESTIC.PRODUCTCODE.REQUIRED",
						format!("drugs.{idx}.mpid"),
					);
				}
			}
			Some(other) if !other.is_empty() => {
				if !has_text(drug.mpid.as_deref()) {
					push_issue_by_code(
						&mut issues,
						"MFDS.KR.FOREIGN.WHOMPID.RECOMMENDED",
						format!("drugs.{idx}.mpid"),
					);
				}
			}
			_ => {}
		}
	});

	active_substances.iter().for_each(|substance| {
		if domestic_drug_ids.contains(&substance.drug_id)
			&& !has_text(substance.substance_termid.as_deref())
		{
			let drug_index = drug_index_by_id.get(&substance.drug_id).copied();
			let substance_index = substance
				.sequence_number
				.checked_sub(1)
				.and_then(|v| usize::try_from(v).ok());
			let path = match (drug_index, substance_index) {
				(Some(d_idx), Some(s_idx)) => {
					format!("drugs.{d_idx}.activeSubstances.{s_idx}.substanceTermId")
				}
				_ => "drugs".to_string(),
			};
			push_issue_by_code(
				&mut issues,
				"MFDS.KR.DOMESTIC.INGREDIENTCODE.REQUIRED",
				path,
			);
		}
	});

	relatedness.iter().for_each(|r| {
		let has_source = has_text(r.source_of_assessment.as_deref());
		let has_method = has_text(r.method_of_assessment.as_deref());
		let has_result = has_text(r.result_of_assessment.as_deref());
		let drug_index = drug_index_by_id.get(&r.drug_id).copied();
		let assess_index = r
			.relatedness_sequence_number
			.checked_sub(1)
			.and_then(|v| usize::try_from(v).ok());
		let path_for = |field: &str| match (drug_index, assess_index) {
			(Some(d_idx), Some(a_idx)) => {
				format!("drugs.{d_idx}.drugReactionAssessments.{a_idx}.{field}")
			}
			_ => "drugs".to_string(),
		};

		if has_source && !has_method {
			push_issue_by_code(
				&mut issues,
				"MFDS.G.k.9.i.2.r.2.KR.1.REQUIRED",
				path_for("methodOfAssessment"),
			);
		}
		if has_source && !has_result {
			push_issue_by_code(
				&mut issues,
				"MFDS.G.k.9.i.2.r.3.KR.1.REQUIRED",
				path_for("resultOfAssessment"),
			);
		}
		if !has_source && (has_method || has_result) {
			push_issue_by_code(
				&mut issues,
				"MFDS.G.k.9.i.2.r.1.REQUIRED",
				path_for("sourceOfAssessment"),
			);
		}
	});

	Ok(build_report(ValidationProfile::Mfds, case_id, issues))
}
