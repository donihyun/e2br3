use crate::ctx::Ctx;
use crate::model::case::CaseBmc;
use crate::model::case_identifiers::{LinkedReportNumber, OtherCaseIdentifier};
use crate::model::patient::{
	AutopsyCauseOfDeath, MedicalHistoryEpisode, ParentInformation, PastDrugHistory,
	PatientDeathInformation, PatientIdentifier, PatientInformation, ReportedCauseOfDeath,
};
use crate::model::reaction::Reaction;
use crate::model::safety_report::{
	DocumentsHeldBySender, LiteratureReference, PrimarySource,
	SafetyReportIdentificationBmc, SenderInformation, StudyInformation,
	StudyRegistrationNumber,
};
use crate::model::test_result::TestResult;
use crate::model::drug::{
	DrugActiveSubstance, DrugIndication, DrugInformation, DosageInformation,
};
use crate::model::narrative::{
	CaseSummaryInformation, NarrativeInformation, SenderDiagnosis,
};
use crate::model::message_header::MessageHeader;
use crate::model::parent_history::{ParentMedicalHistory, ParentPastDrugHistory};
use crate::model::drug_recurrence::DrugRecurrenceInformation;
use crate::model::drug_reaction_assessment::{
	DrugReactionAssessment, RelatednessAssessment,
};
use crate::model::ModelManager;
use crate::xml::error::Error;
use crate::xml::Result;
use libxml::parser::Parser;
use libxml::tree::{Document, Node, NodeType};
use libxml::xpath::Context;
use sqlx::types::time::Date;
use std::path::PathBuf;

