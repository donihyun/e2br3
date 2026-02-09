use crate::ctx::Ctx;
use crate::model;
use crate::model::case::CaseBmc;
use crate::model::drug::{
	DosageInformation, DrugActiveSubstance, DrugDeviceCharacteristic,
	DrugIndication, DrugInformation,
};
use crate::model::message_header::MessageHeader;
use crate::model::narrative::NarrativeInformationBmc;
use crate::model::patient::PatientInformationBmc;
use crate::model::reaction::Reaction;
use crate::model::safety_report::SafetyReportIdentificationBmc;
use crate::model::test_result::TestResult;
use crate::model::ModelManager;
use crate::xml::error::Error;
use crate::xml::export_postprocess::postprocess_export_doc;
use crate::xml::export_sections::c_safety_report::export_c_safety_report_patch;
use crate::xml::export_sections::c_safety_report::export_c_safety_report_xml;
use crate::xml::export_sections::d_patient::export_d_patient_patch;
use crate::xml::export_sections::d_patient::export_d_patient_xml;
use crate::xml::export_sections::e_reaction::export_e_reactions_xml;
use crate::xml::export_sections::f_test_result::export_f_test_results_xml;
use crate::xml::export_sections::g_drug::export_g_drugs_xml;
use crate::xml::export_sections::h_narrative::export_h_narrative_xml;
use crate::xml::raw::patch::{
	patch_e_reactions, patch_f_test_results, patch_g_drugs, patch_h_narrative,
};
use crate::xml::Result;
use libxml::parser::Parser;
use libxml::tree::Document;
use libxml::xpath::Context;

