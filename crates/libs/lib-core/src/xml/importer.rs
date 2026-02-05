use crate::ctx::Ctx;
use crate::model::audit::{CaseVersionBmc, CaseVersionForCreate};
use crate::model::case::{CaseBmc, CaseForCreate, CaseForUpdate};
use crate::model::drug::{
	DrugActiveSubstanceBmc, DrugActiveSubstanceForCreate, DrugDeviceCharacteristicBmc,
	DrugDeviceCharacteristicForCreate, DrugIndicationBmc, DrugIndicationForCreate,
	DrugInformationBmc, DrugInformationForCreate, DrugInformationForUpdate, DosageInformationBmc,
	DosageInformationForCreate,
};
use crate::model::message_header::{
	MessageHeaderBmc, MessageHeaderForCreate, MessageHeaderForUpdate,
};
use crate::model::reaction::{ReactionBmc, ReactionForCreate, ReactionForUpdate};
use crate::model::ModelManager;
use crate::xml::error::Error;
use crate::xml::types::XmlImportResult;
use crate::xml::validator::validate_e2b_xml;
use crate::xml::{parse_e2b_xml, Result};
use libxml::tree::Node;
use libxml::parser::Parser;
use libxml::xpath::Context;
use serde_json::json;
use rust_decimal::Decimal;
use sqlx::types::time::Date;
use time::Month;
use sqlx::types::Uuid;

#[derive(Debug)]
struct DrugSubstanceImport {
	substance_name: Option<String>,
	substance_termid: Option<String>,
	substance_termid_version: Option<String>,
	strength_value: Option<Decimal>,
	strength_unit: Option<String>,
}

#[derive(Debug)]
struct DrugDosageImport {
	dosage_text: Option<String>,
	frequency_value: Option<Decimal>,
	frequency_unit: Option<String>,
	start_date: Option<Date>,
	end_date: Option<Date>,
	duration_value: Option<Decimal>,
	duration_unit: Option<String>,
	dose_value: Option<Decimal>,
	dose_unit: Option<String>,
	route: Option<String>,
	dose_form: Option<String>,
	dose_form_termid: Option<String>,
	dose_form_termid_version: Option<String>,
	batch_lot: Option<String>,
	parent_route_termid: Option<String>,
	parent_route_termid_version: Option<String>,
	parent_route: Option<String>,
}

#[derive(Debug)]
struct DrugIndicationImport {
	text: Option<String>,
	version: Option<String>,
	code: Option<String>,
}

#[derive(Debug)]
struct DrugDeviceCharacteristicImport {
	code: Option<String>,
	code_system: Option<String>,
	code_display_name: Option<String>,
	value_type: Option<String>,
	value_value: Option<String>,
	value_code: Option<String>,
	value_code_system: Option<String>,
	value_display_name: Option<String>,
}

#[derive(Debug)]
struct DrugImport {
	sequence_number: i32,
	medicinal_product: String,
	brand_name: Option<String>,
	drug_characterization: String,
	mpid: Option<String>,
	mpid_version: Option<String>,
	investigational_product_blinded: Option<bool>,
	obtain_drug_country: Option<String>,
	manufacturer_name: Option<String>,
	manufacturer_country: Option<String>,
	batch_lot_number: Option<String>,
	dosage_text: Option<String>,
	action_taken: Option<String>,
	rechallenge: Option<String>,
	parent_route: Option<String>,
	parent_route_termid: Option<String>,
	parent_route_termid_version: Option<String>,
	parent_dosage_text: Option<String>,
	fda_additional_info_coded: Option<String>,
	substances: Vec<DrugSubstanceImport>,
	dosages: Vec<DrugDosageImport>,
	indications: Vec<DrugIndicationImport>,
	characteristics: Vec<DrugDeviceCharacteristicImport>,
}

#[derive(Debug, Clone)]
pub struct XmlImportRequest {
	pub xml: Vec<u8>,
	pub filename: Option<String>,	
}