pub async fn export_case_xml(
	ctx: &Ctx,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<String> {
	let case = CaseBmc::get(ctx, mm, case_id).await.map_err(Error::from)?;
	if case.status != "validated" {
		return Err(Error::InvalidXml {
			message: "Only validated cases can be exported".to_string(),
			line: None,
			column: None,
		});
	}

	export_case_xml_from_db(ctx, mm, case_id).await
}

async fn export_case_xml_from_db(
	ctx: &Ctx,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<String> {
	let case = CaseBmc::get(ctx, mm, case_id)
		.await
		.map_err(Error::from)?;
	let template_path = template_path_from_env().ok_or(Error::InvalidXml {
		message: "E2BR3_EXPORT_TEMPLATE not set".to_string(),
		line: None,
		column: None,
	})?;
	let xml_str = std::fs::read_to_string(&template_path)?;
	let parser = Parser::default();
	let mut doc = parser.parse_string(&xml_str).map_err(|err| Error::InvalidXml {
		message: format!("XML parse error: {err}"),
		line: None,
		column: None,
	})?;

	let mut xpath = Context::new(&doc).map_err(|_| Error::InvalidXml {
		message: "Failed to initialize XPath context".to_string(),
		line: None,
		column: None,
	})?;
	let _ = xpath.register_namespace("hl7", "urn:hl7-org:v3");
	let _ = xpath.register_namespace("xsi", "http://www.w3.org/2001/XMLSchema-instance");

	apply_section_c(
		&mut doc,
		&parser,
		ctx,
		mm,
		case_id,
		&case.safety_report_id,
		&mut xpath,
	)
	.await?;
	apply_section_d(&mut doc, &parser, mm, case_id, &mut xpath).await?;
	apply_section_e(&mut doc, &parser, mm, case_id, &mut xpath).await?;
	apply_section_f(&mut doc, &parser, mm, case_id, &mut xpath).await?;
	apply_section_g(&mut doc, &parser, mm, case_id, &mut xpath).await?;
	apply_section_h(&mut doc, &parser, mm, case_id, &mut xpath).await?;
	apply_section_n(&mut doc, &parser, mm, case_id, &mut xpath).await?;

	let mut xml = doc.to_string();
	if let Some(header) = fetch_message_header(mm, case_id).await? {
		let has_batch_sender = xpath
			.findnodes(
				"/hl7:MCCI_IN200100UV01/hl7:sender/hl7:device/hl7:id",
				None,
			)
			.map(|nodes| !nodes.is_empty())
			.unwrap_or(false);
		let has_batch_receiver = xpath
			.findnodes(
				"/hl7:MCCI_IN200100UV01/hl7:receiver/hl7:device/hl7:id",
				None,
			)
			.map(|nodes| !nodes.is_empty())
			.unwrap_or(false);
		if (!has_batch_sender && header.batch_sender_identifier.is_some())
			|| (!has_batch_receiver && header.batch_receiver_identifier.is_some())
		{
			let mut insert = String::new();
			if let Some(batch_receiver) =
				header.batch_receiver_identifier.as_deref()
			{
				insert.push_str(&format!(
					"  <receiver typeCode=\"RCV\"><device classCode=\"DEV\" determinerCode=\"INSTANCE\"><id extension=\"{batch_receiver}\" root=\"2.16.840.1.113883.3.989.2.1.3.14\"/></device></receiver>\n"
				));
			}
			if let Some(batch_sender) =
				header.batch_sender_identifier.as_deref()
			{
				insert.push_str(&format!(
					"  <sender typeCode=\"SND\"><device classCode=\"DEV\" determinerCode=\"INSTANCE\"><id extension=\"{batch_sender}\" root=\"2.16.840.1.113883.3.989.2.1.3.13\"/></device></sender>\n"
				));
			}
			if let Some(pos) = xml.rfind("</MCCI_IN200100UV01>") {
				xml.insert_str(pos, &insert);
			}
		}
	}

	Ok(xml)
}

fn template_path_from_env() -> Option<PathBuf> {
	std::env::var("E2BR3_EXPORT_TEMPLATE")
		.ok()
		.map(PathBuf::from)
}

async fn apply_section_c(
	doc: &mut Document,
	parser: &Parser,
	ctx: &Ctx,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	safety_report_id: &str,
	xpath: &mut Context,
) -> Result<()> {
	let report = SafetyReportIdentificationBmc::get_by_case(ctx, mm, case_id)
		.await
		.map_err(Error::from)?;

	set_attr_all(
		xpath,
		"//hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.1']",
		"extension",
		safety_report_id,
	);
	if let Some(worldwide_unique_id) = report.worldwide_unique_id.as_deref() {
		set_attr_all(
			xpath,
			"//hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.2']",
			"extension",
			worldwide_unique_id,
		);
	}

	set_attr_first(
		xpath,
		"//hl7:controlActProcess/hl7:effectiveTime",
		"value",
		&fmt_date(report.transmission_date),
	);
	set_attr_first(
		xpath,
		"//hl7:investigationEvent/hl7:effectiveTime/hl7:low",
		"value",
		&fmt_date(report.date_first_received_from_source),
	);
	set_attr_first(
		xpath,
		"//hl7:investigationEvent/hl7:availabilityTime",
		"value",
		&fmt_date(report.date_of_most_recent_information),
	);
	set_attr_first(
		xpath,
		"//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.23']]/hl7:value",
		"code",
		&report.report_type,
	);
	set_attr_first(
		xpath,
		"//hl7:component/hl7:observationEvent[hl7:code[@code='23' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value",
		"value",
		if report.fulfil_expedited_criteria { "true" } else { "false" },
	);

	let other_ids = fetch_other_case_identifiers(mm, case_id).await?;
	if !other_ids.is_empty() {
		let base = "//hl7:investigationEvent/hl7:subjectOf1[hl7:controlActEvent/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.3']]";
		ensure_node_count(doc, parser, xpath, base, other_ids.len())?;
		for (idx, other) in other_ids.iter().enumerate() {
			let path = indexed_path(base, idx + 1, "/hl7:controlActEvent/hl7:id");
			set_attr_first(xpath, &path, "assigningAuthorityName", &other.source_of_identifier);
			set_attr_first(xpath, &path, "extension", &other.case_identifier);
		}
	}
	set_attr_first(
		xpath,
		"//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='2' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.23']]/hl7:value",
		"value",
		if other_ids.is_empty() { "false" } else { "true" },
	);
	remove_attr_first(
		xpath,
		"//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='2' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.23']]/hl7:value",
		"nullFlavor",
	);

	let linked_reports = fetch_linked_report_numbers(mm, case_id).await?;
	if !linked_reports.is_empty() {
		let base = "//hl7:outboundRelationship[@typeCode='SPRT'][hl7:relatedInvestigation/hl7:subjectOf2/hl7:controlActEvent/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.2']]";
		ensure_node_count(doc, parser, xpath, base, linked_reports.len())?;
		for (idx, linked) in linked_reports.iter().enumerate() {
			let path = indexed_path(
				base,
				idx + 1,
				"/hl7:relatedInvestigation/hl7:subjectOf2/hl7:controlActEvent/hl7:id",
			);
			set_attr_first(xpath, &path, "extension", &linked.linked_report_number);
		}
	}

	if let Some(reason) = report.nullification_reason.as_deref() {
		set_text_first(
			xpath,
			"//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='4' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.23']]/hl7:value/hl7:originalText",
			reason,
		);
	}
	if let Some(code) = report.nullification_code.as_deref() {
		set_attr_first(
			xpath,
			"//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='3' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.23']]/hl7:value",
			"code",
			code,
		);
	}

	if let Some(sender) = fetch_sender_information(mm, case_id).await? {
		apply_sender_information(xpath, &sender);
	}

	let primaries = fetch_primary_sources(mm, case_id).await?;
	if !primaries.is_empty() {
		let base = "//hl7:outboundRelationship[@typeCode='SPRT'][hl7:relatedInvestigation/hl7:code[@code='2' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.22']]";
		ensure_node_count(doc, parser, xpath, base, primaries.len())?;
		for (idx, primary) in primaries.iter().enumerate() {
			let rel_path = indexed_path(base, idx + 1, "");
			let assigned_base = format!(
				"{rel_path}/hl7:relatedInvestigation/hl7:subjectOf2/hl7:controlActEvent/hl7:author/hl7:assignedEntity"
			);
			apply_primary_source_at(xpath, &assigned_base, primary);
			if let Some(value) = primary.primary_source_regulatory.as_deref() {
				set_attr_first(
					xpath,
					&format!("{rel_path}/hl7:priorityNumber"),
					"value",
					value,
				);
			}
		}
	}

	let lit_refs = fetch_literature_references(mm, case_id).await?;
	if !lit_refs.is_empty() {
		let base = "//hl7:reference[@typeCode='REFR'][hl7:document/hl7:code[@code='2' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.27']]";
		ensure_node_count(doc, parser, xpath, base, lit_refs.len())?;
		for (idx, lit) in lit_refs.iter().enumerate() {
			let ref_path = indexed_path(base, idx + 1, "/hl7:document");
			set_text_first(
				xpath,
				&format!("{ref_path}/hl7:bibliographicDesignationText"),
				&lit.reference_text,
			);
			if let Some(text) = lit.document_base64.as_deref() {
				set_text_first(xpath, &format!("{ref_path}/hl7:text"), text);
			}
			if let Some(media) = lit.media_type.as_deref() {
				set_attr_first(xpath, &format!("{ref_path}/hl7:text"), "mediaType", media);
			}
			if let Some(rep) = lit.representation.as_deref() {
				set_attr_first(
					xpath,
					&format!("{ref_path}/hl7:text"),
					"representation",
					rep,
				);
			}
			if let Some(comp) = lit.compression.as_deref() {
				set_attr_first(
					xpath,
					&format!("{ref_path}/hl7:text"),
					"compression",
					comp,
				);
			}
		}
	}

	let docs = fetch_documents_held_by_sender(mm, case_id).await?;
	if !docs.is_empty() {
		let base = "//hl7:reference[@typeCode='REFR'][hl7:document/hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.27']]";
		ensure_node_count(doc, parser, xpath, base, docs.len())?;
		for (idx, doc_item) in docs.iter().enumerate() {
			let ref_path = indexed_path(base, idx + 1, "/hl7:document");
			if let Some(title) = doc_item.title.as_deref() {
				set_text_first(xpath, &format!("{ref_path}/hl7:title"), title);
			}
			if let Some(text) = doc_item.document_base64.as_deref() {
				set_text_first(xpath, &format!("{ref_path}/hl7:text"), text);
			}
			if let Some(media) = doc_item.media_type.as_deref() {
				set_attr_first(xpath, &format!("{ref_path}/hl7:text"), "mediaType", media);
			}
			if let Some(rep) = doc_item.representation.as_deref() {
				set_attr_first(
					xpath,
					&format!("{ref_path}/hl7:text"),
					"representation",
					rep,
				);
			}
			if let Some(comp) = doc_item.compression.as_deref() {
				set_attr_first(
					xpath,
					&format!("{ref_path}/hl7:text"),
					"compression",
					comp,
				);
			}
		}
	}
	let has_docs = !docs.is_empty();
	set_attr_first(
		xpath,
		"//hl7:component/hl7:observationEvent[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value",
		"value",
		if has_docs { "true" } else { "false" },
	);

	if let Some(study) = fetch_study_information(mm, case_id).await? {
		let base = "//hl7:primaryRole/hl7:subjectOf1/hl7:researchStudy";
		if let Some(name) = study.study_name.as_deref() {
			set_text_first(xpath, &format!("{base}/hl7:title"), name);
		}
		if let Some(num) = study.sponsor_study_number.as_deref() {
			set_attr_first(
				xpath,
				&format!("{base}/hl7:id"),
				"extension",
				num,
			);
		}
		if let Some(study_type) = study.study_type_reaction.as_deref() {
			set_attr_first(
				xpath,
				&format!("{base}/hl7:code"),
				"code",
				study_type,
			);
		}

		let regs = fetch_study_registration_numbers(mm, study.id).await?;
		if !regs.is_empty() {
			let reg_base = format!("{base}/hl7:authorization/hl7:studyRegistration");
			ensure_node_count(doc, parser, xpath, &reg_base, regs.len())?;
			for (idx, reg) in regs.iter().enumerate() {
				let reg_path = indexed_path(&reg_base, idx + 1, "");
				set_attr_first(
					xpath,
					&format!("{reg_path}/hl7:id"),
					"extension",
					&reg.registration_number,
				);
				if let Some(country) = reg.country_code.as_deref() {
					set_attr_first(
						xpath,
						&format!(
							"{reg_path}/hl7:author/hl7:territorialAuthority/hl7:governingPlace/hl7:code"
						),
						"code",
						country,
					);
				}
			}
		}
	}

	Ok(())
}

async fn apply_section_d(
	doc: &mut Document,
	parser: &Parser,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	xpath: &mut Context,
) -> Result<()> {
	let patient = fetch_patient_information(mm, case_id).await?;
	let Some(patient) = patient else {
		return Ok(());
	};

	if let Some(name) = patient
		.patient_initials
		.as_deref()
		.filter(|v| !v.is_empty())
		.map(|v| v.to_string())
		.or_else(|| {
			let given = patient.patient_given_name.as_deref().unwrap_or("");
			let family = patient.patient_family_name.as_deref().unwrap_or("");
			let combined = format!("{} {}", given, family).trim().to_string();
			if combined.is_empty() { None } else { Some(combined) }
		}) {
		set_text_first(
			xpath,
			"//hl7:primaryRole/hl7:player1/hl7:name",
			&name,
		);
	}

	if let Some(sex) = patient.sex.as_deref() {
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:player1/hl7:administrativeGenderCode",
			"code",
			sex,
		);
	}

	if let Some(birth_date) = patient.birth_date {
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:player1/hl7:birthTime",
			"value",
			&fmt_date(birth_date),
		);
	}
	let identifiers = fetch_patient_identifiers(mm, patient.id).await?;
	if !identifiers.is_empty() {
		let base = "//hl7:primaryRole/hl7:player1/hl7:asIdentifiedEntity";
		ensure_node_count(doc, parser, xpath, base, identifiers.len())?;
		for (idx, ident) in identifiers.iter().enumerate() {
			let path = indexed_path(base, idx + 1, "");
			if let Some(root) = patient_identifier_root(ident.identifier_type_code.as_str()) {
				set_attr_first(xpath, &format!("{path}/hl7:id"), "root", root);
			}
			set_attr_first(
				xpath,
				&format!("{path}/hl7:id"),
				"extension",
				&ident.identifier_value,
			);
			set_attr_first(
				xpath,
				&format!("{path}/hl7:code"),
				"code",
				&ident.identifier_type_code,
			);
		}
	}

	if let Some(age) = patient.age_at_time_of_onset {
		set_attr_first(
			xpath,
			"//hl7:subjectOf2/hl7:observation[hl7:code[@code='3' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value",
			"value",
			&age.to_string(),
		);
		if let Some(unit) = patient.age_unit.as_deref() {
			set_attr_first(
				xpath,
				"//hl7:subjectOf2/hl7:observation[hl7:code[@code='3' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value",
				"unit",
				unit,
			);
		}
	}
	if let Some(gestation) = patient.gestation_period {
		set_attr_first(
			xpath,
			"//hl7:subjectOf2/hl7:observation[hl7:code[@code='16' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value",
			"value",
			&gestation.to_string(),
		);
		if let Some(unit) = patient.gestation_period_unit.as_deref() {
			set_attr_first(
				xpath,
				"//hl7:subjectOf2/hl7:observation[hl7:code[@code='16' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value",
				"unit",
				unit,
			);
		}
	}
	if let Some(age_group) = patient.age_group.as_deref() {
		set_attr_first(
			xpath,
			"//hl7:subjectOf2/hl7:observation[hl7:code[@code='4' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value",
			"code",
			age_group,
		);
	}

	if let Some(weight) = patient.weight_kg {
		set_attr_first(
			xpath,
			"//hl7:subjectOf2/hl7:observation[hl7:code[@code='7' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value",
			"value",
			&weight.to_string(),
		);
	}

	if let Some(height) = patient.height_cm {
		set_attr_first(
			xpath,
			"//hl7:subjectOf2/hl7:observation[hl7:code[@code='17' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value",
			"value",
			&height.to_string(),
		);
	}

	if let Some(lmp) = patient.last_menstrual_period_date {
		set_attr_first(
			xpath,
			"//hl7:subjectOf2/hl7:observation[hl7:code[@code='22' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value",
			"value",
			&fmt_date(lmp),
		);
		remove_attr_first(
			xpath,
			"//hl7:subjectOf2/hl7:observation[hl7:code[@code='22' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value",
			"nullFlavor",
		);
	}

	let medical_history = fetch_medical_histories(mm, patient.id).await?;
	if !medical_history.is_empty() {
		let base = "//hl7:organizer[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.20']]/hl7:component/hl7:observation";
		ensure_node_count(doc, parser, xpath, base, medical_history.len())?;
		for (idx, med) in medical_history.iter().enumerate() {
			let path = indexed_path(base, idx + 1, "");
			if let Some(code) = med.meddra_code.as_deref() {
				set_attr_first(
					xpath,
					&format!("{path}/hl7:code"),
					"code",
					code,
				);
			}
			if let Some(version) = med.meddra_version.as_deref() {
				set_attr_first(
					xpath,
					&format!("{path}/hl7:code"),
					"codeSystemVersion",
					version,
				);
			}
			if let Some(start) = med.start_date {
				set_attr_first(
					xpath,
					&format!("{path}/hl7:effectiveTime/hl7:low"),
					"value",
					&fmt_date(start),
				);
			}
			if let Some(continuing) = med.continuing {
				set_attr_first(
					xpath,
					&format!(
						"{path}/hl7:inboundRelationship/hl7:observation/hl7:value"
					),
					"value",
					if continuing { "true" } else { "false" },
				);
			}
			if let Some(text) = med.comments.as_deref() {
				set_text_first(
					xpath,
					&format!(
						"{path}/hl7:outboundRelationship2/hl7:observation[hl7:code[@code='10']]/hl7:value"
					),
					text,
				);
			}
			if let Some(value) = med.family_history {
				set_attr_first(
					xpath,
					&format!(
						"{path}/hl7:outboundRelationship2/hl7:observation[hl7:code[@code='38']]/hl7:value"
					),
					"value",
					if value { "true" } else { "false" },
				);
			}
		}
	}
	if let Some(text) = patient.medical_history_text.as_deref() {
		set_text_first(
			xpath,
			"//hl7:organizer[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.20']]/hl7:component/hl7:observation[hl7:code[@code='18']]/hl7:value",
			text,
		);
	}
	if let Some(concomitant) = patient.concomitant_therapy {
		set_attr_first(
			xpath,
			"//hl7:organizer[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.20']]/hl7:component/hl7:observation[hl7:code[@code='11']]/hl7:value",
			"value",
			if concomitant { "true" } else { "false" },
		);
	}

	if let Some(parent) = fetch_parent_information(mm, patient.id).await? {
		if let Some(name) = parent.parent_identification.as_deref() {
			set_text_first(
				xpath,
				"//hl7:primaryRole/hl7:player1/hl7:role[@classCode='PRS']/hl7:associatedPerson/hl7:name",
				name,
			);
		}
		if let Some(sex) = parent.sex.as_deref() {
			set_attr_first(
				xpath,
				"//hl7:primaryRole/hl7:player1/hl7:role[@classCode='PRS']/hl7:associatedPerson/hl7:administrativeGenderCode",
				"code",
				sex,
			);
		}
		if let Some(birth_date) = parent.parent_birth_date {
			set_attr_first(
				xpath,
				"//hl7:primaryRole/hl7:player1/hl7:role[@classCode='PRS']/hl7:associatedPerson/hl7:birthTime",
				"value",
				&fmt_date(birth_date),
			);
		}
		if let Some(age) = parent.parent_age {
			set_attr_first(
				xpath,
				"//hl7:primaryRole/hl7:player1/hl7:role[@classCode='PRS']/hl7:subjectOf2/hl7:observation[hl7:code[@code='3']]/hl7:value",
				"value",
				&age.to_string(),
			);
		}
		if let Some(unit) = parent.parent_age_unit.as_deref() {
			set_attr_first(
				xpath,
				"//hl7:primaryRole/hl7:player1/hl7:role[@classCode='PRS']/hl7:subjectOf2/hl7:observation[hl7:code[@code='3']]/hl7:value",
				"unit",
				unit,
			);
		}
		if let Some(lmp) = parent.last_menstrual_period_date {
			set_attr_first(
				xpath,
				"//hl7:primaryRole/hl7:player1/hl7:role[@classCode='PRS']/hl7:subjectOf2/hl7:observation[hl7:code[@code='22']]/hl7:value",
				"value",
				&fmt_date(lmp),
			);
		}
		if let Some(weight) = parent.weight_kg {
			set_attr_first(
				xpath,
				"//hl7:primaryRole/hl7:player1/hl7:role[@classCode='PRS']/hl7:subjectOf2/hl7:observation[hl7:code[@code='7']]/hl7:value",
				"value",
				&weight.to_string(),
			);
			set_attr_first(
				xpath,
				"//hl7:primaryRole/hl7:player1/hl7:role[@classCode='PRS']/hl7:subjectOf2/hl7:observation[hl7:code[@code='7']]/hl7:value",
				"unit",
				"kg",
			);
		}
		if let Some(height) = parent.height_cm {
			set_attr_first(
				xpath,
				"//hl7:primaryRole/hl7:player1/hl7:role[@classCode='PRS']/hl7:subjectOf2/hl7:observation[hl7:code[@code='17']]/hl7:value",
				"value",
				&height.to_string(),
			);
			set_attr_first(
				xpath,
				"//hl7:primaryRole/hl7:player1/hl7:role[@classCode='PRS']/hl7:subjectOf2/hl7:observation[hl7:code[@code='17']]/hl7:value",
				"unit",
				"cm",
			);
		}
		if let Some(text) = parent.medical_history_text.as_deref() {
			set_text_first(
				xpath,
				"//hl7:primaryRole/hl7:player1/hl7:role[@classCode='PRS']/hl7:subjectOf2/hl7:organizer[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.20']]/hl7:component/hl7:observation[hl7:code[@code='18']]/hl7:value",
				text,
			);
		}

		let parent_med = fetch_parent_medical_histories(mm, parent.id).await?;
		if !parent_med.is_empty() {
			let base = "//hl7:primaryRole/hl7:player1/hl7:role[@classCode='PRS']/hl7:subjectOf2/hl7:organizer[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.20']]/hl7:component/hl7:observation";
			ensure_node_count(doc, parser, xpath, base, parent_med.len())?;
			for (idx, med) in parent_med.iter().enumerate() {
				let path = indexed_path(base, idx + 1, "");
				if let Some(code) = med.meddra_code.as_deref() {
					set_attr_first(
						xpath,
						&format!("{path}/hl7:code"),
						"code",
						code,
					);
				}
				if let Some(version) = med.meddra_version.as_deref() {
					set_attr_first(
						xpath,
						&format!("{path}/hl7:code"),
						"codeSystemVersion",
						version,
					);
				}
				if let Some(continuing) = med.continuing {
					set_attr_first(
						xpath,
						&format!(
							"{path}/hl7:inboundRelationship/hl7:observation/hl7:value"
						),
						"value",
						if continuing { "true" } else { "false" },
					);
				}
				if let Some(text) = med.comments.as_deref() {
					set_text_first(
						xpath,
						&format!(
							"{path}/hl7:outboundRelationship2/hl7:observation[hl7:code[@code='10']]/hl7:value"
						),
						text,
					);
				}
			}
		}

		let parent_past_drugs = fetch_parent_past_drug_histories(mm, parent.id).await?;
		if !parent_past_drugs.is_empty() {
			let base = "//hl7:primaryRole/hl7:player1/hl7:role[@classCode='PRS']/hl7:subjectOf2/hl7:organizer[hl7:code[@code='2' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.20']]/hl7:component/hl7:substanceAdministration";
			ensure_node_count(doc, parser, xpath, base, parent_past_drugs.len())?;
			for (idx, drug) in parent_past_drugs.iter().enumerate() {
				let path = indexed_path(base, idx + 1, "");
				if let Some(start) = drug.start_date {
					set_attr_first(
						xpath,
						&format!("{path}/hl7:effectiveTime/hl7:low"),
						"value",
						&fmt_date(start),
					);
				}
				if let Some(end) = drug.end_date {
					set_attr_first(
						xpath,
						&format!("{path}/hl7:effectiveTime/hl7:high"),
						"value",
						&fmt_date(end),
					);
				}
				if let Some(mpid) = drug.mpid.as_deref() {
					set_attr_first(
						xpath,
						&format!(
							"{path}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:code"
						),
						"code",
						mpid,
					);
					if let Some(version) = drug.mpid_version.as_deref() {
						set_attr_first(
							xpath,
							&format!(
								"{path}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:code"
							),
							"codeSystemVersion",
							version,
						);
					}
				} else if let Some(phpid) = drug.phpid.as_deref() {
					set_attr_first(
						xpath,
						&format!(
							"{path}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:code"
						),
						"code",
						phpid,
					);
					if let Some(version) = drug.phpid_version.as_deref() {
						set_attr_first(
							xpath,
							&format!(
								"{path}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:code"
							),
							"codeSystemVersion",
							version,
						);
					}
				}
				if let Some(name) = drug.drug_name.as_deref() {
					set_text_first(
						xpath,
						&format!(
							"{path}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:name"
						),
						name,
					);
				}
				if let Some(code) = drug.indication_meddra_code.as_deref() {
					set_attr_first(
						xpath,
						&format!(
							"{path}/hl7:outboundRelationship2[@typeCode='RSON']/hl7:observation/hl7:value"
						),
						"code",
						code,
					);
				}
				if let Some(version) = drug.indication_meddra_version.as_deref() {
					set_attr_first(
						xpath,
						&format!(
							"{path}/hl7:outboundRelationship2[@typeCode='RSON']/hl7:observation/hl7:value"
						),
						"codeSystemVersion",
						version,
					);
				}
				if let Some(code) = drug.reaction_meddra_code.as_deref() {
					set_attr_first(
						xpath,
						&format!(
							"{path}/hl7:outboundRelationship2[@typeCode='CAUS']/hl7:observation/hl7:value"
						),
						"code",
						code,
					);
				}
				if let Some(version) = drug.reaction_meddra_version.as_deref() {
					set_attr_first(
						xpath,
						&format!(
							"{path}/hl7:outboundRelationship2[@typeCode='CAUS']/hl7:observation/hl7:value"
						),
						"codeSystemVersion",
						version,
					);
				}
			}
		}
	}

	let past_drugs = fetch_past_drug_histories(mm, patient.id).await?;
	if !past_drugs.is_empty() {
		let base = "//hl7:organizer[hl7:code[@code='2' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.20']]/hl7:component/hl7:substanceAdministration";
		ensure_node_count(doc, parser, xpath, base, past_drugs.len())?;
		for (idx, drug) in past_drugs.iter().enumerate() {
			let path = indexed_path(base, idx + 1, "");
			if let Some(start) = drug.start_date {
				set_attr_first(
					xpath,
					&format!("{path}/hl7:effectiveTime/hl7:low"),
					"value",
					&fmt_date(start),
				);
			}
			if let Some(end) = drug.end_date {
				set_attr_first(
					xpath,
					&format!("{path}/hl7:effectiveTime/hl7:high"),
					"value",
					&fmt_date(end),
				);
			}
			if let Some(mpid) = drug.mpid.as_deref() {
				set_attr_first(
					xpath,
					&format!(
						"{path}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:code"
					),
					"code",
					mpid,
				);
				if let Some(version) = drug.mpid_version.as_deref() {
					set_attr_first(
						xpath,
						&format!(
							"{path}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:code"
						),
						"codeSystemVersion",
						version,
					);
				}
			} else if let Some(phpid) = drug.phpid.as_deref() {
				set_attr_first(
					xpath,
					&format!(
						"{path}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:code"
					),
					"code",
					phpid,
				);
				if let Some(version) = drug.phpid_version.as_deref() {
					set_attr_first(
						xpath,
						&format!(
							"{path}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:code"
						),
						"codeSystemVersion",
						version,
					);
				}
			}
			if let Some(name) = drug.drug_name.as_deref() {
				set_text_first(
					xpath,
					&format!(
						"{path}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:name"
					),
					name,
				);
			}
			if let Some(code) = drug.indication_meddra_code.as_deref() {
				set_attr_first(
					xpath,
					&format!(
						"{path}/hl7:outboundRelationship2[@typeCode='RSON']/hl7:observation/hl7:value"
					),
					"code",
					code,
				);
			}
			if let Some(version) = drug.indication_meddra_version.as_deref() {
				set_attr_first(
					xpath,
					&format!(
						"{path}/hl7:outboundRelationship2[@typeCode='RSON']/hl7:observation/hl7:value"
					),
					"codeSystemVersion",
					version,
				);
			}
			if let Some(code) = drug.reaction_meddra_code.as_deref() {
				set_attr_first(
					xpath,
					&format!(
						"{path}/hl7:outboundRelationship2[@typeCode='CAUS']/hl7:observation/hl7:value"
					),
					"code",
					code,
				);
			}
			if let Some(version) = drug.reaction_meddra_version.as_deref() {
				set_attr_first(
					xpath,
					&format!(
						"{path}/hl7:outboundRelationship2[@typeCode='CAUS']/hl7:observation/hl7:value"
					),
					"codeSystemVersion",
					version,
				);
			}
		}
	}

	if let Some(death) = fetch_patient_death_information(mm, patient.id).await? {
		if let Some(date) = death.date_of_death {
			set_attr_first(
				xpath,
				"//hl7:primaryRole/hl7:player1/hl7:deceasedTime",
				"value",
				&fmt_date(date),
			);
		}
		if let Some(done) = death.autopsy_performed {
			set_bool_flag(
				xpath,
				"//hl7:subjectOf2/hl7:observation[hl7:code[@code='5']]/hl7:value",
				done,
			);
		}

		let reported = fetch_reported_causes_of_death(mm, death.id).await?;
		if !reported.is_empty() {
			let base =
				"//hl7:subjectOf2/hl7:observation[hl7:code[@code='32'] and @classCode='OBS']";
			ensure_node_count(doc, parser, xpath, base, reported.len())?;
			for (idx, cause) in reported.iter().enumerate() {
				let path = indexed_path(base, idx + 1, "/hl7:value");
				if let Some(code) = cause.meddra_code.as_deref() {
					set_attr_first(xpath, &path, "code", code);
				}
				if let Some(version) = cause.meddra_version.as_deref() {
					set_attr_first(xpath, &path, "codeSystemVersion", version);
				}
			}
		}

		let autopsy = fetch_autopsy_causes_of_death(mm, death.id).await?;
		if !autopsy.is_empty() {
			let base = "//hl7:subjectOf2/hl7:observation[hl7:code[@code='5']]/hl7:outboundRelationship2/hl7:observation";
			ensure_node_count(doc, parser, xpath, base, autopsy.len())?;
			for (idx, cause) in autopsy.iter().enumerate() {
				let path = indexed_path(base, idx + 1, "/hl7:value");
				if let Some(code) = cause.meddra_code.as_deref() {
					set_attr_first(xpath, &path, "code", code);
				}
				if let Some(version) = cause.meddra_version.as_deref() {
					set_attr_first(xpath, &path, "codeSystemVersion", version);
				}
			}
		}
	}

	Ok(())
}