pub async fn export_case_xml(
	ctx: &Ctx,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<String> {
	let case = CaseBmc::get(ctx, mm, case_id).await.map_err(Error::from)?;
	let has_dirty = case.dirty_c
		|| case.dirty_d
		|| case.dirty_e
		|| case.dirty_f
		|| case.dirty_g
		|| case.dirty_h;
	if case.status != "validated" {
		if let Some(raw_xml) = case.raw_xml.as_deref() {
			if !has_dirty {
				return Ok(String::from_utf8_lossy(raw_xml).to_string());
			}
		}
		return Err(Error::InvalidXml {
			message: "Only validated cases can be exported".to_string(),
			line: None,
			column: None,
		});
	}

	if let Some(raw_xml) = case.raw_xml.as_deref() {
		let only_c_dirty = case.dirty_c
			&& !case.dirty_d
			&& !case.dirty_e
			&& !case.dirty_f
			&& !case.dirty_g
			&& !case.dirty_h;
		if only_c_dirty && std::env::var("XML_V2_PATCH_C").unwrap_or_default() == "1"
		{
			let report =
				SafetyReportIdentificationBmc::get_by_case(ctx, mm, case_id)
					.await
					.map_err(Error::from)?;
			return export_c_safety_report_patch(raw_xml, &case, &report);
		}

		let only_d_dirty = case.dirty_d
			&& !case.dirty_c
			&& !case.dirty_e
			&& !case.dirty_f
			&& !case.dirty_g
			&& !case.dirty_h;
		if only_d_dirty && std::env::var("XML_V2_PATCH_D").unwrap_or_default() == "1"
		{
			let patient = PatientInformationBmc::get_by_case(ctx, mm, case_id)
				.await
				.map_err(Error::from)?;
			return export_d_patient_patch(raw_xml, &patient);
		}

		let only_e_dirty = case.dirty_e
			&& !case.dirty_c
			&& !case.dirty_d
			&& !case.dirty_f
			&& !case.dirty_g
			&& !case.dirty_h;
		if only_e_dirty && std::env::var("XML_V2_PATCH_E").unwrap_or_default() == "1"
		{
			let sql = "SELECT * FROM reactions WHERE case_id = $1 ORDER BY sequence_number";
			let reactions = mm
				.dbx()
				.fetch_all(sqlx::query_as::<_, Reaction>(sql).bind(case_id))
				.await
				.map_err(model::Error::from)
				.map_err(Error::from)?;
			return patch_e_reactions(raw_xml, &reactions);
		}

		let only_f_dirty = case.dirty_f
			&& !case.dirty_c
			&& !case.dirty_d
			&& !case.dirty_e
			&& !case.dirty_g
			&& !case.dirty_h;
		if only_f_dirty && std::env::var("XML_V2_PATCH_F").unwrap_or_default() == "1"
		{
			let sql = "SELECT * FROM test_results WHERE case_id = $1 ORDER BY sequence_number";
			let tests = mm
				.dbx()
				.fetch_all(sqlx::query_as::<_, TestResult>(sql).bind(case_id))
				.await
				.map_err(model::Error::from)
				.map_err(Error::from)?;
			return patch_f_test_results(raw_xml, &tests);
		}

		let only_g_dirty = case.dirty_g
			&& !case.dirty_c
			&& !case.dirty_d
			&& !case.dirty_e
			&& !case.dirty_f
			&& !case.dirty_h;
		if only_g_dirty && std::env::var("XML_V2_PATCH_G").unwrap_or_default() == "1"
		{
			let drugs = mm
				.dbx()
				.fetch_all(
					sqlx::query_as::<_, DrugInformation>(
						"SELECT * FROM drug_information WHERE case_id = $1 ORDER BY sequence_number",
					)
					.bind(case_id),
				)
				.await
				.map_err(model::Error::from)
				.map_err(Error::from)?;
			let drug_ids: Vec<_> = drugs.iter().map(|d| d.id).collect();
			let substances = if drug_ids.is_empty() {
				Vec::new()
			} else {
				mm.dbx()
					.fetch_all(
						sqlx::query_as::<_, DrugActiveSubstance>(
							"SELECT * FROM drug_active_substances WHERE drug_id = ANY($1) ORDER BY sequence_number",
						)
						.bind(&drug_ids),
					)
					.await
					.map_err(model::Error::from)
					.map_err(Error::from)?
			};
			let dosages = if drug_ids.is_empty() {
				Vec::new()
			} else {
				mm.dbx()
					.fetch_all(
						sqlx::query_as::<_, DosageInformation>(
							"SELECT * FROM dosage_information WHERE drug_id = ANY($1) ORDER BY sequence_number",
						)
						.bind(&drug_ids),
					)
					.await
					.map_err(model::Error::from)
					.map_err(Error::from)?
			};
			let indications = if drug_ids.is_empty() {
				Vec::new()
			} else {
				mm.dbx()
					.fetch_all(
						sqlx::query_as::<_, DrugIndication>(
							"SELECT * FROM drug_indications WHERE drug_id = ANY($1) ORDER BY sequence_number",
						)
						.bind(&drug_ids),
					)
					.await
					.map_err(model::Error::from)
					.map_err(Error::from)?
			};
			let characteristics = if drug_ids.is_empty() {
				Vec::new()
			} else {
				mm.dbx()
					.fetch_all(
						sqlx::query_as::<_, DrugDeviceCharacteristic>(
							"SELECT * FROM drug_device_characteristics WHERE drug_id = ANY($1) ORDER BY sequence_number",
						)
						.bind(&drug_ids),
					)
					.await
					.map_err(model::Error::from)
					.map_err(Error::from)?
			};
			return patch_g_drugs(
				raw_xml,
				&drugs,
				&substances,
				&dosages,
				&indications,
				&characteristics,
			);
		}

		let only_h_dirty = case.dirty_h
			&& !case.dirty_c
			&& !case.dirty_d
			&& !case.dirty_e
			&& !case.dirty_f
			&& !case.dirty_g;
		if only_h_dirty && std::env::var("XML_V2_PATCH_H").unwrap_or_default() == "1"
		{
			let narrative = NarrativeInformationBmc::get_by_case(ctx, mm, case_id)
				.await
				.map_err(Error::from)?;
			return patch_h_narrative(raw_xml, &narrative);
		}
	}

	if case.raw_xml.is_none() {
		let only_c_dirty = case.dirty_c
			&& !case.dirty_d
			&& !case.dirty_e
			&& !case.dirty_f
			&& !case.dirty_g
			&& !case.dirty_h;
		if only_c_dirty
			&& std::env::var("XML_V2_EXPORT_C").unwrap_or_default() == "1"
		{
			let report =
				SafetyReportIdentificationBmc::get_by_case(ctx, mm, case_id)
					.await
					.map_err(Error::from)?;
			return export_c_safety_report_xml(&case, &report);
		}

		let only_d_dirty = case.dirty_d
			&& !case.dirty_c
			&& !case.dirty_e
			&& !case.dirty_f
			&& !case.dirty_g
			&& !case.dirty_h;
		if only_d_dirty
			&& std::env::var("XML_V2_EXPORT_D").unwrap_or_default() == "1"
		{
			let patient = PatientInformationBmc::get_by_case(ctx, mm, case_id)
				.await
				.map_err(Error::from)?;
			return export_d_patient_xml(&patient);
		}

		let only_e_dirty = case.dirty_e
			&& !case.dirty_c
			&& !case.dirty_d
			&& !case.dirty_f
			&& !case.dirty_g
			&& !case.dirty_h;
		if only_e_dirty
			&& std::env::var("XML_V2_EXPORT_E").unwrap_or_default() == "1"
		{
			let sql = "SELECT * FROM reactions WHERE case_id = $1 ORDER BY sequence_number";
			let reactions = mm
				.dbx()
				.fetch_all(sqlx::query_as::<_, Reaction>(sql).bind(case_id))
				.await
				.map_err(model::Error::from)
				.map_err(Error::from)?;
			return export_e_reactions_xml(&reactions);
		}

		let only_f_dirty = case.dirty_f
			&& !case.dirty_c
			&& !case.dirty_d
			&& !case.dirty_e
			&& !case.dirty_g
			&& !case.dirty_h;
		if only_f_dirty
			&& std::env::var("XML_V2_EXPORT_F").unwrap_or_default() == "1"
		{
			let sql = "SELECT * FROM test_results WHERE case_id = $1 ORDER BY sequence_number";
			let tests = mm
				.dbx()
				.fetch_all(sqlx::query_as::<_, TestResult>(sql).bind(case_id))
				.await
				.map_err(model::Error::from)
				.map_err(Error::from)?;
			return export_f_test_results_xml(&tests);
		}

		let only_g_dirty = case.dirty_g
			&& !case.dirty_c
			&& !case.dirty_d
			&& !case.dirty_e
			&& !case.dirty_f
			&& !case.dirty_h;
		if only_g_dirty
			&& std::env::var("XML_V2_EXPORT_G").unwrap_or_default() == "1"
		{
			let drugs = mm
				.dbx()
				.fetch_all(
					sqlx::query_as::<_, DrugInformation>(
						"SELECT * FROM drug_information WHERE case_id = $1 ORDER BY sequence_number",
					)
					.bind(case_id),
				)
				.await
				.map_err(model::Error::from)
				.map_err(Error::from)?;
			let drug_ids: Vec<_> = drugs.iter().map(|d| d.id).collect();
			let substances = if drug_ids.is_empty() {
				Vec::new()
			} else {
				mm.dbx()
					.fetch_all(
						sqlx::query_as::<_, DrugActiveSubstance>(
							"SELECT * FROM drug_active_substances WHERE drug_id = ANY($1) ORDER BY sequence_number",
						)
						.bind(&drug_ids),
					)
					.await
					.map_err(model::Error::from)
					.map_err(Error::from)?
			};
			let dosages = if drug_ids.is_empty() {
				Vec::new()
			} else {
				mm.dbx()
					.fetch_all(
						sqlx::query_as::<_, DosageInformation>(
							"SELECT * FROM dosage_information WHERE drug_id = ANY($1) ORDER BY sequence_number",
						)
						.bind(&drug_ids),
					)
					.await
					.map_err(model::Error::from)
					.map_err(Error::from)?
			};
			let indications = if drug_ids.is_empty() {
				Vec::new()
			} else {
				mm.dbx()
					.fetch_all(
						sqlx::query_as::<_, DrugIndication>(
							"SELECT * FROM drug_indications WHERE drug_id = ANY($1) ORDER BY sequence_number",
						)
						.bind(&drug_ids),
					)
					.await
					.map_err(model::Error::from)
					.map_err(Error::from)?
			};
			let characteristics = if drug_ids.is_empty() {
				Vec::new()
			} else {
				mm.dbx()
					.fetch_all(
						sqlx::query_as::<_, DrugDeviceCharacteristic>(
							"SELECT * FROM drug_device_characteristics WHERE drug_id = ANY($1) ORDER BY sequence_number",
						)
						.bind(&drug_ids),
					)
					.await
					.map_err(model::Error::from)
					.map_err(Error::from)?
			};
			return export_g_drugs_xml(
				&drugs,
				&substances,
				&dosages,
				&indications,
				&characteristics,
			);
		}

		let only_h_dirty = case.dirty_h
			&& !case.dirty_c
			&& !case.dirty_d
			&& !case.dirty_e
			&& !case.dirty_f
			&& !case.dirty_g;
		if only_h_dirty
			&& std::env::var("XML_V2_EXPORT_H").unwrap_or_default() == "1"
		{
			let narrative = NarrativeInformationBmc::get_by_case(ctx, mm, case_id)
				.await
				.map_err(Error::from)?;
			return export_h_narrative_xml(&narrative);
		}
	}

	export_case_xml_from_db(ctx, mm, case_id).await
}

async fn export_case_xml_from_db(
	ctx: &Ctx,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<String> {
	let case = CaseBmc::get(ctx, mm, case_id).await.map_err(Error::from)?;
	let has_dirty = case.dirty_c
		|| case.dirty_d
		|| case.dirty_e
		|| case.dirty_f
		|| case.dirty_g
		|| case.dirty_h;
	if let Some(raw_xml) = case.raw_xml.as_deref() {
		if !has_dirty {
			return Ok(String::from_utf8_lossy(raw_xml).to_string());
		}
	}
	let mut xml = if let Some(raw_xml) = case.raw_xml.as_deref() {
		String::from_utf8_lossy(raw_xml).to_string()
	} else {
		base_export_skeleton().to_string()
	};

	if case.dirty_c {
		let report = SafetyReportIdentificationBmc::get_by_case(ctx, mm, case_id)
			.await
			.map_err(Error::from)?;
		xml = export_c_safety_report_patch(xml.as_bytes(), &case, &report)?;
	}
	if case.dirty_d {
		let patient = PatientInformationBmc::get_by_case(ctx, mm, case_id)
			.await
			.map_err(Error::from)?;
		xml = export_d_patient_patch(xml.as_bytes(), &patient)?;
	}
	if case.dirty_e {
		let sql =
			"SELECT * FROM reactions WHERE case_id = $1 ORDER BY sequence_number";
		let reactions = mm
			.dbx()
			.fetch_all(sqlx::query_as::<_, Reaction>(sql).bind(case_id))
			.await
			.map_err(model::Error::from)
			.map_err(Error::from)?;
		xml = patch_e_reactions(xml.as_bytes(), &reactions)?;
	}
	if case.dirty_f {
		let sql =
			"SELECT * FROM test_results WHERE case_id = $1 ORDER BY sequence_number";
		let tests = mm
			.dbx()
			.fetch_all(sqlx::query_as::<_, TestResult>(sql).bind(case_id))
			.await
			.map_err(model::Error::from)
			.map_err(Error::from)?;
		xml = patch_f_test_results(xml.as_bytes(), &tests)?;
	}
	if case.dirty_g {
		let drugs = mm
			.dbx()
			.fetch_all(
				sqlx::query_as::<_, DrugInformation>(
					"SELECT * FROM drug_information WHERE case_id = $1 ORDER BY sequence_number",
				)
				.bind(case_id),
			)
			.await
			.map_err(model::Error::from)
			.map_err(Error::from)?;
		let drug_ids: Vec<_> = drugs.iter().map(|d| d.id).collect();
		let substances = if drug_ids.is_empty() {
			Vec::new()
		} else {
			mm.dbx()
				.fetch_all(
					sqlx::query_as::<_, DrugActiveSubstance>(
						"SELECT * FROM drug_active_substances WHERE drug_id = ANY($1) ORDER BY sequence_number",
					)
					.bind(&drug_ids),
				)
				.await
				.map_err(model::Error::from)
				.map_err(Error::from)?
		};
		let dosages = if drug_ids.is_empty() {
			Vec::new()
		} else {
			mm.dbx()
				.fetch_all(
					sqlx::query_as::<_, DosageInformation>(
						"SELECT * FROM dosage_information WHERE drug_id = ANY($1) ORDER BY sequence_number",
					)
					.bind(&drug_ids),
				)
				.await
				.map_err(model::Error::from)
				.map_err(Error::from)?
		};
		let indications = if drug_ids.is_empty() {
			Vec::new()
		} else {
			mm.dbx()
				.fetch_all(
					sqlx::query_as::<_, DrugIndication>(
						"SELECT * FROM drug_indications WHERE drug_id = ANY($1) ORDER BY sequence_number",
					)
					.bind(&drug_ids),
				)
				.await
				.map_err(model::Error::from)
				.map_err(Error::from)?
		};
		let characteristics = if drug_ids.is_empty() {
			Vec::new()
		} else {
			mm.dbx()
				.fetch_all(
					sqlx::query_as::<_, DrugDeviceCharacteristic>(
						"SELECT * FROM drug_device_characteristics WHERE drug_id = ANY($1) ORDER BY sequence_number",
					)
					.bind(&drug_ids),
				)
				.await
				.map_err(model::Error::from)
				.map_err(Error::from)?
		};
		xml = patch_g_drugs(
			xml.as_bytes(),
			&drugs,
			&substances,
			&dosages,
			&indications,
			&characteristics,
		)?;
	}
	if case.dirty_h {
		let narrative = NarrativeInformationBmc::get_by_case(ctx, mm, case_id)
			.await
			.map_err(Error::from)?;
		xml = patch_h_narrative(xml.as_bytes(), &narrative)?;
	}

	let parser = Parser::default();
	let mut doc = parser.parse_string(&xml).map_err(|err| Error::InvalidXml {
		message: format!("XML parse error (patched): {err}"),
		line: None,
		column: None,
	})?;
	let mut xpath = Context::new(&doc).map_err(|_| Error::InvalidXml {
		message: "Failed to initialize XPath context".to_string(),
		line: None,
		column: None,
	})?;
	let _ = xpath.register_namespace("hl7", "urn:hl7-org:v3");
	let _ =
		xpath.register_namespace("xsi", "http://www.w3.org/2001/XMLSchema-instance");
	apply_section_n(&mut doc, &parser, mm, case_id, &mut xpath).await?;
	postprocess_export_doc(&mut doc, &mut xpath);

	Ok(doc.to_string())
}

fn base_export_skeleton() -> &'static str {
	"<?xml version=\"1.0\" encoding=\"utf-8\"?>\
<MCCI_IN200100UV01 xmlns=\"urn:hl7-org:v3\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" ITSVersion=\"XML_1.0\">\
	<id/>\
	<creationTime/>\
	<receiver typeCode=\"RCV\">\
		<device classCode=\"DEV\" determinerCode=\"INSTANCE\">\
			<id/>\
		</device>\
	</receiver>\
	<sender typeCode=\"SND\">\
		<device classCode=\"DEV\" determinerCode=\"INSTANCE\">\
			<id/>\
		</device>\
	</sender>\
	<PORR_IN049016UV>\
		<id/>\
		<creationTime/>\
		<receiver typeCode=\"RCV\">\
			<device classCode=\"DEV\" determinerCode=\"INSTANCE\">\
				<id/>\
			</device>\
		</receiver>\
		<sender typeCode=\"SND\">\
			<device classCode=\"DEV\" determinerCode=\"INSTANCE\">\
				<id/>\
			</device>\
		</sender>\
		<controlActProcess classCode=\"CACT\" moodCode=\"EVN\">\
			<code code=\"PORR_TE049016UV\" codeSystem=\"2.16.840.1.113883.1.18\"/>\
			<subject>\
				<investigationEvent classCode=\"INVSTG\" moodCode=\"EVN\">\
					<component typeCode=\"COMP\">\
						<adverseEventAssessment classCode=\"INVSTG\" moodCode=\"EVN\"/>\
					</component>\
				</investigationEvent>\
			</subject>\
		</controlActProcess>\
	</PORR_IN049016UV>\
</MCCI_IN200100UV01>"
}

async fn apply_section_n(
	_doc: &mut Document,
	_parser: &Parser,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	xpath: &mut Context,
) -> Result<()> {
	let header = fetch_message_header(mm, case_id).await?;
	let Some(header) = header else {
		return Ok(());
	};

	if let Some(batch_number) = header.batch_number.as_deref() {
		set_attr_first(
			xpath,
			"/hl7:MCCI_IN200100UV01/hl7:id",
			"extension",
			batch_number,
		);
	}
	if let Some(batch_tx) = header.batch_transmission_date {
		set_attr_first(
			xpath,
			"/hl7:MCCI_IN200100UV01/hl7:creationTime",
			"value",
			&fmt_datetime(batch_tx),
		);
	}
	let batch_sender = header
		.batch_sender_identifier
		.as_deref()
		.filter(|val| !val.trim().is_empty())
		.unwrap_or(&header.message_sender_identifier);
	tracing::debug!(batch_sender, "XML export: applying batch sender identifier");
	set_attr_first(
		xpath,
		"/hl7:MCCI_IN200100UV01/hl7:sender/hl7:device/hl7:id",
		"extension",
		batch_sender,
	);

	let batch_receiver = header
		.batch_receiver_identifier
		.as_deref()
		.filter(|val| !val.trim().is_empty())
		.unwrap_or(&header.message_receiver_identifier);
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

fn fmt_datetime(dt: sqlx::types::time::OffsetDateTime) -> String {
	format!(
		"{:04}{:02}{:02}{:02}{:02}{:02}",
		dt.year(),
		u8::from(dt.month()),
		dt.day(),
		dt.hour(),
		dt.minute(),
		dt.second()
	)
}