pub async fn import_e2b_xml(
	ctx: &Ctx,
	mm: &ModelManager,
	req: XmlImportRequest,
) -> Result<XmlImportResult> {
	let mm = mm.new_with_txn()?;
	let report = validate_e2b_xml(&req.xml, None)?;
	if !report.ok {
		return Err(Error::XsdValidationFailed {
			errors: report.errors,
		});
	}

	let parsed = parse_e2b_xml(&req.xml)?;
	let safety_report_id_raw = extract_safety_report_id(&req.xml)?;
	let safety_report_id = clamp_str(
		Some(safety_report_id_raw),
		100,
		"cases.safety_report_id",
	)
	.unwrap_or_else(|| "UNKNOWN".to_string());
	let header_extract = extract_message_header(&req.xml).ok();

	let case_id = CaseBmc::create(
		ctx,
		&mm,
		CaseForCreate {
			organization_id: ctx.organization_id(),
			safety_report_id: safety_report_id.clone(),
			status: Some("draft".to_string()),
		},
	)
	.await?;

	if let Some(header) = header_extract {
		let message_number = header
			.message_number
			.unwrap_or_else(|| safety_report_id.clone());
		let message_sender = header
			.message_sender
			.clone()
			.or_else(|| header.batch_sender.clone());
		let message_receiver = header
			.message_receiver
			.clone()
			.or_else(|| header.batch_receiver.clone());
		let message_date = header
			.message_date
			.clone()
			.or_else(|| header.batch_transmission.clone());
		let (msg_sender, msg_receiver, msg_date) = (
			message_sender.clone(),
			message_receiver.clone(),
			message_date.clone(),
		);
		if let (Some(message_sender), Some(message_receiver), Some(message_date)) =
			(msg_sender, msg_receiver, msg_date)
		{
			let has_header = MessageHeaderBmc::get_by_case(ctx, &mm, case_id).await.is_ok();
			if !has_header {
				let _ = MessageHeaderBmc::create(
					ctx,
					&mm,
					MessageHeaderForCreate {
						case_id,
						message_number,
						message_sender_identifier: message_sender,
						message_receiver_identifier: message_receiver,
						message_date,
					},
				)
				.await;
			}
			let _ = MessageHeaderBmc::update_by_case(
				ctx,
				&mm,
				case_id,
				MessageHeaderForUpdate {
					batch_number: header.batch_number,
					batch_sender_identifier: header.batch_sender.clone(),
					batch_receiver_identifier: header.batch_receiver.clone(),
					batch_transmission_date: None,
					message_number: None,
					message_sender_identifier: None,
					message_receiver_identifier: None,
				},
			)
			.await;
		} else {
			tracing::warn!(
				message_sender = ?message_sender,
				message_receiver = ?message_receiver,
				message_date = ?message_date,
				"message header incomplete; skipping create"
			);
		}
	}

	CaseBmc::update(
		ctx,
		&mm,
		case_id,
		CaseForUpdate {
			raw_xml: Some(req.xml.to_vec()),
			safety_report_id: None,
			status: None,
			submitted_by: None,
			submitted_at: None,
			dirty_c: Some(false),
			dirty_d: Some(false),
			dirty_e: Some(false),
			dirty_f: Some(false),
			dirty_g: Some(false),
			dirty_h: Some(false),
		},
	)
	.await?;

	let snapshot = json!({
		"parsed": parsed.json,
		"raw_xml": String::from_utf8_lossy(&req.xml),
	});

	import_reactions(ctx, &mm, &req.xml, case_id).await?;
	import_drugs(ctx, &mm, &req.xml, case_id).await?;

	let version_id = match CaseVersionBmc::create(
		ctx,
		&mm,
		CaseVersionForCreate {
			case_id,
			version: 1,
			snapshot,
			change_reason: Some("XML import".to_string()),
		},
	)
		.await
	{
		Ok(id) => id,
		Err(err) => return Err(err.into()),
	};

	Ok(XmlImportResult {
		case_id: Some(case_id.to_string()),
		case_version: Some(1),
		xml_key: None,
		parsed_json_id: Some(version_id.to_string()),
	})
}

async fn import_reactions(
	ctx: &Ctx,
	mm: &ModelManager,
	xml: &[u8],
	case_id: Uuid,
) -> Result<()> {
	let imports = parse_reactions(xml, case_id)?;

	for (reaction_c, reaction_u) in imports {
		let rec_id = ReactionBmc::create(ctx, mm, reaction_c).await?;
		ReactionBmc::update(ctx, mm, rec_id, reaction_u).await?;
	}

	Ok(())
}