async fn apply_section_e(
	doc: &mut Document,
	parser: &Parser,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	xpath: &mut Context,
) -> Result<()> {
	let reactions = fetch_reactions(mm, case_id).await?;
	if reactions.is_empty() {
		return Ok(());
	}

	let base = "//hl7:subjectOf2/hl7:observation[hl7:code[@code='29' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]";
	ensure_node_count(doc, parser, xpath, base, reactions.len())?;

	for (idx, reaction) in reactions.iter().enumerate() {
		let path = indexed_path(base, idx + 1, "");

		if let Some(start) = reaction.start_date {
			set_attr_first(
				xpath,
				&format!("{path}/hl7:effectiveTime/hl7:comp[@xsi:type='IVL_TS']/hl7:low"),
				"value",
				&fmt_date(start),
			);
		}
		if let Some(end) = reaction.end_date {
			set_attr_first(
				xpath,
				&format!("{path}/hl7:effectiveTime/hl7:comp[@xsi:type='IVL_TS']/hl7:high"),
				"value",
				&fmt_date(end),
			);
		}
		if let Some(duration) = reaction.duration_value {
			set_attr_first(
				xpath,
				&format!("{path}/hl7:effectiveTime/hl7:comp[@operator='A']/hl7:width"),
				"value",
				&duration.to_string(),
			);
			if let Some(unit) = reaction.duration_unit.as_deref() {
				set_attr_first(
					xpath,
					&format!("{path}/hl7:effectiveTime/hl7:comp[@operator='A']/hl7:width"),
					"unit",
					unit,
				);
			}
		}

		if let Some(code) = reaction.reaction_meddra_code.as_deref() {
			set_attr_first(
				xpath,
				&format!("{path}/hl7:value[@xsi:type='CE']"),
				"code",
				code,
			);
		}
		if let Some(version) = reaction.reaction_meddra_version.as_deref() {
			set_attr_first(
				xpath,
				&format!("{path}/hl7:value[@xsi:type='CE']"),
				"codeSystemVersion",
				version,
			);
		}
		set_text_first(
			xpath,
			&format!("{path}/hl7:value[@xsi:type='CE']/hl7:originalText"),
			&reaction.primary_source_reaction,
		);
		if let Some(lang) = reaction.reaction_language.as_deref() {
			set_attr_first(
				xpath,
				&format!("{path}/hl7:value[@xsi:type='CE']/hl7:originalText"),
				"language",
				lang,
			);
		}

		set_text_first(
			xpath,
			&format!(
				"{path}/hl7:outboundRelationship2/hl7:observation[hl7:code[@code='30']]/hl7:value"
			),
			&reaction.primary_source_reaction,
		);

		if let Some(country) = reaction.country_code.as_deref() {
			set_attr_first(
				xpath,
				&format!("{path}/hl7:location/hl7:locatedEntity/hl7:locatedPlace/hl7:code"),
				"code",
				country,
			);
		}

		if let Some(term_highlighted) = reaction.term_highlighted {
			set_attr_first(
				xpath,
				&format!(
					"{path}/hl7:outboundRelationship2/hl7:observation[hl7:code[@code='37']]/hl7:value"
				),
				"code",
				if term_highlighted { "1" } else { "2" },
			);
		}

		set_bool_flag(
			xpath,
			&format!(
				"{path}/hl7:outboundRelationship2/hl7:observation[hl7:code[@code='34']]/hl7:value"
			),
			reaction.criteria_death,
		);
		set_bool_flag(
			xpath,
			&format!(
				"{path}/hl7:outboundRelationship2/hl7:observation[hl7:code[@code='21']]/hl7:value"
			),
			reaction.criteria_life_threatening,
		);
		set_bool_flag(
			xpath,
			&format!(
				"{path}/hl7:outboundRelationship2/hl7:observation[hl7:code[@code='33']]/hl7:value"
			),
			reaction.criteria_hospitalization,
		);
		set_bool_flag(
			xpath,
			&format!(
				"{path}/hl7:outboundRelationship2/hl7:observation[hl7:code[@code='35']]/hl7:value"
			),
			reaction.criteria_disabling,
		);
		set_bool_flag(
			xpath,
			&format!(
				"{path}/hl7:outboundRelationship2/hl7:observation[hl7:code[@code='12']]/hl7:value"
			),
			reaction.criteria_congenital_anomaly,
		);
		set_bool_flag(
			xpath,
			&format!(
				"{path}/hl7:outboundRelationship2/hl7:observation[hl7:code[@code='26']]/hl7:value"
			),
			reaction.criteria_other_medically_important,
		);

		if let Some(outcome) = reaction.outcome.as_deref() {
			set_attr_first(
				xpath,
				&format!(
					"{path}/hl7:outboundRelationship2/hl7:observation[hl7:code[@code='27']]/hl7:value"
				),
				"code",
				outcome,
			);
		}
	}

	Ok(())
}

