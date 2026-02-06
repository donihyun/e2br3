use crate::xml::error::Error;
use crate::xml::types::{XmlValidationError, XmlValidationReport};
use crate::xml::Result;
use libxml::parser::Parser;
use libxml::schemas::{SchemaParserContext, SchemaValidationContext};
use libxml::xpath::Context;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct XmlValidatorConfig {
	pub max_bytes: usize,
	pub allowed_roots: &'static [&'static str],
	pub xsd_path: Option<PathBuf>,
	pub require_schema_location: bool,
	pub require_its_version: Option<&'static str>,
}

impl Default for XmlValidatorConfig {
	fn default() -> Self {
		Self {
			max_bytes: 5 * 1024 * 1024,
			allowed_roots: &["MCCI_IN200100UV01", "MCCI_IN200101UV01"],
			xsd_path: xsd_path_from_env(),
			require_schema_location: true,
			require_its_version: Some("XML_1.0"),
		}
	}
}

pub fn validate_e2b_xml(
	xml: &[u8],
	config: Option<XmlValidatorConfig>,
) -> Result<XmlValidationReport> {
	let config = config.unwrap_or_default();
	if xml.len() > config.max_bytes {
		return Ok(XmlValidationReport {
			ok: false,
			errors: vec![XmlValidationError {
				message: format!(
					"XML payload exceeds max size ({} bytes)",
					config.max_bytes
				),
				line: None,
				column: None,
			}],
			root_element: None,
		});
	}

	let mut reader = Reader::from_reader(xml);
	reader.trim_text(true);
	let mut buf = Vec::new();
	let mut root: Option<String> = None;
	let mut errors: Vec<XmlValidationError> = Vec::new();

	loop {
		match reader.read_event_into(&mut buf) {
			Ok(Event::Start(e)) => {
				if root.is_none() {
					let name_bytes = e.name().as_ref().to_vec();
					root = Some(String::from_utf8_lossy(&name_bytes).to_string());
				}
			}
			Ok(Event::Eof) => break,
			Ok(_) => {}
			Err(e) => {
				let pos = reader.buffer_position();
				errors.push(XmlValidationError {
					message: format!("XML parse error: {e}"),
					line: None,
					column: Some(pos),
				});
				break;
			}
		}
		buf.clear();
	}

	if root.is_none() {
		errors.push(XmlValidationError {
			message: "Missing root element".to_string(),
			line: None,
			column: None,
		});
	}

	if let Some(root_name) = &root {
		if !config.allowed_roots.iter().any(|v| *v == root_name) {
			errors.push(XmlValidationError {
				message: format!(
					"Unexpected root element '{root_name}', expected one of [{}]",
					config.allowed_roots.join(", ")
				),
				line: None,
				column: None,
			});
		}
	}

	if let Some(xsd_path) = config.xsd_path.as_ref() {
		let mut xsd_errors = validate_e2b_xml_xsd(xml, xsd_path)?;
		let xml_str = std::str::from_utf8(xml).unwrap_or_default();
		let has_porr = xml_str.contains("<PORR_IN049016UV")
			|| xml_str.contains("<PORR_IN049017UV")
			|| xml_str.contains("<PORR_IN049018UV");
		if has_porr {
			xsd_errors.retain(|err| {
				!err.message.contains("Missing child element(s)")
					|| !err.message.contains("PORR_IN04901")
			});
		}
		errors.append(&mut xsd_errors);
	} else {
		errors.push(XmlValidationError {
			message: "XSD validation not configured (set E2BR3_XSD_PATH)"
				.to_string(),
			line: None,
			column: None,
		});
	}

	let mut rule_errors = validate_e2b_xml_rules(xml, &config)?;
	errors.append(&mut rule_errors);

	Ok(XmlValidationReport {
		ok: errors.is_empty(),
		errors,
		root_element: root,
	})
}

pub fn should_skip_xml_validation() -> bool {
	match std::env::var("E2BR3_SKIP_XML_VALIDATE") {
		Ok(value) => {
			let value = value.trim().to_ascii_lowercase();
			value == "1" || value == "true" || value == "yes"
		}
		Err(_) => false,
	}
}

pub fn validate_e2b_xml_xsd(
	xml: &[u8],
	xsd_path: &Path,
) -> Result<Vec<XmlValidationError>> {
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

	let mut schema_parser = SchemaParserContext::from_file(
		xsd_path
			.to_str()
			.ok_or(Error::InvalidXml {
				message: "XSD path is not valid UTF-8".to_string(),
				line: None,
				column: None,
			})?,
	);
	let mut ctx = SchemaValidationContext::from_parser(&mut schema_parser)
		.map_err(|errors| Error::InvalidXml {
			message: format!(
				"XSD parse error: {}",
				errors
					.first()
					.and_then(|e| e.message.as_deref())
					.unwrap_or("unknown")
			),
			line: None,
			column: None,
		})?;

	match ctx.validate_document(&doc) {
		Ok(()) => Ok(Vec::new()),
		Err(errors) => {
			let mut out = Vec::new();
			for err in errors {
				out.push(XmlValidationError {
					message: err.message.unwrap_or_else(|| "XSD validation error".to_string()),
					line: err.line.map(|v| v as usize),
					column: err.col.map(|v| v as usize),
				});
			}
			Ok(out)
		}
	}
}

fn xsd_path_from_env() -> Option<PathBuf> {
	std::env::var("E2BR3_XSD_PATH")
		.ok()
		.map(PathBuf::from)
}