fn parse_reactions(
	xml: &[u8],
	case_id: Uuid,
) -> Result<Vec<(ReactionForCreate, ReactionForUpdate)>> {
	let xml_str = std::str::from_utf8(xml).map_err(|err| Error::InvalidXml {
		message: format!("XML not valid UTF-8: {err}"),
		line: None,
		column: None,
	})?;
	let parser = Parser::default();
	let doc = parser.parse_string(xml_str).map_err(|err| Error::InvalidXml {
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

	let nodes = xpath
		.findnodes(
			"//hl7:subjectOf2/hl7:observation[hl7:code[@code='29' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]",
			None,
		)
		.map_err(|_| Error::InvalidXml {
			message: "Failed to query reactions".to_string(),
			line: None,
			column: None,
		})?;

	let mut imports: Vec<(ReactionForCreate, ReactionForUpdate)> = Vec::new();
	for (idx, node) in nodes.into_iter().enumerate() {
		let primary = first_text(
			&mut xpath,
			&node,
			"hl7:value[@xsi:type='CE']/hl7:originalText",
		)
		.or_else(|| {
			first_text(
				&mut xpath,
				&node,
				"hl7:outboundRelationship2/hl7:observation[hl7:code[@code='30']]/hl7:value",
			)
		})
		.unwrap_or_else(|| "UNKNOWN".to_string());

		let reaction_c = ReactionForCreate {
			case_id,
			sequence_number: (idx + 1) as i32,
			primary_source_reaction: primary.clone(),
		};

		let reaction_meddra_version = clamp_str(
			first_attr(&mut xpath, &node, "hl7:value[@xsi:type='CE']", "codeSystemVersion"),
			10,
			"reactions.reaction_meddra_version",
		);
		let reaction_u = ReactionForUpdate {
			primary_source_reaction: Some(primary),
			reaction_language: normalize_lang2(
				first_attr(
					&mut xpath,
					&node,
					"hl7:value[@xsi:type='CE']/hl7:originalText",
					"language",
				),
				"reactions.reaction_language",
			),
			reaction_meddra_code: first_attr(
				&mut xpath,
				&node,
				"hl7:value[@xsi:type='CE']",
				"code",
			),
			reaction_meddra_version,
			term_highlighted: first_attr(
				&mut xpath,
				&node,
				"hl7:outboundRelationship2/hl7:observation[hl7:code[@code='37']]/hl7:value",
				"code",
			)
			.and_then(|v| match v.as_str() {
				"1" => Some(true),
				"2" => Some(false),
				_ => None,
			}),
			serious: None,
			criteria_death: parse_bool_attr(
				&mut xpath,
				&node,
				"hl7:outboundRelationship2/hl7:observation[hl7:code[@code='34']]/hl7:value",
				"value",
			),
			criteria_life_threatening: parse_bool_attr(
				&mut xpath,
				&node,
				"hl7:outboundRelationship2/hl7:observation[hl7:code[@code='21']]/hl7:value",
				"value",
			),
			criteria_hospitalization: parse_bool_attr(
				&mut xpath,
				&node,
				"hl7:outboundRelationship2/hl7:observation[hl7:code[@code='33']]/hl7:value",
				"value",
			),
			criteria_disabling: parse_bool_attr(
				&mut xpath,
				&node,
				"hl7:outboundRelationship2/hl7:observation[hl7:code[@code='35']]/hl7:value",
				"value",
			),
			criteria_congenital_anomaly: parse_bool_attr(
				&mut xpath,
				&node,
				"hl7:outboundRelationship2/hl7:observation[hl7:code[@code='12']]/hl7:value",
				"value",
			),
			criteria_other_medically_important: parse_bool_attr(
				&mut xpath,
				&node,
				"hl7:outboundRelationship2/hl7:observation[hl7:code[@code='26']]/hl7:value",
				"value",
			),
			required_intervention: clamp_str(
				first_attr(
					&mut xpath,
					&node,
					"hl7:outboundRelationship2/hl7:observation[hl7:code[@code='726']]/hl7:value",
					"value",
				),
				10,
				"reactions.required_intervention",
			),
			start_date: first_attr(
				&mut xpath,
				&node,
				"hl7:effectiveTime/hl7:comp[@xsi:type='IVL_TS']/hl7:low",
				"value",
			)
			.and_then(parse_date),
			end_date: first_attr(
				&mut xpath,
				&node,
				"hl7:effectiveTime/hl7:comp[@xsi:type='IVL_TS']/hl7:high",
				"value",
			)
			.and_then(parse_date),
			duration_value: first_attr(
				&mut xpath,
				&node,
				"hl7:effectiveTime/hl7:comp[@operator='A']/hl7:width",
				"value",
			)
			.and_then(|v| v.parse::<Decimal>().ok()),
			duration_unit: normalize_code3(
				first_attr(
					&mut xpath,
					&node,
					"hl7:effectiveTime/hl7:comp[@operator='A']/hl7:width",
					"unit",
				),
				"reactions.duration_unit",
			),
			outcome: first_attr(
				&mut xpath,
				&node,
				"hl7:outboundRelationship2/hl7:observation[hl7:code[@code='27']]/hl7:value",
				"code",
			),
			medical_confirmation: None,
			country_code: normalize_iso2(
				first_attr(
					&mut xpath,
					&node,
					"hl7:location/hl7:locatedEntity/hl7:locatedPlace/hl7:code",
					"code",
				),
				"reactions.country_code",
			),
		};

		imports.push((reaction_c, reaction_u));
	}

	Ok(imports)
}

fn first_attr(
	xpath: &mut Context,
	node: &Node,
	expr: &str,
	attr: &str,
) -> Option<String> {
	let expr = format!("{expr}/@{attr}");
	xpath.findvalues(&expr, Some(node)).ok()?.into_iter().find(|v| !v.trim().is_empty())
}

fn first_text(xpath: &mut Context, node: &Node, expr: &str) -> Option<String> {
	let nodes = xpath.findnodes(expr, Some(node)).ok()?;
	for n in nodes {
		let content = n.get_content();
		if !content.trim().is_empty() {
			return Some(content);
		}
	}
	None
}

fn parse_bool_attr(
	xpath: &mut Context,
	node: &Node,
	expr: &str,
	attr: &str,
) -> Option<bool> {
	let val = first_attr(xpath, node, expr, attr)?;
	match val.to_ascii_lowercase().as_str() {
		"true" | "1" => Some(true),
		"false" | "0" => Some(false),
		_ => None,
	}
}

fn clamp_str(value: Option<String>, max: usize, field: &str) -> Option<String> {
	match value {
		Some(v) if v.len() > max => {
			eprintln!(
				"[import_e2b_xml] truncating {field} len={} -> {max}",
				v.len()
			);
			Some(v.chars().take(max).collect())
		}
		other => other,
	}
}

fn normalize_code(
	value: Option<String>,
	allowed: &[&str],
	field: &str,
) -> Option<String> {
	match value {
		Some(v) => {
			let trimmed = v.trim();
			if allowed.contains(&trimmed) {
				return Some(trimmed.to_string());
			}
			let digit = trimmed.chars().next().filter(|c| c.is_ascii_digit());
			if let Some(d) = digit {
				let s = d.to_string();
				if allowed.contains(&s.as_str()) {
					eprintln!(
						"[import_e2b_xml] coercing {field} value={trimmed} -> {s}"
					);
					return Some(s);
				}
			}
			eprintln!("[import_e2b_xml] dropping invalid {field} value={trimmed}");
			None
		}
		None => None,
	}
}

fn normalize_iso2(value: Option<String>, field: &str) -> Option<String> {
	let v = value?.trim().to_string();
	let len = v.len();
	let upper = v.to_ascii_uppercase();
	if len == 2 && upper.chars().all(|c| c.is_ascii_uppercase()) {
		Some(upper)
	} else {
		tracing::warn!(field, value = %v, len, "dropping invalid ISO-3166-1 alpha-2");
		None
	}
}

fn normalize_lang2(value: Option<String>, field: &str) -> Option<String> {
	let v = value?.trim().to_string();
	let len = v.len();
	let lower = v.to_ascii_lowercase();
	if len == 2 && lower.chars().all(|c| c.is_ascii_lowercase()) {
		Some(lower)
	} else {
		tracing::warn!(field, value = %v, len, "dropping invalid ISO-639-1");
		None
	}
}

fn normalize_code3(value: Option<String>, field: &str) -> Option<String> {
	let v = value?.trim().to_string();
	let len = v.len();
	if (1..=3).contains(&len) && v.chars().all(|c| c.is_ascii_alphanumeric()) {
		Some(v)
	} else {
		tracing::warn!(field, value = %v, len, "dropping invalid 3-char code");
		None
	}
}

fn parse_date(value: String) -> Option<Date> {
	let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
	if digits.len() < 8 {
		return None;
	}
	let y: i32 = digits[0..4].parse().ok()?;
	let m: u8 = digits[4..6].parse().ok()?;
	let d: u8 = digits[6..8].parse().ok()?;
	let month = Month::try_from(m).ok()?;
	Date::from_calendar_date(y, month, d).ok()
}

#[derive(Debug)]
struct MessageHeaderExtract {
	message_number: Option<String>,
	message_sender: Option<String>,
	message_receiver: Option<String>,
	message_date: Option<String>,
	batch_number: Option<String>,
	batch_sender: Option<String>,
	batch_receiver: Option<String>,
	batch_transmission: Option<String>,
}

fn extract_message_header(xml: &[u8]) -> Result<MessageHeaderExtract> {
	let xml_str = std::str::from_utf8(xml).map_err(|err| Error::InvalidXml {
		message: format!("XML not valid UTF-8: {err}"),
		line: None,
		column: None,
	})?;
	let parser = Parser::default();
	let doc = parser.parse_string(xml_str).map_err(|err| Error::InvalidXml {
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

	let mut first_value = |expr: &str| -> Option<String> {
		xpath.findvalues(expr, None).ok()?.into_iter().find(|v| !v.trim().is_empty())
	};

	Ok(MessageHeaderExtract {
		message_number: first_value("//hl7:PORR_IN049016UV/hl7:id/@extension"),
		message_sender: first_value(
			"//hl7:PORR_IN049016UV/hl7:sender/hl7:device/hl7:id/@extension",
		),
		message_receiver: first_value(
			"//hl7:PORR_IN049016UV/hl7:receiver/hl7:device/hl7:id/@extension",
		),
		message_date: first_value("//hl7:PORR_IN049016UV/hl7:creationTime/@value"),
		batch_number: first_value("/hl7:MCCI_IN200100UV01/hl7:id/@extension"),
		batch_sender: first_value(
			"/hl7:MCCI_IN200100UV01/hl7:sender/hl7:device/hl7:id/@extension",
		),
		batch_receiver: first_value(
			"/hl7:MCCI_IN200100UV01/hl7:receiver/hl7:device/hl7:id/@extension",
		),
		batch_transmission: first_value("/hl7:MCCI_IN200100UV01/hl7:creationTime/@value"),
	})
}

fn extract_safety_report_id(xml: &[u8]) -> Result<String> {
	let xml_str = std::str::from_utf8(xml).map_err(|err| Error::InvalidXml {
		message: format!("XML not valid UTF-8: {err}"),
		line: None,
		column: None,
	})?;
	let parser = Parser::default();
	let doc = parser.parse_string(xml_str).map_err(|err| Error::InvalidXml {
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

	let candidates = xpath
		.findvalues(
			"//hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.1']/@extension",
			None,
		)
		.map_err(|_| Error::InvalidXml {
			message: "Failed to query safety_report_id".to_string(),
			line: None,
			column: None,
		})?;
	for value in candidates {
		if !value.trim().is_empty() {
			return Ok(value);
		}
	}

	Err(Error::InvalidXml {
		message: "safety_report_id not found".to_string(),
		line: None,
		column: None,
	})
}

async fn import_drugs(
	ctx: &Ctx,
	mm: &ModelManager,
	xml: &[u8],
	case_id: Uuid,
) -> Result<()> {
	let imports = parse_drugs(xml)?;

	for drug in imports {
		let drug_id = DrugInformationBmc::create(
			ctx,
			mm,
			DrugInformationForCreate {
				case_id,
				sequence_number: drug.sequence_number,
				drug_characterization: drug.drug_characterization.clone(),
				medicinal_product: drug.medicinal_product.clone(),
			},
		)
		.await?;

		DrugInformationBmc::update(
			ctx,
			mm,
			drug_id,
			DrugInformationForUpdate {
				medicinal_product: Some(drug.medicinal_product),
				drug_characterization: Some(drug.drug_characterization),
				brand_name: drug.brand_name,
				manufacturer_name: drug.manufacturer_name,
				manufacturer_country: drug.manufacturer_country,
				batch_lot_number: drug.batch_lot_number,
				dosage_text: drug.dosage_text,
				action_taken: drug.action_taken,
				rechallenge: drug.rechallenge,
				investigational_product_blinded: drug.investigational_product_blinded,
				mpid: drug.mpid,
				mpid_version: drug.mpid_version,
				obtain_drug_country: drug.obtain_drug_country,
				parent_route: drug.parent_route,
				parent_route_termid: drug.parent_route_termid,
				parent_route_termid_version: drug.parent_route_termid_version,
				parent_dosage_text: drug.parent_dosage_text,
				fda_additional_info_coded: drug.fda_additional_info_coded,
			},
		)
		.await?;

		for (sidx, sub) in drug.substances.into_iter().enumerate() {
			let _ = DrugActiveSubstanceBmc::create(
				ctx,
				mm,
				DrugActiveSubstanceForCreate {
					drug_id,
					sequence_number: (sidx + 1) as i32,
					substance_name: sub.substance_name,
					substance_termid: sub.substance_termid,
					substance_termid_version: sub.substance_termid_version,
					strength_value: sub.strength_value,
					strength_unit: sub.strength_unit,
				},
			)
			.await?;
		}

		for (didx, dose) in drug.dosages.into_iter().enumerate() {
			let _ = DosageInformationBmc::create(
				ctx,
				mm,
				DosageInformationForCreate {
					drug_id,
					sequence_number: (didx + 1) as i32,
					dose_value: dose.dose_value,
					dose_unit: dose.dose_unit,
					frequency_value: dose.frequency_value,
					frequency_unit: dose.frequency_unit,
					first_administration_date: dose.start_date,
					first_administration_time: None,
					last_administration_date: dose.end_date,
					last_administration_time: None,
					duration_value: dose.duration_value,
					duration_unit: dose.duration_unit,
					batch_lot_number: dose.batch_lot,
					dosage_text: dose.dosage_text,
					dose_form: dose.dose_form,
					dose_form_termid: dose.dose_form_termid,
					dose_form_termid_version: dose.dose_form_termid_version,
					route_of_administration: dose.route,
					parent_route: dose.parent_route,
					parent_route_termid: dose.parent_route_termid,
					parent_route_termid_version: dose.parent_route_termid_version,
					number_of_units: None,
					first_administration_date_null_flavor: None,
					last_administration_date_null_flavor: None,
				},
			)
			.await?;
		}

		for (iidx, ind) in drug.indications.into_iter().enumerate() {
			let _ = DrugIndicationBmc::create(
				ctx,
				mm,
				DrugIndicationForCreate {
					drug_id,
					sequence_number: (iidx + 1) as i32,
					indication_text: ind.text,
					indication_meddra_version: ind.version,
					indication_meddra_code: ind.code,
				},
			)
			.await?;
		}

		for (cidx, ch) in drug.characteristics.into_iter().enumerate() {
			let _ = DrugDeviceCharacteristicBmc::create(
				ctx,
				mm,
				DrugDeviceCharacteristicForCreate {
					drug_id,
					sequence_number: (cidx + 1) as i32,
					code: ch.code,
					code_system: ch.code_system,
					code_display_name: ch.code_display_name,
					value_type: ch.value_type,
					value_value: ch.value_value,
					value_code: ch.value_code,
					value_code_system: ch.value_code_system,
					value_display_name: ch.value_display_name,
				},
			)
			.await?;
		}
	}

	Ok(())
}

fn parse_drugs(xml: &[u8]) -> Result<Vec<DrugImport>> {
	let xml_str = std::str::from_utf8(xml).map_err(|err| Error::InvalidXml {
		message: format!("XML not valid UTF-8: {err}"),
		line: None,
		column: None,
	})?;
	let parser = Parser::default();
	let doc = parser.parse_string(xml_str).map_err(|err| Error::InvalidXml {
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

	let drug_nodes = xpath
		.findnodes(
			"//hl7:subjectOf2/hl7:organizer[hl7:code[@code='4' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.20']]/hl7:component/hl7:substanceAdministration",
			None,
		)
		.map_err(|_| Error::InvalidXml {
			message: "Failed to query drug information".to_string(),
			line: None,
			column: None,
		})?;

	let mut imports: Vec<DrugImport> = Vec::new();
	for (idx, node) in drug_nodes.into_iter().enumerate() {
		let name1 = first_text(
			&mut xpath,
			&node,
			"hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:name[1]",
		)
		.unwrap_or_else(|| "UNKNOWN".to_string());
		let name2 = first_text(
			&mut xpath,
			&node,
			"hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:name[2]",
		);

		let drug_characterization = "1".to_string();
		let mpid = first_attr(
			&mut xpath,
			&node,
			"hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:code",
			"code",
		);
		let mpid_version = clamp_str(
			first_attr(
				&mut xpath,
				&node,
				"hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:code",
				"codeSystemVersion",
			),
			10,
			"drug_information.mpid_version",
		);
		let investigational_product_blinded = first_attr(
			&mut xpath,
			&node,
			"hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:subjectOf/hl7:observation[hl7:code[@code='G.k.2.5']]/hl7:value",
			"value",
		)
		.and_then(|v| match v.to_ascii_lowercase().as_str() {
			"true" | "1" => Some(true),
			"false" | "0" => Some(false),
			_ => None,
		});
		let manufacturer_name = first_text(
			&mut xpath,
			&node,
			"hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:asManufacturedProduct/hl7:subjectOf/hl7:approval/hl7:holder/hl7:role/hl7:playingOrganization/hl7:name",
		);
		let manufacturer_country = normalize_iso2(
			first_attr(
				&mut xpath,
				&node,
				"hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:asManufacturedProduct/hl7:subjectOf/hl7:approval/hl7:author/hl7:territorialAuthority/hl7:territory/hl7:code",
				"code",
			),
			"drug_information.manufacturer_country",
		);
		let obtain_drug_country = normalize_iso2(
			first_text(
				&mut xpath,
				&node,
				"hl7:consumable/hl7:instanceOfKind/hl7:subjectOf/hl7:productEvent/hl7:performer/hl7:assignedEntity/hl7:representedOrganization/hl7:addr/hl7:country",
			),
			"drug_information.obtain_drug_country",
		);
		let action_taken = normalize_code(
			first_attr(
				&mut xpath,
				&node,
				"hl7:inboundRelationship[@typeCode='CAUS']/hl7:act/hl7:code",
				"code",
			),
			&["1", "2", "3", "4", "5", "6"],
			"drug_information.action_taken",
		)
		.or_else(|| Some("5".to_string()));
		if let Some(val) = action_taken.as_deref() {
			eprintln!("[import_e2b_xml] action_taken={val}");
		}
		let rechallenge = normalize_code(
			first_attr(
				&mut xpath,
				&node,
				"hl7:outboundRelationship2/hl7:observation[hl7:code[@code='31']]/hl7:value",
				"code",
			),
			&["1", "2", "3", "4"],
			"drug_information.rechallenge",
		);
		let dosage_text = first_text(&mut xpath, &node, "hl7:text");
		let batch_lot_number = first_text(
			&mut xpath,
			&node,
			"hl7:consumable/hl7:instanceOfKind/hl7:productInstanceInstance/hl7:lotNumberText",
		);
		let fda_additional_info_coded = clamp_str(
			first_attr(
				&mut xpath,
				&node,
				"hl7:outboundRelationship2[@typeCode='REFR']/hl7:observation[hl7:code[@code='9']]/hl7:value",
				"code",
			),
			10,
			"drug_information.fda_additional_info_coded",
		);
		let parent_route_termid_version = clamp_str(
			first_attr(
				&mut xpath,
				&node,
				"hl7:outboundRelationship2/hl7:observation[hl7:code[@code='G.k.4.r.11']]/hl7:value",
				"codeSystemVersion",
			),
			10,
			"drug_information.parent_route_termid_version",
		);
		let parent_route_termid = first_attr(
			&mut xpath,
			&node,
			"hl7:outboundRelationship2/hl7:observation[hl7:code[@code='G.k.4.r.11']]/hl7:value",
			"code",
		);
		let parent_dosage_text = first_text(
			&mut xpath,
			&node,
			"hl7:outboundRelationship2[@typeCode='REFR']/hl7:observation[hl7:code[@code='2']]/hl7:value",
		);
		let parent_route = first_text(
			&mut xpath,
			&node,
			"hl7:outboundRelationship2/hl7:observation[hl7:code[@code='G.k.4.r.11']]/hl7:value/hl7:originalText",
		);

		let subs = xpath
			.findnodes(
				"hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:ingredient",
				Some(&node),
			)
			.unwrap_or_default();
		let mut substances = Vec::new();
		for sub in subs.into_iter() {
			let sub_name = first_text(&mut xpath, &sub, "hl7:ingredientSubstance/hl7:name");
			let termid =
				first_attr(&mut xpath, &sub, "hl7:ingredientSubstance/hl7:code", "code");
			let termid_version = clamp_str(
				first_attr(
					&mut xpath,
					&sub,
					"hl7:ingredientSubstance/hl7:code",
					"codeSystemVersion",
				),
				10,
				"drug_active_substances.substance_termid_version",
			);
			let strength_value = first_attr(&mut xpath, &sub, "hl7:quantity/hl7:numerator", "value")
				.and_then(|v| v.parse::<Decimal>().ok());
			let strength_unit =
				first_attr(&mut xpath, &sub, "hl7:quantity/hl7:numerator", "unit");
			substances.push(DrugSubstanceImport {
				substance_name: sub_name,
				substance_termid: termid,
				substance_termid_version: termid_version,
				strength_value,
				strength_unit,
			});
		}

		let dosages = xpath
			.findnodes(
				"hl7:outboundRelationship2[@typeCode='COMP']/hl7:substanceAdministration",
				Some(&node),
			)
			.unwrap_or_default();
		let mut dosage_list = Vec::new();
		for dose in dosages.into_iter() {
			let dosage_text = first_text(&mut xpath, &dose, "hl7:text");
			let frequency_value = first_attr(
				&mut xpath,
				&dose,
				"hl7:effectiveTime/hl7:comp[@xsi:type='PIVL_TS']/hl7:period",
				"value",
			)
			.and_then(|v| v.parse::<Decimal>().ok());
			let frequency_unit = normalize_code3(
				first_attr(
					&mut xpath,
					&dose,
					"hl7:effectiveTime/hl7:comp[@xsi:type='PIVL_TS']/hl7:period",
					"unit",
				),
				"dosage_information.frequency_unit",
			);
			let start_date = first_attr(
				&mut xpath,
				&dose,
				"hl7:effectiveTime/hl7:comp[@operator='A']/hl7:low",
				"value",
			)
			.and_then(parse_date);
			let end_date = first_attr(
				&mut xpath,
				&dose,
				"hl7:effectiveTime/hl7:comp[@operator='A']/hl7:high",
				"value",
			)
			.and_then(parse_date);
			let duration_value = first_attr(
				&mut xpath,
				&dose,
				"hl7:effectiveTime/hl7:comp[@operator='A']/hl7:width",
				"value",
			)
			.and_then(|v| v.parse::<Decimal>().ok());
			let duration_unit = normalize_code3(
				first_attr(
					&mut xpath,
					&dose,
					"hl7:effectiveTime/hl7:comp[@operator='A']/hl7:width",
					"unit",
				),
				"dosage_information.duration_unit",
			);
			let dose_value = first_attr(&mut xpath, &dose, "hl7:doseQuantity", "value")
				.and_then(|v| v.parse::<Decimal>().ok());
			let dose_unit = first_attr(&mut xpath, &dose, "hl7:doseQuantity", "unit");
			let route = normalize_code3(
				first_attr(&mut xpath, &dose, "hl7:routeCode", "code"),
				"dosage_information.route_of_administration",
			);
			let dose_form = first_text(
				&mut xpath,
				&dose,
				"hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:formCode/hl7:originalText",
			);
			let dose_form_termid = first_attr(
				&mut xpath,
				&dose,
				"hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:formCode",
				"code",
			);
			let dose_form_termid_version = clamp_str(
				first_attr(
					&mut xpath,
					&dose,
					"hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:formCode",
					"codeSystemVersion",
				),
				10,
				"dosage_information.dose_form_termid_version",
			);
			let batch_lot = first_text(
				&mut xpath,
				&dose,
				"hl7:consumable/hl7:instanceOfKind/hl7:productInstanceInstance/hl7:lotNumberText",
			);
			let parent_route_termid = first_attr(
				&mut xpath,
				&dose,
				"hl7:outboundRelationship2/hl7:observation[hl7:code[@code='G.k.4.r.11']]/hl7:value",
				"code",
			);
			let parent_route_termid_version = clamp_str(
				first_attr(
					&mut xpath,
					&dose,
					"hl7:outboundRelationship2/hl7:observation[hl7:code[@code='G.k.4.r.11']]/hl7:value",
					"codeSystemVersion",
				),
				10,
				"dosage_information.parent_route_termid_version",
			);
			let parent_route = first_text(
				&mut xpath,
				&dose,
				"hl7:outboundRelationship2/hl7:observation[hl7:code[@code='G.k.4.r.11']]/hl7:value/hl7:originalText",
			);

			dosage_list.push(DrugDosageImport {
				dosage_text,
				frequency_value,
				frequency_unit,
				start_date,
				end_date,
				duration_value,
				duration_unit,
				dose_value,
				dose_unit,
				route,
				dose_form,
				dose_form_termid,
				dose_form_termid_version,
				batch_lot,
				parent_route_termid,
				parent_route_termid_version,
				parent_route,
			});
		}

		let inds = xpath
			.findnodes(
				"hl7:inboundRelationship[@typeCode='RSON']/hl7:observation/hl7:value",
				Some(&node),
			)
			.unwrap_or_default();
		let mut indications = Vec::new();
		for ind in inds.into_iter() {
			let text = first_text(&mut xpath, &ind, "hl7:originalText");
			let code = first_attr(&mut xpath, &ind, ".", "code");
			let version = clamp_str(
				first_attr(&mut xpath, &ind, ".", "codeSystemVersion"),
				10,
				"drug_indications.indication_meddra_version",
			);
			indications.push(DrugIndicationImport { text, version, code });
		}

		let chars = xpath
			.findnodes(
				"hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:part/hl7:partProduct/hl7:asManufacturedProduct/hl7:subjectOf/hl7:characteristic",
				Some(&node),
			)
			.unwrap_or_default();
		let mut characteristics = Vec::new();
		for ch in chars.into_iter() {
			let code = first_attr(&mut xpath, &ch, "hl7:code", "code");
			let code_system = first_attr(&mut xpath, &ch, "hl7:code", "codeSystem");
			let code_display_name = first_attr(&mut xpath, &ch, "hl7:code", "displayName");
			let value_type = clamp_str(
				first_attr(&mut xpath, &ch, "hl7:value", "xsi:type")
					.or_else(|| first_attr(&mut xpath, &ch, "hl7:value", "type")),
				10,
				"drug_device_characteristics.value_type",
			);
			let value_value = first_attr(&mut xpath, &ch, "hl7:value", "value");
			let value_code = first_attr(&mut xpath, &ch, "hl7:value", "code");
			let value_code_system = first_attr(&mut xpath, &ch, "hl7:value", "codeSystem");
			let value_display_name = first_attr(&mut xpath, &ch, "hl7:value", "displayName");
			characteristics.push(DrugDeviceCharacteristicImport {
				code,
				code_system,
				code_display_name,
				value_type,
				value_value,
				value_code,
				value_code_system,
				value_display_name,
			});
		}

		imports.push(DrugImport {
			sequence_number: (idx + 1) as i32,
			medicinal_product: name1,
			brand_name: name2,
			drug_characterization,
			mpid,
			mpid_version,
			investigational_product_blinded,
			obtain_drug_country,
			manufacturer_name,
			manufacturer_country,
			batch_lot_number,
			dosage_text,
			action_taken,
			rechallenge,
			parent_route,
			parent_route_termid,
			parent_route_termid_version,
			parent_dosage_text,
			fda_additional_info_coded,
			substances,
			dosages: dosage_list,
			indications,
			characteristics,
		});
	}

	Ok(imports)
}