async fn apply_section_f(
	doc: &mut Document,
	parser: &Parser,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	xpath: &mut Context,
) -> Result<()> {
	let tests = fetch_test_results(mm, case_id).await?;
	if tests.is_empty() {
		return Ok(());
	}

	let base = "//hl7:subjectOf2/hl7:organizer[hl7:code[@code='3' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.20']]/hl7:component/hl7:observation";
	ensure_node_count(doc, parser, xpath, base, tests.len())?;

	for (idx, test) in tests.iter().enumerate() {
		let path = indexed_path(base, idx + 1, "");

		if let Some(code) = test.test_meddra_code.as_deref() {
			set_attr_first(xpath, &format!("{path}/hl7:code"), "code", code);
		}
		if let Some(version) = test.test_meddra_version.as_deref() {
			set_attr_first(
				xpath,
				&format!("{path}/hl7:code"),
				"codeSystemVersion",
				version,
			);
		}

		if let Some(date) = test.test_date {
			set_attr_first(
				xpath,
				&format!("{path}/hl7:effectiveTime"),
				"value",
				&fmt_date(date),
			);
		}

		if let Some(value) = test.test_result_value.as_deref() {
			set_attr_first(
				xpath,
				&format!("{path}/hl7:value[@xsi:type='IVL_PQ']/hl7:center"),
				"value",
				value,
			);
		}
		if let Some(unit) = test.test_result_unit.as_deref() {
			set_attr_first(
				xpath,
				&format!("{path}/hl7:value[@xsi:type='IVL_PQ']/hl7:center"),
				"unit",
				unit,
			);
		}

		if let Some(high) = test.normal_high_value.as_deref() {
			set_attr_first(
				xpath,
				&format!(
					"{path}/hl7:referenceRange/hl7:observationRange/hl7:value"
				),
				"value",
				high,
			);
		}

		if let Some(more) = test.more_info_available {
			set_bool_flag(
				xpath,
				&format!(
					"{path}/hl7:outboundRelationship2/hl7:observation[hl7:code[@code='25']]/hl7:value"
				),
				more,
			);
		}

		if let Some(comments) = test.comments.as_deref() {
			set_text_first(
				xpath,
				&format!(
					"{path}/hl7:outboundRelationship2/hl7:observation/hl7:value"
				),
				comments,
			);
		}
	}

	Ok(())
}