fn validate_e2b_xml_rules(
	xml: &[u8],
	config: &XmlValidatorConfig,
) -> Result<Vec<XmlValidationError>> {
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
	let root = doc.get_root_element().ok_or(Error::MissingRootElement)?;
	let root_name = root.get_name();
	let mut errors = Vec::new();
	let mut xpath = Context::new(&doc).map_err(|_| Error::InvalidXml {
		message: "Failed to initialize XPath context".to_string(),
		line: None,
		column: None,
	})?;
	let _ = xpath.register_namespace("hl7", "urn:hl7-org:v3");
	let _ = xpath.register_namespace(
		"xsi",
		"http://www.w3.org/2001/XMLSchema-instance",
	);

	if let Some(req) = config.require_its_version {
		match root.get_attribute("ITSVersion") {
			Some(value) if value == req => {}
			Some(value) => errors.push(XmlValidationError {
				message: format!(
					"ITSVersion '{value}' does not match required '{req}'"
				),
				line: None,
				column: None,
			}),
			None => errors.push(XmlValidationError {
				message: "Missing ITSVersion attribute on root".to_string(),
				line: None,
				column: None,
			}),
		}
	}

	if config.require_schema_location {
		let schema_location = root
			.get_attribute_ns(
				"schemaLocation",
				"http://www.w3.org/2001/XMLSchema-instance",
			)
			.or_else(|| root.get_attribute("xsi:schemaLocation"));

		match schema_location {
			Some(value) => {
				let expected = format!("{root_name}.xsd");
				if !value.contains(&expected) {
					errors.push(XmlValidationError {
						message: format!(
							"schemaLocation missing expected '{expected}'"
						),
						line: None,
						column: None,
					});
				}
			}
			None => errors.push(XmlValidationError {
				message: "Missing xsi:schemaLocation on root".to_string(),
				line: None,
				column: None,
			}),
		}
	}

	// Rule: telecom values must use tel:/fax:/mailto:
	if let Ok(values) = xpath.findvalues("//hl7:telecom/@value", None) {
		for value in values {
			if !(value.starts_with("tel:")
				|| value.starts_with("fax:")
				|| value.starts_with("mailto:"))
			{
				errors.push(XmlValidationError {
					message: format!(
						"telecom value must start with tel:, fax:, or mailto:, got '{value}'"
					),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: telecom missing/blank value requires nullFlavor
	if let Ok(nodes) = xpath.findnodes("//hl7:telecom", None) {
		for node in nodes {
			let value = node.get_attribute("value");
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if value.as_deref().unwrap_or("").trim().is_empty()
				&& !has_null_flavor
			{
				errors.push(XmlValidationError {
					message: "telecom missing value; nullFlavor is required"
						.to_string(),
					line: None,
					column: None,
				});
			}
			if value
				.as_deref()
				.map(|v| !v.trim().is_empty())
				.unwrap_or(false)
				&& has_null_flavor
			{
				errors.push(XmlValidationError {
					message: "telecom has value and nullFlavor; nullFlavor must be absent when value present"
						.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: if ingredientSubstance/name is empty, nullFlavor is required
	if let Ok(nodes) = xpath.findnodes("//hl7:ingredientSubstance/hl7:name", None) {
		for node in nodes {
			let content = node.get_content();
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if content.trim().is_empty() && !has_null_flavor {
				errors.push(XmlValidationError {
					message: "ingredientSubstance/name is empty; nullFlavor is required"
						.to_string(),
					line: None,
					column: None,
				});
			}
			if !content.trim().is_empty() && has_null_flavor {
				errors.push(XmlValidationError {
					message:
						"ingredientSubstance/name has value and nullFlavor; nullFlavor must be absent when value present"
							.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: if primary reporter name fields are empty, nullFlavor is required
	if let Ok(nodes) = xpath.findnodes("//hl7:primaryRole//hl7:name/*", None) {
		for node in nodes {
			let content = node.get_content();
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if content.trim().is_empty() && !has_null_flavor {
				errors.push(XmlValidationError {
					message: format!(
						"primaryRole name element '{}' is empty; nullFlavor is required",
						node.get_name()
					),
					line: None,
					column: None,
				});
			}
			if !content.trim().is_empty() && has_null_flavor {
				errors.push(XmlValidationError {
					message: format!(
						"primaryRole name element '{}' has value and nullFlavor; nullFlavor must be absent when value present",
						node.get_name()
					),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: organization name empty requires nullFlavor
	if let Ok(nodes) = xpath.findnodes(
		"//hl7:representedOrganization/hl7:name",
		None,
	) {
		for node in nodes {
			let content = node.get_content();
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if content.trim().is_empty() && !has_null_flavor {
				errors.push(XmlValidationError {
					message:
						"representedOrganization/name is empty; nullFlavor is required"
							.to_string(),
					line: None,
					column: None,
				});
			}
			if !content.trim().is_empty() && has_null_flavor {
				errors.push(XmlValidationError {
					message:
						"representedOrganization/name has value and nullFlavor; nullFlavor must be absent when value present"
							.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: primaryRole id missing extension requires nullFlavor
	if let Ok(nodes) = xpath.findnodes(
		"//hl7:primaryRole/hl7:id",
		None,
	) {
		for node in nodes {
			let extension = node.get_attribute("extension");
			let root_attr = node.get_attribute("root");
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if extension.as_deref().unwrap_or("").trim().is_empty()
				&& !has_null_flavor
			{
				errors.push(XmlValidationError {
					message:
						"primaryRole/id missing extension; nullFlavor is required"
							.to_string(),
					line: None,
					column: None,
				});
			}
			if root_attr.as_deref()
				.map(|v| v == "2.16.840.1.113883.3.989.2.1.3.6")
				.unwrap_or(false)
				&& extension.as_deref().unwrap_or("").trim().is_empty()
				&& !has_null_flavor
			{
				errors.push(XmlValidationError {
					message:
						"primaryRole/id with root 2.16.840.1.113883.3.989.2.1.3.6 requires extension or nullFlavor"
							.to_string(),
					line: None,
					column: None,
				});
			}
			if extension
				.as_deref()
				.map(|v| !v.trim().is_empty())
				.unwrap_or(false)
				&& has_null_flavor
			{
				errors.push(XmlValidationError {
					message:
						"primaryRole/id has extension and nullFlavor; nullFlavor must be absent when value present"
							.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: birthTime empty requires nullFlavor
	if let Ok(nodes) =
		xpath.findnodes("//hl7:primaryRole//hl7:birthTime", None)
	{
		for node in nodes {
			let value = node.get_attribute("value");
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if value.as_deref().unwrap_or("").trim().is_empty()
				&& !has_null_flavor
			{
				errors.push(XmlValidationError {
					message: "birthTime missing value; nullFlavor is required"
						.to_string(),
					line: None,
					column: None,
				});
			}
			if value
				.as_deref()
				.map(|v| !v.trim().is_empty())
				.unwrap_or(false)
				&& has_null_flavor
			{
				errors.push(XmlValidationError {
					message:
						"birthTime has value and nullFlavor; nullFlavor must be absent when value present"
							.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: narrative/free text empty requires nullFlavor
	if let Ok(nodes) = xpath.findnodes(
		"//hl7:text | //hl7:originalText",
		None,
	) {
		for node in nodes {
			let content = node.get_content();
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if content.trim().is_empty() && !has_null_flavor {
				errors.push(XmlValidationError {
					message: format!(
						"{} is empty; nullFlavor is required",
						node.get_name()
					),
					line: None,
					column: None,
				});
			}
			if !content.trim().is_empty() && has_null_flavor {
				errors.push(XmlValidationError {
					message: format!(
						"{} has value and nullFlavor; nullFlavor must be absent when value present",
						node.get_name()
					),
					line: None,
					column: None,
				});
			}
		}
	}

	// Determine whether FDA-specific rules should be enforced
	let batch_receiver = xpath
		.findvalues(
			"/hl7:MCCI_IN200100UV01/hl7:receiver/hl7:device/hl7:id/@extension",
			None,
		)
		.ok()
		.and_then(|vals| vals.get(0).cloned());
	let msg_receiver = xpath
		.findvalues(
			"/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV/hl7:receiver/hl7:device/hl7:id/@extension",
			None,
		)
		.ok()
		.and_then(|vals| vals.get(0).cloned());
	let is_fda = matches!(
		batch_receiver.as_deref(),
		Some("ZZFDA") | Some("ZZFDA_PREMKT")
	) || matches!(
		msg_receiver.as_deref(),
		Some("CDER")
			| Some("CBER")
			| Some("CDER_IND")
			| Some("CBER_IND")
			| Some("CDER_IND_EXEMPT_BA_BE")
	);

		if is_fda {
			// FDA Rule: N.1.4 Batch Receiver Identifier is required
			let has_batch_receiver = batch_receiver
				.as_deref()
				.map(|val| !val.trim().is_empty())
				.unwrap_or(false);
			if !has_batch_receiver {
				errors.push(XmlValidationError {
					message: "FDA.N.1.4 batch receiver identifier missing".to_string(),
					line: None,
					column: None,
				});
			}

			// FDA Rule: Combination Product Report Indicator must have value or nullFlavor
			if let Ok(nodes) = xpath.findnodes(
				"//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.5.1.2.2.1.3']]/hl7:value",
			None,
		) {
			for node in nodes {
				let value = node.get_attribute("value");
				let has_null_flavor = node.get_attribute("nullFlavor").is_some();
				if value.as_deref().unwrap_or("").trim().is_empty() && !has_null_flavor {
					errors.push(XmlValidationError {
						message: "FDA.C.1.12 combination product indicator missing value; nullFlavor is required"
							.to_string(),
						line: None,
						column: None,
					});
				}
			}
		}

		// FDA Rule: Local Criteria Report Type must have code or nullFlavor
		if let Ok(nodes) = xpath.findnodes(
			"//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='2' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value",
			None,
		) {
			for node in nodes {
				let code = node.get_attribute("code");
				let has_null_flavor = node.get_attribute("nullFlavor").is_some();
				if code.as_deref().unwrap_or("").trim().is_empty() && !has_null_flavor {
					errors.push(XmlValidationError {
						message: "FDA.C.1.7.1 local criteria report type missing code; nullFlavor is required"
							.to_string(),
						line: None,
						column: None,
					});
				}
			}
		}

			// FDA Rule: Patient race must have code or nullFlavor
			if let Ok(nodes) = xpath.findnodes(
				"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='C17049' and @codeSystem='2.16.840.1.113883.3.26.1.1']]/hl7:value",
				None,
			) {
				for node in nodes {
					let code = node.get_attribute("code");
					let has_null_flavor = node.get_attribute("nullFlavor").is_some();
				if code.as_deref().unwrap_or("").trim().is_empty() && !has_null_flavor {
					errors.push(XmlValidationError {
						message: "FDA.D.11 patient race missing code; nullFlavor is required"
							.to_string(),
						line: None,
						column: None,
					});
				}
			}
		}

		// FDA Rule: Patient ethnicity must have code or nullFlavor
		if let Ok(nodes) = xpath.findnodes(
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='C16564' and @codeSystem='2.16.840.1.113883.3.26.1.1']]/hl7:value",
			None,
		) {
			for node in nodes {
				let code = node.get_attribute("code");
				let has_null_flavor = node.get_attribute("nullFlavor").is_some();
				if code.as_deref().unwrap_or("").trim().is_empty() && !has_null_flavor {
					errors.push(XmlValidationError {
						message: "FDA.D.12 patient ethnicity missing code; nullFlavor is required"
							.to_string(),
						line: None,
						column: None,
					});
				}
			}
		}

		// FDA Rule: Required Intervention must have value or nullFlavor when present
		if let Ok(nodes) = xpath.findnodes(
			"//hl7:observation[hl7:code[@code='29' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]//hl7:outboundRelationship2/hl7:observation[hl7:code[@code='726' and @codeSystem='2.16.840.1.113883.3.989.5.1.2.2.1.32']]/hl7:value",
			None,
		) {
			for node in nodes {
				let value = node.get_attribute("value");
				let has_null_flavor = node.get_attribute("nullFlavor").is_some();
				if value.as_deref().unwrap_or("").trim().is_empty() && !has_null_flavor {
					errors.push(XmlValidationError {
						message: "FDA.E.i.3.2h required intervention missing value; nullFlavor is required"
							.to_string(),
						line: None,
						column: None,
					});
				}
			}
		}
	}

	// FDA Rule: Local Criteria Report Type codes depend on Combination Product indicator and C.1.7
	let comb_ind = xpath
		.findvalues(
			"//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.5.1.2.2.1.3']]/hl7:value/@value",
			None,
		)
		.ok()
		.and_then(|vals| vals.get(0).cloned());
	let local_criteria = xpath
		.findvalues(
			"//hl7:component/hl7:observationEvent[hl7:code[@code='23' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value/@value",
			None,
		)
		.ok()
		.and_then(|vals| vals.get(0).cloned());
	let local_criteria_code = xpath
		.findvalues(
			"//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='2' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value/@code",
			None,
		)
		.ok()
		.and_then(|vals| vals.get(0).cloned());
	if let (Some(comb), Some(criteria), Some(code)) = (
		comb_ind.as_deref(),
		local_criteria.as_deref(),
		local_criteria_code.as_deref(),
	)
	{
		let comb_true = comb.eq_ignore_ascii_case("true");
		let criteria_true = criteria.eq_ignore_ascii_case("true");
		let allowed: &[&str] = if comb_true && criteria_true {
			&["1", "4"]
		} else if comb_true && !criteria_true {
			&["2", "5"]
		} else if !comb_true && criteria_true {
			&["1"]
		} else {
			&["2"]
		};
		if !allowed.contains(&code) {
			errors.push(XmlValidationError {
				message: format!(
					"FDA.C.1.7.1 local criteria report type '{code}' invalid for combination_product={comb} expedited={criteria}"
				),
				line: None,
				column: None,
			});
		}
	}

	// FDA Rule: G.k.10a required when FDA.C.5.5b present (codes 1 or 2, or nullFlavor NA)
	let pre_anda = xpath
		.findvalues(
			"//hl7:researchStudy/hl7:authorization/hl7:studyRegistration/hl7:id[@root='2.16.840.1.113883.3.989.5.1.2.2.1.2.2']/@extension",
			None,
		)
		.ok()
		.and_then(|vals| vals.get(0).cloned());
	if pre_anda
		.as_deref()
		.map(|v| !v.trim().is_empty())
		.unwrap_or(false)
	{
		let values = xpath
			.findnodes(
				"//hl7:organizer[hl7:code[@code='4' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.20']]/hl7:component/hl7:substanceAdministration/hl7:outboundRelationship2[@typeCode='REFR']/hl7:observation[hl7:code[@code='9']]/hl7:value",
				None,
			)
			.unwrap_or_default();
		if values.is_empty() {
			errors.push(XmlValidationError {
				message: "FDA.G.k.10a missing: required when FDA.C.5.5b is present"
					.to_string(),
				line: None,
				column: None,
			});
		} else {
			for node in values {
				let code = node.get_attribute("code");
				let null_flavor = node.get_attribute("nullFlavor");
				let code_ok = code
					.as_deref()
					.map(|v| v == "1" || v == "2")
					.unwrap_or(false);
				let null_ok = null_flavor
					.as_deref()
					.map(|v| v == "NA")
					.unwrap_or(false);
				if !(code_ok || null_ok) {
					errors.push(XmlValidationError {
						message:
							"FDA.G.k.10a must be code 1/2 or nullFlavor NA when FDA.C.5.5b is present"
								.to_string(),
						line: None,
						column: None,
					});
				}
			}
		}
	}

	// FDA Rule: Reporter email required when primary source present
	let has_primary_source = xpath
		.findnodes(
			"//hl7:outboundRelationship[@typeCode='SPRT']/hl7:relatedInvestigation/hl7:subjectOf2/hl7:controlActEvent/hl7:author/hl7:assignedEntity",
			None,
		)
		.map(|nodes| !nodes.is_empty())
		.unwrap_or(false);
	if is_fda && has_primary_source {
		let has_email = xpath
			.findvalues(
				"//hl7:outboundRelationship[@typeCode='SPRT']/hl7:relatedInvestigation/hl7:subjectOf2/hl7:controlActEvent/hl7:author/hl7:assignedEntity/hl7:telecom/@value",
				None,
			)
			.ok()
			.map(|vals| vals.iter().any(|v| v.starts_with("mailto:")))
			.unwrap_or(false);
		if !has_email {
			errors.push(XmlValidationError {
				message: "FDA requires reporter email when primary source is present"
					.to_string(),
				line: None,
				column: None,
			});
		}
	}

	// FDA Rule: C.1.3 report type must be 2 under specific premarket conditions
	let study_type = xpath
		.findvalues(
			"//hl7:researchStudy/hl7:code/@code",
			None,
		)
		.ok()
		.and_then(|vals| vals.get(0).cloned());
	let type_of_report = xpath
		.findvalues(
			"//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.23']]/hl7:value/@code",
			None,
		)
		.ok()
		.and_then(|vals| vals.get(0).cloned());
	let type_of_report = type_of_report.or_else(|| {
		xpath
			.findvalues(
				"//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.23']]/hl7:value/@code",
				None,
			)
			.ok()
			.and_then(|vals| vals.get(0).cloned())
	});
	let has_pre_anda = pre_anda
		.as_deref()
		.map(|v| !v.trim().is_empty())
		.unwrap_or(false);
	if is_fda && batch_receiver.as_deref() == Some("ZZFDA_PREMKT") {
		if let Some(receiver) = msg_receiver.as_deref() {
			let premkt_rcv = ["CDER_IND", "CBER_IND", "CDER_IND_EXEMPT_BA_BE"];
			if premkt_rcv.contains(&receiver) {
				let study_match = study_type
					.as_deref()
					.map(|v| v == "1" || v == "2" || v == "3")
					.unwrap_or(false);
				if has_pre_anda && study_match {
					if type_of_report.as_deref() != Some("2") {
						errors.push(XmlValidationError {
							message: "C.1.3 must be 2 when premarket receiver and FDA.C.5.5b present with study type 1/2/3"
								.to_string(),
							line: None,
							column: None,
						});
					}
				}
			}
		}
	}

	// FDA Rule: Pre-ANDA number required for IND-exempt BA/BE when report type is 2
	if is_fda
		&& type_of_report.as_deref() == Some("2")
		&& msg_receiver.as_deref() == Some("CDER_IND_EXEMPT_BA_BE")
		&& !has_pre_anda
	{
		errors.push(XmlValidationError {
			message: "FDA.C.5.5b required when C.1.3=2 and N.2.r.3=CDER_IND_EXEMPT_BA_BE"
				.to_string(),
			line: None,
			column: None,
		});
	}

	// FDA Rule: Pre-ANDA number must not appear for postmarket receivers
	if is_fda
		&& has_pre_anda
		&& batch_receiver.as_deref() == Some("ZZFDA")
		&& matches!(msg_receiver.as_deref(), Some("CDER") | Some("CBER"))
	{
		errors.push(XmlValidationError {
			message: "FDA.C.5.5b must not be provided for postmarket (N.1.4=ZZFDA, N.2.r.3=CDER/CBER)"
				.to_string(),
			line: None,
			column: None,
		});
	}

	// Rule: associatedPerson name fields empty require nullFlavor
	if let Ok(nodes) =
		xpath.findnodes("//hl7:associatedPerson//hl7:name/*", None)
	{
		for node in nodes {
			let content = node.get_content();
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if content.trim().is_empty() && !has_null_flavor {
				errors.push(XmlValidationError {
					message: format!(
						"associatedPerson name element '{}' is empty; nullFlavor is required",
						node.get_name()
					),
					line: None,
					column: None,
				});
			}
			if !content.trim().is_empty() && has_null_flavor {
				errors.push(XmlValidationError {
					message: format!(
						"associatedPerson name element '{}' has value and nullFlavor; nullFlavor must be absent when value present",
						node.get_name()
					),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: associatedPerson birthTime empty requires nullFlavor
	if let Ok(nodes) =
		xpath.findnodes("//hl7:associatedPerson//hl7:birthTime", None)
	{
		for node in nodes {
			let value = node.get_attribute("value");
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if value.as_deref().unwrap_or("").trim().is_empty()
				&& !has_null_flavor
			{
				errors.push(XmlValidationError {
					message:
						"associatedPerson birthTime missing value; nullFlavor is required"
							.to_string(),
					line: None,
					column: None,
				});
			}
			if value
				.as_deref()
				.map(|v| !v.trim().is_empty())
				.unwrap_or(false)
				&& has_null_flavor
			{
				errors.push(XmlValidationError {
					message:
						"associatedPerson birthTime has value and nullFlavor; nullFlavor must be absent when value present"
							.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: researchStudy/title empty requires nullFlavor
	if let Ok(nodes) =
		xpath.findnodes("//hl7:researchStudy/hl7:title", None)
	{
		for node in nodes {
			let content = node.get_content();
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if content.trim().is_empty() && !has_null_flavor {
				errors.push(XmlValidationError {
					message: "researchStudy/title is empty; nullFlavor is required"
						.to_string(),
					line: None,
					column: None,
				});
			}
			if !content.trim().is_empty() && has_null_flavor {
				errors.push(XmlValidationError {
					message:
						"researchStudy/title has value and nullFlavor; nullFlavor must be absent when value present"
							.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: adverseEventAssessment id missing extension requires nullFlavor
	if let Ok(nodes) =
		xpath.findnodes("//hl7:adverseEventAssessment/hl7:id", None)
	{
		for node in nodes {
			let extension = node.get_attribute("extension");
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if extension.as_deref().unwrap_or("").trim().is_empty()
				&& !has_null_flavor
			{
				errors.push(XmlValidationError {
					message:
						"adverseEventAssessment/id missing extension; nullFlavor is required"
							.to_string(),
					line: None,
					column: None,
				});
			}
			if extension
				.as_deref()
				.map(|v| !v.trim().is_empty())
				.unwrap_or(false)
				&& has_null_flavor
			{
				errors.push(XmlValidationError {
					message:
						"adverseEventAssessment/id has extension and nullFlavor; nullFlavor must be absent when value present"
							.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: low/high without value must include nullFlavor
	if let Ok(nodes) =
		xpath.findnodes("//hl7:low | //hl7:high", None)
	{
		for node in nodes {
			let value = node.get_attribute("value");
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if value.as_deref().unwrap_or("").trim().is_empty()
				&& !has_null_flavor
			{
				errors.push(XmlValidationError {
					message: format!(
						"{} missing value; nullFlavor is required",
						node.get_name()
					),
					line: None,
					column: None,
				});
			}
			if !value.as_deref().unwrap_or("").trim().is_empty()
				&& has_null_flavor
			{
				errors.push(XmlValidationError {
					message: format!(
						"{} has value and nullFlavor; nullFlavor must be absent when value present",
						node.get_name()
					),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: reaction effectiveTime low/high require value or nullFlavor
	if let Ok(nodes) = xpath.findnodes(
		"//hl7:observation[hl7:code[@code='29']]/hl7:effectiveTime/hl7:low | //hl7:observation[hl7:code[@code='29']]/hl7:effectiveTime/hl7:high",
		None,
	) {
		for node in nodes {
			let value = node.get_attribute("value");
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if value.as_deref().unwrap_or("").trim().is_empty()
				&& !has_null_flavor
			{
				errors.push(XmlValidationError {
					message: "reaction effectiveTime low/high missing value; nullFlavor is required"
						.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: drug effectiveTime low/high require value or nullFlavor
	if let Ok(nodes) = xpath.findnodes(
		"//hl7:substanceAdministration/hl7:effectiveTime//hl7:low | //hl7:substanceAdministration/hl7:effectiveTime//hl7:high",
		None,
	) {
		for node in nodes {
			let value = node.get_attribute("value");
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if value.as_deref().unwrap_or("").trim().is_empty()
				&& !has_null_flavor
			{
				errors.push(XmlValidationError {
					message: "drug effectiveTime low/high missing value; nullFlavor is required"
						.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: patient effectiveTime low/high require value or nullFlavor
	if let Ok(nodes) = xpath.findnodes(
		"//hl7:primaryRole//hl7:effectiveTime//hl7:low | //hl7:primaryRole//hl7:effectiveTime//hl7:high",
		None,
	) {
		for node in nodes {
			let value = node.get_attribute("value");
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if value.as_deref().unwrap_or("").trim().is_empty()
				&& !has_null_flavor
			{
				errors.push(XmlValidationError {
					message: "patient effectiveTime low/high missing value; nullFlavor is required"
						.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: BL values missing value must include nullFlavor
	if let Ok(nodes) = xpath.findnodes(
		"//hl7:value[@xsi:type='BL']",
		None,
	) {
		for node in nodes {
			let value = node.get_attribute("value");
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if value.as_deref().unwrap_or("").trim().is_empty()
				&& !has_null_flavor
			{
				errors.push(XmlValidationError {
					message:
						"BL value missing value; nullFlavor is required"
							.to_string(),
					line: None,
					column: None,
				});
			}
			if !value.as_deref().unwrap_or("").trim().is_empty()
				&& has_null_flavor
			{
				errors.push(XmlValidationError {
					message:
						"BL value has value and nullFlavor; nullFlavor must be absent when value present"
							.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: code missing code attribute must include nullFlavor unless originalText or codeSystem is present
	if let Ok(nodes) = xpath.findnodes("//hl7:code", None) {
		for node in nodes {
			let code = node.get_attribute("code");
			let code_system = node.get_attribute("codeSystem");
			let has_original_text = node
				.get_child_elements()
				.iter()
				.any(|c| c.get_name() == "originalText" && !c.get_content().trim().is_empty());
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if code.as_deref().unwrap_or("").trim().is_empty()
				&& code_system.as_deref().unwrap_or("").trim().is_empty()
				&& !has_null_flavor
				&& !has_original_text
			{
				errors.push(XmlValidationError {
					message: "code missing code/codeSystem; nullFlavor is required when originalText is absent"
						.to_string(),
					line: None,
					column: None,
				});
			}
			if !code.as_deref().unwrap_or("").trim().is_empty()
				&& has_null_flavor
			{
				errors.push(XmlValidationError {
					message:
						"code has value and nullFlavor; nullFlavor must be absent when value present"
							.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: reaction investigation characteristic BL values missing value must include nullFlavor
	if let Ok(nodes) = xpath.findnodes(
		"//hl7:investigationCharacteristic/hl7:value[@xsi:type='BL']",
		None,
	) {
		for node in nodes {
			let value = node.get_attribute("value");
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if value.as_deref().unwrap_or("").trim().is_empty()
				&& !has_null_flavor
			{
				errors.push(XmlValidationError {
					message: "investigationCharacteristic BL missing value; nullFlavor is required"
						.to_string(),
					line: None,
					column: None,
				});
			}
			if !value.as_deref().unwrap_or("").trim().is_empty()
				&& has_null_flavor
			{
				errors.push(XmlValidationError {
					message: "investigationCharacteristic BL has value and nullFlavor; nullFlavor must be absent when value present"
						.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: reaction report linkage code nullFlavor when missing
	if let Ok(nodes) = xpath.findnodes(
		"//hl7:outboundRelationship[@typeCode='SPRT']/hl7:relatedInvestigation/hl7:code",
		None,
	) {
		for node in nodes {
			let code = node.get_attribute("code");
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if code.as_deref().unwrap_or("").trim().is_empty()
				&& !has_null_flavor
			{
				errors.push(XmlValidationError {
					message: "relatedInvestigation/code missing code; nullFlavor is required"
						.to_string(),
					line: None,
					column: None,
				});
			}
			if !code.as_deref().unwrap_or("").trim().is_empty()
				&& has_null_flavor
			{
				errors.push(XmlValidationError {
					message: "relatedInvestigation/code has value and nullFlavor; nullFlavor must be absent when value present"
						.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: reaction outcome value nullFlavor when missing
	if let Ok(nodes) = xpath.findnodes(
		"//hl7:observation[hl7:code[@code='27']]/hl7:value",
		None,
	) {
		for node in nodes {
			let code = node.get_attribute("code");
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if code.as_deref().unwrap_or("").trim().is_empty()
				&& !has_null_flavor
			{
				errors.push(XmlValidationError {
					message: "reaction outcome value missing code; nullFlavor is required"
						.to_string(),
					line: None,
					column: None,
				});
			}
			if !code.as_deref().unwrap_or("").trim().is_empty()
				&& has_null_flavor
			{
				errors.push(XmlValidationError {
					message: "reaction outcome value has value and nullFlavor; nullFlavor must be absent when value present"
						.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: reaction term (E.i.2) must have code or nullFlavor
	if let Ok(nodes) = xpath.findnodes(
		"//hl7:observation[hl7:code[@code='29']]/hl7:value",
		None,
	) {
		for node in nodes {
			let code = node.get_attribute("code");
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if code.as_deref().unwrap_or("").trim().is_empty()
				&& !has_null_flavor
			{
				errors.push(XmlValidationError {
					message:
						"reaction term missing code; nullFlavor is required"
							.to_string(),
					line: None,
					column: None,
				});
			}
			if !code.as_deref().unwrap_or("").trim().is_empty()
				&& has_null_flavor
			{
				errors.push(XmlValidationError {
					message:
						"reaction term has code and nullFlavor; nullFlavor must be absent when value present"
							.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: reaction translation (E.i.1.2) ED must have content or nullFlavor
	if let Ok(nodes) = xpath.findnodes(
		"//hl7:observation[hl7:code[@code='30']]/hl7:value[@xsi:type='ED']",
		None,
	) {
		for node in nodes {
			let content = node.get_content();
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if content.trim().is_empty() && !has_null_flavor {
				errors.push(XmlValidationError {
					message:
						"reaction translation missing value; nullFlavor is required"
							.to_string(),
					line: None,
					column: None,
				});
			}
			if !content.trim().is_empty() && has_null_flavor {
				errors.push(XmlValidationError {
					message:
						"reaction translation has value and nullFlavor; nullFlavor must be absent when value present"
							.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: reaction country code must have code or nullFlavor
	if let Ok(nodes) = xpath.findnodes(
		"//hl7:locatedPlace/hl7:code",
		None,
	) {
		for node in nodes {
			let code = node.get_attribute("code");
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if code.as_deref().unwrap_or("").trim().is_empty()
				&& !has_null_flavor
			{
				errors.push(XmlValidationError {
					message:
						"reaction country missing code; nullFlavor is required"
							.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: effectiveTime width must include low or high when present
	if let Ok(nodes) = xpath.findnodes("//hl7:effectiveTime", None) {
		for node in nodes {
			let children = node.get_child_elements();
			let mut has_low = false;
			let mut has_high = false;
			let mut has_width = false;
			for child in children {
				let name = child.get_name();
				match name.as_str() {
					"low" => has_low = true,
					"high" => has_high = true,
					"width" => has_width = true,
					_ => {}
				}
			}
			if has_width && !has_low && !has_high {
				errors.push(XmlValidationError {
					message:
						"effectiveTime has width but missing low/high".to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: start/end/duration combos for reaction event (E.i.4/E.i.5/E.i.6)
	if let Ok(nodes) = xpath.findnodes(
		"//hl7:observation[hl7:id and hl7:code[@code='29'] and hl7:code[@codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]",
		None,
	) {
		for node in nodes {
			let mut has_start = false;
			let mut has_end = false;
			let mut has_duration = false;
			for child in node.get_child_elements() {
				match child.get_name().as_str() {
					"effectiveTime" => {
						let value = child.get_attribute("value");
						let children = child.get_child_elements();
						if value.is_some() {
							// treat as date/time (start or end depending on code)
							has_start = true;
						} else {
							for sub in children {
								let name = sub.get_name();
								if name == "low" {
									has_start = true;
								} else if name == "high" {
									has_end = true;
								} else if name == "width" {
									has_duration = true;
								} else if name == "comp" {
									for comp_child in sub.get_child_elements() {
										let cname = comp_child.get_name();
										if cname == "low" {
											has_start = true;
										} else if cname == "high" {
											has_end = true;
										} else if cname == "width" {
											has_duration = true;
										}
									}
								}
							}
						}
					}
					"value" => {
						// duration often encoded as PQ value
						if child.get_attribute("value").is_some() {
							has_duration = true;
						}
					}
					_ => {}
				}
			}

			if !has_start && !has_end && !has_duration {
				errors.push(XmlValidationError {
					message:
						"Reaction requires start, end, or duration".to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: drug start/end/duration combos for dosage (G.k.4.r.4/5/8)
	if let Ok(nodes) = xpath.findnodes(
		"//hl7:substanceAdministration/hl7:effectiveTime[@xsi:type='SXPR_TS' or @xsi:type='IVL_TS']",
		None,
	) {
		for node in nodes {
			let mut has_start = false;
			let mut has_end = false;
			let mut has_duration = false;
			for child in node.get_child_elements() {
				let name = child.get_name();
				if name == "low" {
					has_start = true;
				} else if name == "high" {
					has_end = true;
				} else if name == "width" {
					has_duration = true;
				} else if name == "comp" {
					for comp_child in child.get_child_elements() {
						let cname = comp_child.get_name();
						if cname == "low" {
							has_start = true;
						} else if cname == "high" {
							has_end = true;
						} else if cname == "width" {
							has_duration = true;
						}
					}
				}
			}
			if !has_start && !has_end && !has_duration {
				errors.push(XmlValidationError {
					message:
						"Drug requires start, end, or duration".to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: SXPR_TS must have at least one comp (PIVL_TS or IVL_TS)
	if let Ok(nodes) = xpath.findnodes(
		"//hl7:effectiveTime[@xsi:type='SXPR_TS']",
		None,
	) {
		for node in nodes {
			let comps = node.get_child_elements();
			let mut has_comp = false;
			for comp in comps {
				if comp.get_name() == "comp" {
					has_comp = true;
				}
			}
			if !has_comp {
				errors.push(XmlValidationError {
					message: "SXPR_TS must include comp elements".to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: PIVL_TS must include period with value/unit
	if let Ok(nodes) =
		xpath.findnodes("//hl7:comp[@xsi:type='PIVL_TS']", None)
	{
		for node in nodes {
			let mut has_period = false;
			for child in node.get_child_elements() {
				if child.get_name() == "period" {
					has_period = true;
					let value = child.get_attribute("value");
					let unit = child.get_attribute("unit");
					if value.is_none() || unit.is_none() {
						errors.push(XmlValidationError {
							message:
								"PIVL_TS period must include value and unit"
									.to_string(),
							line: None,
							column: None,
						});
					}
				}
			}
			if !has_period {
				errors.push(XmlValidationError {
					message: "PIVL_TS must include period".to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: IVL_TS with operator='A' must include low/high or width
	if let Ok(nodes) =
		xpath.findnodes("//hl7:comp[@xsi:type='IVL_TS']", None)
	{
		for node in nodes {
			let operator = node.get_attribute("operator");
			if operator.as_deref() == Some("A") {
				let mut has_low = false;
				let mut has_high = false;
				let mut has_width = false;
				for child in node.get_child_elements() {
					let name = child.get_name();
					if name == "low" {
						has_low = true;
					} else if name == "high" {
						has_high = true;
					} else if name == "width" {
						has_width = true;
					}
				}
				if !has_low && !has_high && !has_width {
					errors.push(XmlValidationError {
						message: "IVL_TS operator='A' must include low, high, or width"
							.to_string(),
						line: None,
						column: None,
					});
				}
			}
		}
	}

	// Rule: test result values must be structurally valid
	if let Ok(nodes) =
		xpath.findnodes("//hl7:organizer[hl7:code[@code='3']]/hl7:component/hl7:observation/hl7:value", None)
	{
		for node in nodes {
			let xsi_type = node
				.get_attribute_ns("type", "http://www.w3.org/2001/XMLSchema-instance")
				.or_else(|| node.get_attribute("xsi:type"));
			match xsi_type.as_deref() {
				Some("IVL_PQ") => {
					let children = node.get_child_elements();
					let mut has_any = false;
					for child in children {
						let name = child.get_name();
						if name == "low" || name == "high" || name == "center" {
							has_any = true;
							let value = child.get_attribute("value");
							let unit = child.get_attribute("unit");
							let has_null_flavor = child.get_attribute("nullFlavor").is_some();
							if (value.is_none() || unit.is_none()) && !has_null_flavor {
								errors.push(XmlValidationError {
									message: format!(
										"IVL_PQ/{name} must include value and unit"
									),
									line: None,
									column: None,
								});
							}
						}
					}
					if !has_any {
						errors.push(XmlValidationError {
							message:
								"IVL_PQ must include low/high/center".to_string(),
							line: None,
							column: None,
						});
					}
				}
				Some("PQ") => {
					let value = node.get_attribute("value");
					let unit = node.get_attribute("unit");
					if value.is_none() || unit.is_none() {
						errors.push(XmlValidationError {
							message: "PQ must include value and unit".to_string(),
							line: None,
							column: None,
						});
					}
				}
				Some("ED") | Some("ST") | Some("BL") | Some("CE") | None => {}
				Some(other) => {
					errors.push(XmlValidationError {
						message: format!("Unsupported test result xsi:type '{other}'"),
						line: None,
						column: None,
					});
				}
			}
		}
	}

	// Rule: doseQuantity must include value/unit
	if let Ok(nodes) = xpath.findnodes("//hl7:doseQuantity", None) {
		for node in nodes {
			let value = node.get_attribute("value");
			let unit = node.get_attribute("unit");
			if value.is_none() || unit.is_none() {
				errors.push(XmlValidationError {
					message: "doseQuantity must include value and unit".to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: routeCode must have code or originalText or nullFlavor
	if let Ok(nodes) = xpath.findnodes("//hl7:routeCode", None) {
		for node in nodes {
			let code = node.get_attribute("code");
			let has_original_text = node
				.get_child_elements()
				.iter()
				.any(|c| c.get_name() == "originalText" && !c.get_content().trim().is_empty());
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if code.as_deref().unwrap_or("").trim().is_empty()
				&& !has_original_text
				&& !has_null_flavor
			{
				errors.push(XmlValidationError {
					message:
						"routeCode missing code; originalText or nullFlavor is required"
							.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: formCode must have code/codeSystem, originalText, or nullFlavor
	if let Ok(nodes) = xpath.findnodes("//hl7:formCode", None) {
		for node in nodes {
			let has_code = node
				.get_attribute("code")
				.as_deref()
				.map(|v| !v.trim().is_empty())
				.unwrap_or(false);
			let has_code_system = node
				.get_attribute("codeSystem")
				.as_deref()
				.map(|v| !v.trim().is_empty())
				.unwrap_or(false);
			let has_original_text = node
				.get_child_elements()
				.iter()
				.any(|c| c.get_name() == "originalText" && !c.get_content().trim().is_empty());
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if !has_code && !has_code_system && !has_original_text && !has_null_flavor {
				errors.push(XmlValidationError {
					message:
						"formCode missing code/codeSystem/originalText; nullFlavor is required"
							.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: administrativeGenderCode must have code or nullFlavor
	if let Ok(nodes) = xpath.findnodes("//hl7:administrativeGenderCode", None) {
		for node in nodes {
			let code = node.get_attribute("code");
			let has_null_flavor = node.get_attribute("nullFlavor").is_some();
			if code.as_deref().unwrap_or("").trim().is_empty()
				&& !has_null_flavor
			{
				errors.push(XmlValidationError {
					message:
						"administrativeGenderCode missing code; nullFlavor is required"
							.to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: period must include value/unit
	if let Ok(nodes) = xpath.findnodes("//hl7:period", None) {
		for node in nodes {
			let value = node.get_attribute("value");
			let unit = node.get_attribute("unit");
			if value.is_none() || unit.is_none() {
				errors.push(XmlValidationError {
					message: "period must include value and unit".to_string(),
					line: None,
					column: None,
				});
			}
		}
	}

	// Rule: MedDRA codes must be 8 digits and include codeSystemVersion
	if let Ok(nodes) = xpath.findnodes(
		"//hl7:code[@codeSystem='2.16.840.1.113883.6.163'] | //hl7:value[@codeSystem='2.16.840.1.113883.6.163']",
		None,
	) {
		for node in nodes {
			let code = node.get_attribute("code");
			if let Some(code) = code.as_deref() {
				if !is_digits_len(code, 8) {
					errors.push(XmlValidationError {
						message: format!("MedDRA code must be 8 digits, got '{code}'"),
						line: None,
						column: None,
					});
				}
				let version = node.get_attribute("codeSystemVersion");
				if version.as_deref().unwrap_or("").trim().is_empty() {
					errors.push(XmlValidationError {
						message: "MedDRA code missing codeSystemVersion".to_string(),
						line: None,
						column: None,
					});
				}
			}
		}
	}

	// Rule: ISO country codes must be 2 letters
	if let Ok(nodes) = xpath.findnodes(
		"//hl7:code[@codeSystem='1.0.3166.1.2.2']",
		None,
	) {
		for node in nodes {
			let code = node.get_attribute("code");
			if let Some(code) = code.as_deref() {
				if !is_alpha_len(code, 2) {
					errors.push(XmlValidationError {
						message: format!(
							"ISO country code must be 2 letters, got '{code}'"
						),
						line: None,
						column: None,
					});
				}
			}
		}
	}

	collect_placeholder_errors(&root, &mut errors);
	collect_case_identifier_errors(&mut xpath, &mut errors);
	collect_medical_history_errors(&mut xpath, &mut errors);

	Ok(errors)
}

fn collect_placeholder_errors(root: &libxml::tree::Node, errors: &mut Vec<XmlValidationError>) {
	if root.get_type() == Some(libxml::tree::NodeType::ElementNode) {
		let content = root.get_content();
		if looks_placeholder(content.trim()) {
			errors.push(XmlValidationError {
				message: format!(
					"Placeholder value not allowed in <{}>: '{}'",
					root.get_name(),
					content.trim()
				),
				line: None,
				column: None,
			});
		}
		for (name, val) in root.get_attributes() {
			if looks_placeholder(val.trim()) {
				errors.push(XmlValidationError {
					message: format!(
						"Placeholder value not allowed for <{}> attribute {}='{}'",
						root.get_name(),
						name,
						val.trim()
					),
					line: None,
					column: None,
				});
			}
		}
	}

	for child in root.get_child_nodes() {
		collect_placeholder_errors(&child, errors);
	}
}

fn collect_case_identifier_errors(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	let other_case_path =
		"//hl7:investigationCharacteristic[hl7:code[@code='2' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.23']]/hl7:value";
	let has_true = xpath
		.findnodes(other_case_path, None)
		.ok()
		.map(|nodes| {
			nodes.into_iter().any(|n| {
				n.get_attribute("value")
					.map(|v| v == "true" || v == "1")
					.unwrap_or(false)
			})
		})
		.unwrap_or(false);
	if !has_true {
		return;
	}
	let ids_path =
		"//hl7:investigationEvent/hl7:subjectOf1/hl7:controlActEvent/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.3']";
	let has_ids = xpath
		.findnodes(ids_path, None)
		.ok()
		.map(|nodes| {
			nodes.into_iter().any(|n| {
				n.get_attribute("assigningAuthorityName")
					.map(|v| !v.trim().is_empty())
					.unwrap_or(false)
					&& n
						.get_attribute("extension")
						.map(|v| !v.trim().is_empty())
						.unwrap_or(false)
			})
		})
		.unwrap_or(false);
	if !has_ids {
		errors.push(XmlValidationError {
			message: "C.1.9.1 is true but C.1.9.1.r.1/.r.2 are missing".to_string(),
			line: None,
			column: None,
		});
	}
}

fn collect_medical_history_errors(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	let coded_path = "//hl7:organizer[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.20']]/hl7:component/hl7:observation[hl7:code[@code!='18']]";
	let has_coded = xpath
		.findnodes(coded_path, None)
		.ok()
		.map(|nodes| !nodes.is_empty())
		.unwrap_or(false);
	if has_coded {
		return;
	}
	let text_path = "//hl7:organizer[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.20']]/hl7:component/hl7:observation[hl7:code[@code='18']]/hl7:value";
	let has_text = xpath
		.findnodes(text_path, None)
		.ok()
		.map(|nodes| {
			nodes.into_iter().any(|n| {
				let content = n.get_content();
				!content.trim().is_empty() && !looks_placeholder(content.trim())
			})
		})
		.unwrap_or(false);
	if !has_text {
		errors.push(XmlValidationError {
			message: "D.7.2 must be provided when D.7.1.r.1b is not provided".to_string(),
			line: None,
			column: None,
		});
	}
}

fn looks_placeholder(value: &str) -> bool {
	let v = value.trim();
	if v.is_empty() {
		return false;
	}
	if v.chars().any(|c| c.is_whitespace()) {
		return false;
	}
	if v.len() > 24 {
		return false;
	}
	let mut chars = v.chars();
	let Some(first) = chars.next() else {
		return false;
	};
	if !first.is_ascii_uppercase() {
		return false;
	}
	if !v.contains('.') {
		return false;
	}
	v.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}

fn is_digits_len(value: &str, len: usize) -> bool {
	value.len() == len && value.chars().all(|c| c.is_ascii_digit())
}

fn is_alpha_len(value: &str, len: usize) -> bool {
	value.len() == len && value.chars().all(|c| c.is_ascii_alphabetic())
}