async fn apply_section_g(
	doc: &mut Document,
	parser: &Parser,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	xpath: &mut Context,
) -> Result<()> {
	let drugs = fetch_drug_informations(mm, case_id).await?;
	if drugs.is_empty() {
		return Ok(());
	}

	let mut relatedness_items: Vec<(DrugReactionAssessment, RelatednessAssessment)> =
		Vec::new();

	let base_comp = "//hl7:subjectOf2/hl7:organizer[hl7:code[@code='4' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.20']]/hl7:component";
	ensure_node_count(doc, parser, xpath, base_comp, drugs.len())?;

	for (idx, drug) in drugs.iter().enumerate() {
		let comp_path = indexed_path(base_comp, idx + 1, "");
		let base = format!("{comp_path}/hl7:substanceAdministration");

		if let Some(text) = drug.dosage_text.as_deref() {
			set_text_first(xpath, &format!("{base}/hl7:text"), text);
		}

		set_text_first(
			xpath,
			&format!("{base}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:name"),
			&drug.medicinal_product,
		);

		if let Some(mpid) = drug.mpid.as_deref() {
			set_attr_first(
				xpath,
				&format!(
					"{base}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:code"
				),
				"code",
				mpid,
			);
		}
		if let Some(version) = drug.mpid_version.as_deref() {
			set_attr_first(
				xpath,
				&format!(
					"{base}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:code"
				),
				"codeSystemVersion",
				version,
			);
		}
		if drug.mpid.is_none() {
			if let Some(phpid) = drug.phpid.as_deref() {
				set_attr_first(
					xpath,
					&format!(
						"{base}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:code"
					),
					"code",
					phpid,
				);
			}
			if let Some(version) = drug.phpid_version.as_deref() {
				set_attr_first(
					xpath,
					&format!(
						"{base}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:code"
					),
					"codeSystemVersion",
					version,
				);
			}
		}
		if let Some(value) = drug.investigational_product_blinded {
			set_bool_flag(
				xpath,
				&format!(
					"{base}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:subjectOf/hl7:observation[hl7:code[@code='G.k.2.5']]/hl7:value"
				),
				value,
			);
		}

		if let Some(name) = drug.manufacturer_name.as_deref() {
			set_text_first(
				xpath,
				&format!(
					"{base}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:asManufacturedProduct/hl7:subjectOf/hl7:approval/hl7:holder/hl7:role/hl7:playingOrganization/hl7:name"
				),
				name,
			);
		}
		if let Some(country) = drug.manufacturer_country.as_deref() {
			set_attr_first(
				xpath,
				&format!(
					"{base}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:asManufacturedProduct/hl7:subjectOf/hl7:approval/hl7:author/hl7:territorialAuthority/hl7:territory/hl7:code"
				),
				"code",
				country,
			);
		}

		if let Some(country) = drug.obtain_drug_country.as_deref() {
			set_text_first(
				xpath,
				&format!(
					"{base}/hl7:consumable/hl7:instanceOfKind/hl7:subjectOf/hl7:productEvent/hl7:performer/hl7:assignedEntity/hl7:representedOrganization/hl7:addr/hl7:country"
				),
				country,
			);
		}

		if let Some(action) = drug.action_taken.as_deref() {
			set_attr_first(
				xpath,
				&format!(
					"{base}/hl7:inboundRelationship[@typeCode='CAUS']/hl7:act/hl7:code"
				),
				"code",
				action,
			);
		}

		if let Some(rechallenge) = drug.rechallenge.as_deref() {
			set_attr_first(
				xpath,
				&format!(
					"{base}/hl7:outboundRelationship2/hl7:observation[hl7:code[@code='31']]/hl7:value"
				),
				"code",
				rechallenge,
			);
		}

		let recurrences = fetch_drug_recurrences(mm, drug.id).await?;
		if !recurrences.is_empty() {
			let rec_base = format!(
				"{base}/hl7:outboundRelationship2[@typeCode='PERT']/hl7:observation[hl7:code[@code='31']]"
			);
			ensure_node_count(doc, parser, xpath, &rec_base, recurrences.len())?;
			for (ridx, rec) in recurrences.iter().enumerate() {
				let rec_path = indexed_path(&rec_base, ridx + 1, "");
				if let Some(code) = rec.reaction_recurred.as_deref() {
					set_attr_first(
						xpath,
						&format!("{rec_path}/hl7:value"),
						"code",
						code,
					);
				}
			}
		}

		let substances = fetch_drug_active_substances(mm, drug.id).await?;
		if !substances.is_empty() {
			let sub_base = format!(
				"{base}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:ingredient"
			);
			ensure_node_count(doc, parser, xpath, &sub_base, substances.len())?;
			for (sidx, substance) in substances.iter().enumerate() {
				let sub_path = indexed_path(&sub_base, sidx + 1, "");
				if let Some(name) = substance.substance_name.as_deref() {
					set_text_first(
						xpath,
						&format!("{sub_path}/hl7:ingredientSubstance/hl7:name"),
						name,
					);
				}
				if let Some(termid) = substance.substance_termid.as_deref() {
					set_attr_first(
						xpath,
						&format!("{sub_path}/hl7:ingredientSubstance/hl7:code"),
						"code",
						termid,
					);
				}
				if let Some(version) = substance.substance_termid_version.as_deref() {
					set_attr_first(
						xpath,
						&format!("{sub_path}/hl7:ingredientSubstance/hl7:code"),
						"codeSystemVersion",
						version,
					);
				}
				if let Some(value) = substance.strength_value {
					set_attr_first(
						xpath,
						&format!("{sub_path}/hl7:quantity/hl7:numerator"),
						"value",
						&value.to_string(),
					);
				}
				if let Some(unit) = substance.strength_unit.as_deref() {
					set_attr_first(
						xpath,
						&format!("{sub_path}/hl7:quantity/hl7:numerator"),
						"unit",
						unit,
					);
				}
			}
		}

		let dosages = fetch_dosage_information_list(mm, drug.id).await?;
		if !dosages.is_empty() {
			let dose_rel_base = format!("{base}/hl7:outboundRelationship2[@typeCode='COMP']");
			ensure_node_count(doc, parser, xpath, &dose_rel_base, dosages.len())?;
			for (didx, dosage) in dosages.iter().enumerate() {
				let dose_base = indexed_path(&dose_rel_base, didx + 1, "/hl7:substanceAdministration");
				if let Some(text) = dosage.dosage_text.as_deref().or(drug.dosage_text.as_deref()) {
					set_text_first(
						xpath,
						&format!("{dose_base}/hl7:text"),
						text,
					);
				}
				if let Some(value) = dosage.frequency_value {
					set_attr_first(
						xpath,
						&format!("{dose_base}/hl7:effectiveTime/hl7:comp[@xsi:type='PIVL_TS']/hl7:period"),
						"value",
						&value.to_string(),
					);
				}
				if let Some(unit) = dosage.frequency_unit.as_deref() {
					set_attr_first(
						xpath,
						&format!("{dose_base}/hl7:effectiveTime/hl7:comp[@xsi:type='PIVL_TS']/hl7:period"),
						"unit",
						unit,
					);
				}
				if let Some(start) = dosage.first_administration_date {
					set_attr_first(
						xpath,
						&format!("{dose_base}/hl7:effectiveTime/hl7:comp[@operator='A']/hl7:low"),
						"value",
						&fmt_date(start),
					);
				}
				if let Some(end) = dosage.last_administration_date {
					set_attr_first(
						xpath,
						&format!("{dose_base}/hl7:effectiveTime/hl7:comp[@operator='A']/hl7:high"),
						"value",
						&fmt_date(end),
					);
				}
				if let Some(value) = dosage.duration_value {
					set_attr_first(
						xpath,
						&format!("{dose_base}/hl7:effectiveTime/hl7:comp[@operator='A']/hl7:width"),
						"value",
						&value.to_string(),
					);
				}
				if let Some(unit) = dosage.duration_unit.as_deref() {
					set_attr_first(
						xpath,
						&format!("{dose_base}/hl7:effectiveTime/hl7:comp[@operator='A']/hl7:width"),
						"unit",
						unit,
					);
				}
				if let Some(dose) = dosage.dose_value {
					set_attr_first(
						xpath,
						&format!("{dose_base}/hl7:doseQuantity"),
						"value",
						&dose.to_string(),
					);
				}
				if let Some(unit) = dosage.dose_unit.as_deref() {
					set_attr_first(
						xpath,
						&format!("{dose_base}/hl7:doseQuantity"),
						"unit",
						unit,
					);
				}
				if let Some(route) = dosage.route_of_administration.as_deref() {
					set_attr_first(
						xpath,
						&format!("{dose_base}/hl7:routeCode"),
						"code",
						route,
					);
					set_text_first(
						xpath,
						&format!("{dose_base}/hl7:routeCode/hl7:originalText"),
						route,
					);
				}
				if dosage.route_of_administration.is_none() {
					if let Some(parent_route) =
						dosage.parent_route.as_deref().or(dosage.parent_route_termid.as_deref())
					{
						set_attr_first(
							xpath,
							&format!("{dose_base}/hl7:routeCode"),
							"code",
							parent_route,
						);
						set_text_first(
							xpath,
							&format!("{dose_base}/hl7:routeCode/hl7:originalText"),
							parent_route,
						);
					}
					if let Some(version) = dosage.parent_route_termid_version.as_deref() {
						set_attr_first(
							xpath,
							&format!("{dose_base}/hl7:routeCode"),
							"codeSystemVersion",
							version,
						);
					}
				}
				if let Some(form) = dosage.dose_form.as_deref() {
					set_text_first(
						xpath,
						&format!(
							"{dose_base}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:formCode/hl7:originalText"
						),
						form,
					);
				}
				if let Some(termid) = dosage.dose_form_termid.as_deref() {
					set_attr_first(
						xpath,
						&format!(
							"{dose_base}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:formCode"
						),
						"code",
						termid,
					);
				}
				if let Some(version) = dosage.dose_form_termid_version.as_deref() {
					set_attr_first(
						xpath,
						&format!(
							"{dose_base}/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:formCode"
						),
						"codeSystemVersion",
						version,
					);
				}
				if let Some(termid) = dosage.parent_route_termid.as_deref() {
					set_attr_first(
						xpath,
						&format!(
							"{dose_base}/hl7:outboundRelationship2/hl7:observation[hl7:code[@code='G.k.4.r.11']]/hl7:value"
						),
						"code",
						termid,
					);
				}
				if let Some(version) = dosage.parent_route_termid_version.as_deref() {
					set_attr_first(
						xpath,
						&format!(
							"{dose_base}/hl7:outboundRelationship2/hl7:observation[hl7:code[@code='G.k.4.r.11']]/hl7:value"
						),
						"codeSystemVersion",
						version,
					);
				}
				if let Some(text) = dosage.parent_route.as_deref() {
					set_text_first(
						xpath,
						&format!(
							"{dose_base}/hl7:outboundRelationship2/hl7:observation[hl7:code[@code='G.k.4.r.11']]/hl7:value/hl7:originalText"
						),
						text,
					);
				}
			}
		}

		if let Some(parent_route) = drug.parent_route.as_deref() {
			let coded_base = format!(
				"{base}/hl7:outboundRelationship2[@typeCode='REFR']/hl7:observation[hl7:code[@code='9']]/hl7:value"
			);
			set_attr_first(xpath, &coded_base, "code", parent_route);
		}
		if let Some(parent_text) = drug.parent_dosage_text.as_deref() {
			let text_base = format!(
				"{base}/hl7:outboundRelationship2[@typeCode='REFR']/hl7:observation[hl7:code[@code='2']]/hl7:value"
			);
			set_text_first(xpath, &text_base, parent_text);
		}

		let indications = fetch_drug_indications(mm, drug.id).await?;
		if !indications.is_empty() {
			let ind_rel_base = format!("{base}/hl7:inboundRelationship[@typeCode='RSON']");
			ensure_node_count(doc, parser, xpath, &ind_rel_base, indications.len())?;
			for (iidx, indication) in indications.iter().enumerate() {
				let ind_base = indexed_path(&ind_rel_base, iidx + 1, "/hl7:observation/hl7:value");
				if let Some(code) = indication.indication_meddra_code.as_deref() {
					set_attr_first(xpath, &ind_base, "code", code);
				}
				if let Some(version) = indication.indication_meddra_version.as_deref() {
					set_attr_first(xpath, &ind_base, "codeSystemVersion", version);
				}
				if let Some(text) = indication.indication_text.as_deref() {
					set_text_first(xpath, &format!("{ind_base}/hl7:originalText"), text);
				}
			}
		}

		if !drug.drug_characterization.is_empty() {
			set_attr_first(
				xpath,
				&format!("{comp_path}/hl7:causalityAssessment[hl7:code[@code='20']]/hl7:value"),
				"code",
				&drug.drug_characterization,
			);
		}

		let assessments = fetch_drug_reaction_assessments(mm, drug.id).await?;
		for assessment in assessments {
			let related = fetch_relatedness_assessments(mm, assessment.id).await?;
			for rel in related {
				relatedness_items.push((assessment.clone(), rel));
			}
		}
	}

	if !relatedness_items.is_empty() {
		let base = "//hl7:component[hl7:causalityAssessment/hl7:code[@code='39']]";
		ensure_node_count(doc, parser, xpath, base, relatedness_items.len())?;
		for (idx, (assess, rel)) in relatedness_items.iter().enumerate() {
			let comp_path = indexed_path(base, idx + 1, "/hl7:causalityAssessment");
			if let Some(result) = rel.result_of_assessment.as_deref() {
				set_text_first(xpath, &format!("{comp_path}/hl7:value"), result);
			}
			if let Some(method) = rel.method_of_assessment.as_deref() {
				set_text_first(
					xpath,
					&format!("{comp_path}/hl7:methodCode/hl7:originalText"),
					method,
				);
			}
			if let Some(source) = rel.source_of_assessment.as_deref() {
				set_text_first(
					xpath,
					&format!(
						"{comp_path}/hl7:author/hl7:assignedEntity/hl7:code/hl7:originalText"
					),
					source,
				);
			}
			set_attr_first(
				xpath,
				&format!(
					"{comp_path}/hl7:subject1/hl7:adverseEffectReference/hl7:id"
				),
				"root",
				&assess.reaction_id.to_string(),
			);
			set_attr_first(
				xpath,
				&format!(
					"{comp_path}/hl7:subject2/hl7:productUseReference/hl7:id"
				),
				"root",
				&assess.drug_id.to_string(),
			);
		}
	}

	Ok(())
}

async fn apply_section_h(
	doc: &mut Document,
	parser: &Parser,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	xpath: &mut Context,
) -> Result<()> {
	let narrative = fetch_narrative_information(mm, case_id).await?;
	let Some(narrative) = narrative else {
		return Ok(());
	};

	set_text_first(
		xpath,
		"//hl7:investigationEvent/hl7:text",
		&narrative.case_narrative,
	);

	if let Some(reporter) = narrative.reporter_comments.as_deref() {
		set_text_first(
			xpath,
			"//hl7:component1/hl7:observationEvent[hl7:author/hl7:assignedEntity/hl7:code[@code='3']]/hl7:value",
			reporter,
		);
	}

	if let Some(sender) = narrative.sender_comments.as_deref() {
		set_text_first(
			xpath,
			"//hl7:component1/hl7:observationEvent[hl7:author/hl7:assignedEntity/hl7:code[@code='1']]/hl7:value",
			sender,
		);
	}

	if let Some(summary) = fetch_case_summary(mm, narrative.id).await? {
		if let Some(text) = summary.summary_text.as_deref() {
			set_text_first(
				xpath,
				"//hl7:component/hl7:observationEvent[hl7:code[@code='36']]/hl7:value",
				text,
			);
		}
		if let Some(lang) = summary.language_code.as_deref() {
			set_attr_first(
				xpath,
				"//hl7:component/hl7:observationEvent[hl7:code[@code='36']]/hl7:value",
				"language",
				lang,
			);
		}
	}

	let diagnoses = fetch_sender_diagnoses(mm, narrative.id).await?;
	if !diagnoses.is_empty() {
		let base = "//hl7:component1/hl7:observationEvent[hl7:code[@code='15']]";
		ensure_node_count(doc, parser, xpath, base, diagnoses.len())?;
		for (idx, diagnosis) in diagnoses.iter().enumerate() {
			let path = indexed_path(base, idx + 1, "/hl7:value");
			if let Some(code) = diagnosis.diagnosis_meddra_code.as_deref() {
				set_attr_first(xpath, &path, "code", code);
			}
			if let Some(version) = diagnosis.diagnosis_meddra_version.as_deref() {
				set_attr_first(xpath, &path, "codeSystemVersion", version);
			}
		}
	}

	Ok(())
}

async fn apply_section_n(
	doc: &mut Document,
	parser: &Parser,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	xpath: &mut Context,
) -> Result<()> {
	let header = fetch_message_header(mm, case_id).await?;
	let Some(header) = header else {
		return Ok(());
	};

	if let Some(batch_number) = header.batch_number.as_deref() {
		set_attr_first(xpath, "/hl7:MCCI_IN200100UV01/hl7:id", "extension", batch_number);
	}
	if let Some(batch_tx) = header.batch_transmission_date {
		set_attr_first(
			xpath,
			"/hl7:MCCI_IN200100UV01/hl7:creationTime",
			"value",
			&fmt_datetime(batch_tx),
		);
	}
	if let Some(batch_sender) = header.batch_sender_identifier.as_deref() {
		tracing::debug!(
			batch_sender,
			"XML export: applying batch sender identifier"
		);
		set_attr_first(
			xpath,
			"/hl7:MCCI_IN200100UV01/hl7:sender/hl7:device/hl7:id",
			"extension",
			batch_sender,
		);
	}
	if let Some(batch_receiver) = header.batch_receiver_identifier.as_deref() {
		tracing::debug!(
			batch_receiver,
			"XML export: applying batch receiver identifier"
		);
		set_attr_first(
			xpath,
			"/hl7:MCCI_IN200100UV01/hl7:receiver/hl7:device/hl7:id",
			"extension",
			batch_receiver,
		);
	}

	set_attr_first(
		xpath,
		"/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV/hl7:id",
		"extension",
		&header.message_number,
	);
	set_attr_first(
		xpath,
		"/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV/hl7:creationTime",
		"value",
		&header.message_date,
	);
	set_attr_first(
		xpath,
		"/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV/hl7:sender/hl7:device/hl7:id",
		"extension",
		&header.message_sender_identifier,
	);
	set_attr_first(
		xpath,
		"/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV/hl7:receiver/hl7:device/hl7:id",
		"extension",
		&header.message_receiver_identifier,
	);

	Ok(())
}

fn apply_sender_information(xpath: &mut Context, sender: &SenderInformation) {
	let base = "//hl7:investigationEvent/hl7:subjectOf1/hl7:controlActEvent/hl7:author/hl7:assignedEntity";
	set_attr_first(
		xpath,
		&format!("{base}/hl7:code"),
		"code",
		&sender.sender_type,
	);
	if let Some(value) = sender.street_address.as_deref() {
		set_text_first(xpath, &format!("{base}/hl7:addr/hl7:streetAddressLine"), value);
	}
	if let Some(value) = sender.city.as_deref() {
		set_text_first(xpath, &format!("{base}/hl7:addr/hl7:city"), value);
	}
	if let Some(value) = sender.state.as_deref() {
		set_text_first(xpath, &format!("{base}/hl7:addr/hl7:state"), value);
	}
	if let Some(value) = sender.postcode.as_deref() {
		set_text_first(xpath, &format!("{base}/hl7:addr/hl7:postalCode"), value);
	}
	if let Some(value) = sender.person_given_name.as_deref() {
		set_text_first(
			xpath,
			&format!("{base}/hl7:assignedPerson/hl7:name/hl7:given"),
			value,
		);
	}
	if let Some(value) = sender.person_title.as_deref() {
		set_text_first(
			xpath,
			&format!("{base}/hl7:assignedPerson/hl7:name/hl7:prefix"),
			value,
		);
	}
	if let Some(value) = sender.person_middle_name.as_deref() {
		set_text_first(
			xpath,
			&format!("{base}/hl7:assignedPerson/hl7:name/hl7:given[2]"),
			value,
		);
	}
	if let Some(value) = sender.person_family_name.as_deref() {
		set_text_first(
			xpath,
			&format!("{base}/hl7:assignedPerson/hl7:name/hl7:family"),
			value,
		);
	}
	if let Some(value) = sender.country_code.as_deref() {
		set_attr_first(
			xpath,
			&format!(
				"{base}/hl7:assignedPerson/hl7:asLocatedEntity/hl7:location/hl7:code"
			),
			"code",
			value,
		);
	}
	if let Some(value) = sender.department.as_deref() {
		set_text_first(
			xpath,
			&format!(
				"{base}/hl7:representedOrganization/hl7:name"
			),
			value,
		);
	}
	set_text_first(
		xpath,
		&format!(
			"{base}/hl7:representedOrganization/hl7:assignedEntity/hl7:representedOrganization/hl7:name"
		),
		&sender.organization_name,
	);

	if let Some(value) = sender.telephone.as_deref() {
		set_telecom_value(xpath, base, 1, &format!("tel:{value}"));
	}
	if let Some(value) = sender.fax.as_deref() {
		set_telecom_value(xpath, base, 2, &format!("fax:{value}"));
	}
	if let Some(value) = sender.email.as_deref() {
		set_telecom_value(xpath, base, 3, &format!("mailto:{value}"));
	}
}

fn apply_primary_source_at(
	xpath: &mut Context,
	base: &str,
	primary: &PrimarySource,
) {
	if let Some(value) = primary.street.as_deref() {
		set_text_first(xpath, &format!("{base}/hl7:addr/hl7:streetAddressLine"), value);
	}
	if let Some(value) = primary.city.as_deref() {
		set_text_first(xpath, &format!("{base}/hl7:addr/hl7:city"), value);
	}
	if let Some(value) = primary.state.as_deref() {
		set_text_first(xpath, &format!("{base}/hl7:addr/hl7:state"), value);
	}
	if let Some(value) = primary.postcode.as_deref() {
		set_text_first(xpath, &format!("{base}/hl7:addr/hl7:postalCode"), value);
	}
	if let Some(value) = primary.reporter_given_name.as_deref() {
		set_text_first(
			xpath,
			&format!("{base}/hl7:assignedPerson/hl7:name/hl7:given"),
			value,
		);
	}
	if let Some(value) = primary.reporter_title.as_deref() {
		set_text_first(
			xpath,
			&format!("{base}/hl7:assignedPerson/hl7:name/hl7:prefix"),
			value,
		);
	}
	if let Some(value) = primary.reporter_middle_name.as_deref() {
		set_text_first(
			xpath,
			&format!("{base}/hl7:assignedPerson/hl7:name/hl7:given[2]"),
			value,
		);
	}
	if let Some(value) = primary.reporter_family_name.as_deref() {
		set_text_first(
			xpath,
			&format!("{base}/hl7:assignedPerson/hl7:name/hl7:family"),
			value,
		);
	}
	if let Some(value) = primary.telephone.as_deref() {
		set_telecom_value(xpath, base, 1, &format!("tel:{value}"));
	}
	if let Some(value) = primary.email.as_deref() {
		set_telecom_value(xpath, base, 2, &format!("mailto:{value}"));
	}
	if let Some(value) = primary.qualification.as_deref() {
		set_attr_first(
			xpath,
			&format!(
				"{base}/hl7:assignedPerson/hl7:asQualifiedEntity/hl7:code"
			),
			"code",
			value,
		);
	}
	if let Some(value) = primary.country_code.as_deref() {
		set_attr_first(
			xpath,
			&format!(
				"{base}/hl7:assignedPerson/hl7:asLocatedEntity/hl7:location/hl7:code"
			),
			"code",
			value,
		);
	}
	if let Some(value) = primary.department.as_deref() {
		set_text_first(
			xpath,
			&format!("{base}/hl7:representedOrganization/hl7:name"),
			value,
		);
	}
	if let Some(value) = primary.organization.as_deref() {
		set_text_first(
			xpath,
			&format!(
				"{base}/hl7:representedOrganization/hl7:assignedEntity/hl7:representedOrganization/hl7:name"
			),
			value,
		);
	}
}

async fn fetch_sender_information(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Option<SenderInformation>> {
	let sql = "SELECT * FROM sender_information WHERE case_id = $1 LIMIT 1";
	mm.dbx()
		.fetch_optional(sqlx::query_as::<_, SenderInformation>(sql).bind(case_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_primary_sources(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Vec<PrimarySource>> {
	let sql = "SELECT * FROM primary_sources WHERE case_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, PrimarySource>(sql).bind(case_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_literature_references(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Vec<LiteratureReference>> {
	let sql = "SELECT * FROM literature_references WHERE case_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, LiteratureReference>(sql).bind(case_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_documents_held_by_sender(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Vec<DocumentsHeldBySender>> {
	let sql =
		"SELECT * FROM documents_held_by_sender WHERE case_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, DocumentsHeldBySender>(sql).bind(case_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}
async fn fetch_study_information(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Option<StudyInformation>> {
	let sql = "SELECT * FROM study_information WHERE case_id = $1 LIMIT 1";
	mm.dbx()
		.fetch_optional(sqlx::query_as::<_, StudyInformation>(sql).bind(case_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_study_registration_numbers(
	mm: &ModelManager,
	study_id: sqlx::types::Uuid,
) -> Result<Vec<StudyRegistrationNumber>> {
	let sql = "SELECT * FROM study_registration_numbers WHERE study_information_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, StudyRegistrationNumber>(sql).bind(study_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_other_case_identifiers(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Vec<OtherCaseIdentifier>> {
	let sql =
		"SELECT * FROM other_case_identifiers WHERE case_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, OtherCaseIdentifier>(sql).bind(case_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_linked_report_numbers(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Vec<LinkedReportNumber>> {
	let sql =
		"SELECT * FROM linked_report_numbers WHERE case_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, LinkedReportNumber>(sql).bind(case_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_patient_information(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Option<PatientInformation>> {
	let sql = "SELECT * FROM patient_information WHERE case_id = $1 LIMIT 1";
	mm.dbx()
		.fetch_optional(sqlx::query_as::<_, PatientInformation>(sql).bind(case_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_patient_identifiers(
	mm: &ModelManager,
	patient_id: sqlx::types::Uuid,
) -> Result<Vec<PatientIdentifier>> {
	let sql =
		"SELECT * FROM patient_identifiers WHERE patient_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, PatientIdentifier>(sql).bind(patient_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_parent_information(
	mm: &ModelManager,
	patient_id: sqlx::types::Uuid,
) -> Result<Option<ParentInformation>> {
	let sql = "SELECT * FROM parent_information WHERE patient_id = $1 LIMIT 1";
	mm.dbx()
		.fetch_optional(sqlx::query_as::<_, ParentInformation>(sql).bind(patient_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_parent_medical_histories(
	mm: &ModelManager,
	parent_id: sqlx::types::Uuid,
) -> Result<Vec<ParentMedicalHistory>> {
	let sql = "SELECT * FROM parent_medical_history WHERE parent_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, ParentMedicalHistory>(sql).bind(parent_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_parent_past_drug_histories(
	mm: &ModelManager,
	parent_id: sqlx::types::Uuid,
) -> Result<Vec<ParentPastDrugHistory>> {
	let sql = "SELECT * FROM parent_past_drug_history WHERE parent_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, ParentPastDrugHistory>(sql).bind(parent_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_past_drug_histories(
	mm: &ModelManager,
	patient_id: sqlx::types::Uuid,
) -> Result<Vec<PastDrugHistory>> {
	let sql = "SELECT * FROM past_drug_history WHERE patient_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, PastDrugHistory>(sql).bind(patient_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_patient_death_information(
	mm: &ModelManager,
	patient_id: sqlx::types::Uuid,
) -> Result<Option<PatientDeathInformation>> {
	let sql = "SELECT * FROM patient_death_information WHERE patient_id = $1 LIMIT 1";
	mm.dbx()
		.fetch_optional(
			sqlx::query_as::<_, PatientDeathInformation>(sql).bind(patient_id),
		)
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_reported_causes_of_death(
	mm: &ModelManager,
	death_id: sqlx::types::Uuid,
) -> Result<Vec<ReportedCauseOfDeath>> {
	let sql = "SELECT * FROM reported_causes_of_death WHERE death_info_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, ReportedCauseOfDeath>(sql).bind(death_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_autopsy_causes_of_death(
	mm: &ModelManager,
	death_id: sqlx::types::Uuid,
) -> Result<Vec<AutopsyCauseOfDeath>> {
	let sql = "SELECT * FROM autopsy_causes_of_death WHERE death_info_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, AutopsyCauseOfDeath>(sql).bind(death_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_medical_histories(
	mm: &ModelManager,
	patient_id: sqlx::types::Uuid,
) -> Result<Vec<MedicalHistoryEpisode>> {
	let sql = "SELECT * FROM medical_history_episodes WHERE patient_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, MedicalHistoryEpisode>(sql).bind(patient_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_drug_recurrences(
	mm: &ModelManager,
	drug_id: sqlx::types::Uuid,
) -> Result<Vec<DrugRecurrenceInformation>> {
	let sql = "SELECT * FROM drug_recurrence_information WHERE drug_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, DrugRecurrenceInformation>(sql).bind(drug_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_drug_reaction_assessments(
	mm: &ModelManager,
	drug_id: sqlx::types::Uuid,
) -> Result<Vec<DrugReactionAssessment>> {
	let sql = "SELECT * FROM drug_reaction_assessments WHERE drug_id = $1 ORDER BY created_at ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, DrugReactionAssessment>(sql).bind(drug_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_relatedness_assessments(
	mm: &ModelManager,
	assessment_id: sqlx::types::Uuid,
) -> Result<Vec<RelatednessAssessment>> {
	let sql = "SELECT * FROM relatedness_assessments WHERE drug_reaction_assessment_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, RelatednessAssessment>(sql).bind(assessment_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_reactions(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Vec<Reaction>> {
	let sql = "SELECT * FROM reactions WHERE case_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, Reaction>(sql).bind(case_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_test_results(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Vec<TestResult>> {
	let sql = "SELECT * FROM test_results WHERE case_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, TestResult>(sql).bind(case_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_drug_informations(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Vec<DrugInformation>> {
	let sql = "SELECT * FROM drug_information WHERE case_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, DrugInformation>(sql).bind(case_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_drug_active_substances(
	mm: &ModelManager,
	drug_id: sqlx::types::Uuid,
) -> Result<Vec<DrugActiveSubstance>> {
	let sql = "SELECT * FROM drug_active_substances WHERE drug_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, DrugActiveSubstance>(sql).bind(drug_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_dosage_information_list(
	mm: &ModelManager,
	drug_id: sqlx::types::Uuid,
) -> Result<Vec<DosageInformation>> {
	let sql = "SELECT * FROM dosage_information WHERE drug_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, DosageInformation>(sql).bind(drug_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_drug_indications(
	mm: &ModelManager,
	drug_id: sqlx::types::Uuid,
) -> Result<Vec<DrugIndication>> {
	let sql = "SELECT * FROM drug_indications WHERE drug_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, DrugIndication>(sql).bind(drug_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_narrative_information(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Option<NarrativeInformation>> {
	let sql = "SELECT * FROM narrative_information WHERE case_id = $1 LIMIT 1";
	mm.dbx()
		.fetch_optional(sqlx::query_as::<_, NarrativeInformation>(sql).bind(case_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_case_summary(
	mm: &ModelManager,
	narrative_id: sqlx::types::Uuid,
) -> Result<Option<CaseSummaryInformation>> {
	let sql = "SELECT * FROM case_summary_information WHERE narrative_id = $1 ORDER BY sequence_number ASC LIMIT 1";
	mm.dbx()
		.fetch_optional(sqlx::query_as::<_, CaseSummaryInformation>(sql).bind(narrative_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_sender_diagnoses(
	mm: &ModelManager,
	narrative_id: sqlx::types::Uuid,
) -> Result<Vec<SenderDiagnosis>> {
	let sql = "SELECT * FROM sender_diagnoses WHERE narrative_id = $1 ORDER BY sequence_number ASC";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, SenderDiagnosis>(sql).bind(narrative_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

async fn fetch_message_header(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Option<MessageHeader>> {
	let sql = "SELECT * FROM message_headers WHERE case_id = $1 LIMIT 1";
	mm.dbx()
		.fetch_optional(sqlx::query_as::<_, MessageHeader>(sql).bind(case_id))
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))
}

fn set_attr_first(xpath: &mut Context, path: &str, attr: &str, value: &str) {
	if let Ok(nodes) = xpath.findnodes(path, None) {
		if let Some(mut node) = nodes.into_iter().next() {
			let _ = node.set_attribute(attr, value);
		}
	}
}

fn set_attr_all(xpath: &mut Context, path: &str, attr: &str, value: &str) {
	if let Ok(nodes) = xpath.findnodes(path, None) {
		for mut node in nodes {
			let _ = node.set_attribute(attr, value);
		}
	}
}

fn patient_identifier_root(code: &str) -> Option<&'static str> {
	match code {
		"1" => Some("2.16.840.1.113883.3.989.2.1.3.7"),
		"2" => Some("2.16.840.1.113883.3.989.2.1.3.8"),
		"3" => Some("2.16.840.1.113883.3.989.2.1.3.9"),
		"4" => Some("2.16.840.1.113883.3.989.2.1.3.10"),
		_ => None,
	}
}

fn remove_attr_first(xpath: &mut Context, path: &str, attr: &str) {
	if let Ok(nodes) = xpath.findnodes(path, None) {
		if let Some(mut node) = nodes.into_iter().next() {
			let _ = node.remove_attribute(attr);
		}
	}
}

fn set_text_first(xpath: &mut Context, path: &str, value: &str) {
	if let Ok(nodes) = xpath.findnodes(path, None) {
		if let Some(mut node) = nodes.into_iter().next() {
			let _ = node.set_content(value);
		}
	}
}

fn set_telecom_value(xpath: &mut Context, base: &str, index: usize, value: &str) {
	let path = format!("{base}/hl7:telecom[{index}]");
	set_attr_first(xpath, &path, "value", value);
}

fn set_bool_flag(xpath: &mut Context, path: &str, value: bool) {
	set_attr_first(xpath, path, "value", if value { "true" } else { "false" });
	remove_attr_first(xpath, path, "nullFlavor");
}

fn indexed_path(base: &str, idx: usize, suffix: &str) -> String {
	if suffix.is_empty() {
		format!("({base})[{idx}]")
	} else {
		format!("({base})[{idx}]{suffix}")
	}
}

fn ensure_node_count(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	path: &str,
	desired: usize,
) -> Result<()> {
	let nodes = xpath
		.findnodes(path, None)
		.map_err(|_| Error::InvalidXml {
			message: format!("Failed to find nodes for path {path}"),
			line: None,
			column: None,
		})?;

	if nodes.is_empty() || nodes.len() >= desired {
		return Ok(());
	}

	let template = nodes[0].clone();
	let parent = template.get_parent().ok_or(Error::InvalidXml {
		message: "Template node has no parent".to_string(),
		line: None,
		column: None,
	})?;

	for _ in nodes.len()..desired {
		let mut clone = clone_node(doc, parser, &template)?;
		let mut parent_mut = parent.clone();
		parent_mut
			.add_child(&mut clone)
			.map_err(|err| Error::InvalidXml {
				message: format!("Failed to clone node: {err}"),
				line: None,
				column: None,
			})?;
	}

	Ok(())
}

fn clone_node(doc: &mut Document, parser: &Parser, node: &Node) -> Result<Node> {
	node_from_fragment(doc, parser, &doc.node_to_string(node))
}

fn node_from_fragment(doc: &mut Document, parser: &Parser, fragment: &str) -> Result<Node> {
	let fragment = wrap_fragment(fragment, "urn:hl7-org:v3");
	node_from_wrapped_fragment(doc, parser, &fragment)
}

fn wrap_fragment(fragment: &str, ns: &str) -> String {
	format!(
		"<wrapper xmlns=\"{ns}\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">{fragment}</wrapper>"
	)
}

fn node_from_wrapped_fragment(
	doc: &mut Document,
	parser: &Parser,
	fragment: &str,
) -> Result<Node> {
	let frag_doc = parser.parse_string(&fragment).map_err(|err| Error::InvalidXml {
		message: format!("XML parse error: {err}"),
		line: None,
		column: None,
	})?;
	let root = frag_doc.get_root_element().ok_or(Error::InvalidXml {
		message: "Failed to get fragment root".to_string(),
		line: None,
		column: None,
	})?;
	let mut child = root
		.get_child_nodes()
		.into_iter()
		.find(|n| n.get_type() == Some(NodeType::ElementNode))
		.ok_or(Error::InvalidXml {
			message: "Failed to get fragment child".to_string(),
			line: None,
			column: None,
		})?;
	child.unlink_node();
	doc.import_node(&mut child)
		.map_err(|_| Error::InvalidXml {
			message: "Failed to import cloned node".to_string(),
			line: None,
			column: None,
		})
}

fn fmt_date(date: Date) -> String {
	let year = date.year();
	let month: u8 = date.month().into();
	let day = date.day();
	format!("{:04}{:02}{:02}", year, month, day)
}

fn fmt_datetime(time: sqlx::types::time::OffsetDateTime) -> String {
	let year = time.year();
	let month: u8 = time.month().into();
	let day = time.day();
	let hour = time.hour();
	let minute = time.minute();
	let second = time.second();
	format!("{:04}{:02}{:02}{:02}{:02}{:02}", year, month, day, hour, minute, second)
}
