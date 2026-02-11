use crate::xml::types::XmlValidationError;
use crate::xml::validate::{
	export_normalization_spec_for_rule, RuleFacts,
};
use libxml::tree::Node;
use libxml::xpath::Context;

use super::xml_validation::{
	validate_condition_rule_violation,
	validate_attr_prefix_on_nodes,
	validate_normalized_code_format_on_nodes,
	push_rule_error, validate_attr_null_flavor_pair_on_nodes,
	validate_attr_or_null_flavor_required_on_nodes,
	validate_attr_or_text_or_null_required_on_nodes,
	validate_code_or_codesystem_or_text_required_with_nullflavor_forbidden_on_nodes,
	validate_code_or_codesystem_or_text_or_null_required_on_nodes,
	validate_required_attrs_on_nodes, validate_required_child_on_nodes,
	validate_supported_xsi_types_on_nodes,
	validate_typed_children_attrs_or_nullflavor_on_nodes,
	validate_when_attr_equals_require_any_children,
	validate_when_child_present_require_any_children,
	validate_text_null_flavor_pair_on_nodes,
	xpath_any_node, xpath_has_nodes,
};

pub(crate) fn collect_ich_identity_text_errors(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	validate_attr_prefix_on_nodes(
		xpath,
		errors,
		"//hl7:telecom",
		"value",
		&["tel:", "fax:", "mailto:"],
		"ICH.XML.TELECOM.FORMAT.REQUIRED",
		"telecom value",
	);

	validate_attr_null_flavor_pair_on_nodes(
		xpath,
		errors,
		"//hl7:telecom",
		"value",
		"ICH.XML.TELECOM.NULLFLAVOR.REQUIRED",
		"telecom missing value; nullFlavor is required",
		Some("ICH.XML.TELECOM.NULLFLAVOR.FORBIDDEN"),
		Some(
			"telecom has value and nullFlavor; nullFlavor must be absent when value present",
		),
	);

	validate_text_null_flavor_pair_on_nodes(
		xpath,
		errors,
		"//hl7:ingredientSubstance/hl7:name",
		"ICH.G.k.2.3.NAME.NULLFLAVOR.REQUIRED",
		"ingredientSubstance/name is empty; nullFlavor is required",
		Some("ICH.G.k.2.3.NAME.NULLFLAVOR.FORBIDDEN"),
		Some(
			"ingredientSubstance/name has value and nullFlavor; nullFlavor must be absent when value present",
		),
	);

	validate_text_null_flavor_pair_on_nodes(
		xpath,
		errors,
		"//hl7:primaryRole//hl7:name/*",
		"ICH.C.2.r.2.NAME.NULLFLAVOR.REQUIRED",
		"primaryRole name element is empty; nullFlavor is required",
		Some("ICH.C.2.r.2.NAME.NULLFLAVOR.FORBIDDEN"),
		Some(
			"primaryRole name element has value and nullFlavor; nullFlavor must be absent when value present",
		),
	);

	validate_text_null_flavor_pair_on_nodes(
		xpath,
		errors,
		"//hl7:representedOrganization/hl7:name",
		"ICH.C.2.r.3.ORG_NAME.NULLFLAVOR.REQUIRED",
		"representedOrganization/name is empty; nullFlavor is required",
		Some("ICH.C.2.r.3.ORG_NAME.NULLFLAVOR.FORBIDDEN"),
		Some(
			"representedOrganization/name has value and nullFlavor; nullFlavor must be absent when value present",
		),
	);

	validate_attr_null_flavor_pair_on_nodes(
		xpath,
		errors,
		"//hl7:primaryRole/hl7:id",
		"extension",
		"ICH.C.2.r.1.ID.NULLFLAVOR.REQUIRED",
		"primaryRole/id missing extension; nullFlavor is required",
		Some("ICH.C.2.r.1.ID.NULLFLAVOR.FORBIDDEN"),
		Some(
			"primaryRole/id has extension and nullFlavor; nullFlavor must be absent when value present",
		),
	);
	validate_attr_or_null_flavor_required_on_nodes(
		xpath,
		errors,
		"//hl7:primaryRole/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.6']",
		"extension",
		"ICH.C.2.r.1.ID.ROOT_3_6.NULLFLAVOR.REQUIRED",
		"primaryRole/id with root 2.16.840.1.113883.3.989.2.1.3.6 requires extension or nullFlavor",
	);

	validate_attr_null_flavor_pair_on_nodes(
		xpath,
		errors,
		"//hl7:primaryRole//hl7:birthTime",
		"value",
		"ICH.D.2.BIRTHTIME.NULLFLAVOR.REQUIRED",
		"birthTime missing value; nullFlavor is required",
		Some("ICH.D.2.BIRTHTIME.NULLFLAVOR.FORBIDDEN"),
		Some(
			"birthTime has value and nullFlavor; nullFlavor must be absent when value present",
		),
	);

	validate_text_null_flavor_pair_on_nodes(
		xpath,
		errors,
		"//hl7:text | //hl7:originalText",
		"ICH.XML.TEXT.NULLFLAVOR.REQUIRED",
		"text/originalText is empty; nullFlavor is required",
		Some("ICH.XML.TEXT.NULLFLAVOR.FORBIDDEN"),
		Some(
			"text/originalText has value and nullFlavor; nullFlavor must be absent when value present",
		),
	);
}

pub(crate) fn collect_ich_profile_value_presence_errors(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	validate_text_null_flavor_pair_on_nodes(
		xpath,
		errors,
		"//hl7:associatedPerson//hl7:name/*",
		"ICH.D.PARENT.NAME.NULLFLAVOR.REQUIRED",
		"associatedPerson name element is empty; nullFlavor is required",
		Some("ICH.D.PARENT.NAME.NULLFLAVOR.FORBIDDEN"),
		Some(
			"associatedPerson name element has value and nullFlavor; nullFlavor must be absent when value present",
		),
	);

	// Rule: associatedPerson birthTime empty requires nullFlavor
	validate_attr_null_flavor_pair_on_nodes(
		xpath,
		errors,
		"//hl7:associatedPerson//hl7:birthTime",
		"value",
		"ICH.D.PARENT.BIRTHTIME.NULLFLAVOR.REQUIRED",
		"associatedPerson birthTime missing value; nullFlavor is required",
		Some("ICH.D.PARENT.BIRTHTIME.NULLFLAVOR.FORBIDDEN"),
		Some(
			"associatedPerson birthTime has value and nullFlavor; nullFlavor must be absent when value present",
		),
	);

	// Rule: researchStudy/title empty requires nullFlavor
	validate_text_null_flavor_pair_on_nodes(
		xpath,
		errors,
		"//hl7:researchStudy/hl7:title",
		"ICH.C.5.TITLE.NULLFLAVOR.REQUIRED",
		"researchStudy/title is empty; nullFlavor is required",
		Some("ICH.C.5.TITLE.NULLFLAVOR.FORBIDDEN"),
		Some(
			"researchStudy/title has value and nullFlavor; nullFlavor must be absent when value present",
		),
	);

	// Rule: adverseEventAssessment id missing extension requires nullFlavor
	validate_attr_null_flavor_pair_on_nodes(
		xpath,
		errors,
		"//hl7:adverseEventAssessment/hl7:id",
		"extension",
		"ICH.G.k.9.i.2.ID.NULLFLAVOR.REQUIRED",
		"adverseEventAssessment/id missing extension; nullFlavor is required",
		Some("ICH.G.k.9.i.2.ID.NULLFLAVOR.FORBIDDEN"),
		Some(
			"adverseEventAssessment/id has extension and nullFlavor; nullFlavor must be absent when value present",
		),
	);

	// Rule: low/high without value must include nullFlavor
	validate_attr_null_flavor_pair_on_nodes(
		xpath,
		errors,
		"//hl7:low | //hl7:high",
		"value",
		"ICH.XML.LOW_HIGH.NULLFLAVOR.REQUIRED",
		"low/high missing value; nullFlavor is required",
		Some("ICH.XML.LOW_HIGH.NULLFLAVOR.FORBIDDEN"),
		Some(
			"low/high has value and nullFlavor; nullFlavor must be absent when value present",
		),
	);

	// Rule: reaction effectiveTime low/high require value or nullFlavor
	validate_attr_or_null_flavor_required_on_nodes(
		xpath,
		errors,
		"//hl7:observation[hl7:code[@code='29']]/hl7:effectiveTime/hl7:low | //hl7:observation[hl7:code[@code='29']]/hl7:effectiveTime/hl7:high",
		"value",
		"ICH.E.i.4-5.LOW_HIGH.NULLFLAVOR.REQUIRED",
		"reaction effectiveTime low/high missing value; nullFlavor is required",
	);

	// Rule: drug effectiveTime low/high require value or nullFlavor
	validate_attr_or_null_flavor_required_on_nodes(
		xpath,
		errors,
		"//hl7:substanceAdministration/hl7:effectiveTime//hl7:low | //hl7:substanceAdministration/hl7:effectiveTime//hl7:high",
		"value",
		"ICH.G.k.4.r.4-5.LOW_HIGH.NULLFLAVOR.REQUIRED",
		"drug effectiveTime low/high missing value; nullFlavor is required",
	);

	// Rule: patient effectiveTime low/high require value or nullFlavor
	validate_attr_or_null_flavor_required_on_nodes(
		xpath,
		errors,
		"//hl7:primaryRole//hl7:effectiveTime//hl7:low | //hl7:primaryRole//hl7:effectiveTime//hl7:high",
		"value",
		"ICH.D.EFFECTIVETIME.LOW_HIGH.NULLFLAVOR.REQUIRED",
		"patient effectiveTime low/high missing value; nullFlavor is required",
	);

	// Rule: BL values missing value must include nullFlavor
	validate_attr_null_flavor_pair_on_nodes(
		xpath,
		errors,
		"//hl7:value[@xsi:type='BL']",
		"value",
		"ICH.XML.BL.NULLFLAVOR.REQUIRED",
		"BL value missing value; nullFlavor is required",
		Some("ICH.XML.BL.NULLFLAVOR.FORBIDDEN"),
		Some(
			"BL value has value and nullFlavor; nullFlavor must be absent when value present",
		),
	);

	validate_code_or_codesystem_or_text_required_with_nullflavor_forbidden_on_nodes(
		xpath,
		errors,
		"//hl7:code",
		"ICH.XML.CODE.NULLFLAVOR.REQUIRED",
		"code missing code/codeSystem; nullFlavor is required when originalText is absent",
		"ICH.XML.CODE.NULLFLAVOR.FORBIDDEN",
		"code has value and nullFlavor; nullFlavor must be absent when value present",
	);

	// Rule: reaction investigation characteristic BL values missing value must include nullFlavor
	validate_attr_null_flavor_pair_on_nodes(
		xpath,
		errors,
		"//hl7:investigationCharacteristic/hl7:value[@xsi:type='BL']",
		"value",
		"ICH.XML.INV_CHAR_BL.NULLFLAVOR.REQUIRED",
		"investigationCharacteristic BL missing value; nullFlavor is required",
		Some("ICH.XML.INV_CHAR_BL.NULLFLAVOR.FORBIDDEN"),
		Some(
			"investigationCharacteristic BL has value and nullFlavor; nullFlavor must be absent when value present",
		),
	);

	// Rule: reaction report linkage code nullFlavor when missing
	validate_attr_null_flavor_pair_on_nodes(
		xpath,
		errors,
		"//hl7:outboundRelationship[@typeCode='SPRT']/hl7:relatedInvestigation/hl7:code",
		"code",
		"ICH.E.i.0.RELATIONSHIP.CODE.NULLFLAVOR.REQUIRED",
		"relatedInvestigation/code missing code; nullFlavor is required",
		Some("ICH.E.i.0.RELATIONSHIP.CODE.NULLFLAVOR.FORBIDDEN"),
		Some(
			"relatedInvestigation/code has value and nullFlavor; nullFlavor must be absent when value present",
		),
	);

	// Rule: reaction outcome value nullFlavor when missing
	validate_attr_null_flavor_pair_on_nodes(
		xpath,
		errors,
		"//hl7:observation[hl7:code[@code='27']]/hl7:value",
		"code",
		"ICH.E.i.7.NULLFLAVOR.REQUIRED",
		"reaction outcome value missing code; nullFlavor is required",
		Some("ICH.E.i.7.NULLFLAVOR.FORBIDDEN"),
		Some(
			"reaction outcome value has value and nullFlavor; nullFlavor must be absent when value present",
		),
	);

	// Rule: reaction term (E.i.2) must have code or nullFlavor
	validate_attr_null_flavor_pair_on_nodes(
		xpath,
		errors,
		"//hl7:observation[hl7:code[@code='29']]/hl7:value",
		"code",
		"ICH.E.i.2.NULLFLAVOR.REQUIRED",
		"reaction term missing code; nullFlavor is required",
		Some("ICH.E.i.2.NULLFLAVOR.FORBIDDEN"),
		Some(
			"reaction term has code and nullFlavor; nullFlavor must be absent when value present",
		),
	);

	// Rule: reaction translation (E.i.1.2) ED must have content or nullFlavor
	validate_text_null_flavor_pair_on_nodes(
		xpath,
		errors,
		"//hl7:observation[hl7:code[@code='30']]/hl7:value[@xsi:type='ED']",
		"ICH.E.i.1.2.NULLFLAVOR.REQUIRED",
		"reaction translation missing value; nullFlavor is required",
		Some("ICH.E.i.1.2.NULLFLAVOR.FORBIDDEN"),
		Some(
			"reaction translation has value and nullFlavor; nullFlavor must be absent when value present",
		),
	);

	// Rule: reaction country code must have code or nullFlavor
	validate_attr_or_null_flavor_required_on_nodes(
		xpath,
		errors,
		"//hl7:locatedPlace/hl7:code",
		"code",
		"ICH.E.i.9.COUNTRY.NULLFLAVOR.REQUIRED",
		"reaction country missing code; nullFlavor is required",
	);

	// Rule: routeCode must have code or originalText or nullFlavor
	validate_attr_or_text_or_null_required_on_nodes(
		xpath,
		errors,
		"//hl7:routeCode",
		"code",
		"ICH.G.k.4.r.11.NULLFLAVOR.REQUIRED",
		"routeCode missing code; originalText or nullFlavor is required",
	);

	// Rule: formCode must have code/codeSystem, originalText, or nullFlavor
	validate_code_or_codesystem_or_text_or_null_required_on_nodes(
		xpath,
		errors,
		"//hl7:formCode",
		"ICH.G.k.4.r.10.NULLFLAVOR.REQUIRED",
		"formCode missing code/codeSystem/originalText; nullFlavor is required",
	);

	// Rule: administrativeGenderCode must have code or nullFlavor
	validate_attr_or_null_flavor_required_on_nodes(
		xpath,
		errors,
		"//hl7:administrativeGenderCode",
		"code",
		"ICH.D.5.SEX.CONDITIONAL",
		"administrativeGenderCode missing code; nullFlavor is required",
	);
}

pub(crate) fn collect_ich_structural_value_errors(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	// Rule: effectiveTime width must include low or high when present
	validate_when_child_present_require_any_children(
		xpath,
		errors,
		"//hl7:effectiveTime",
		"width",
		&["low", "high"],
		"ICH.XML.EFFECTIVETIME.WIDTH.REQUIRES_BOUND",
		"effectiveTime has width but missing low/high",
	);

	// Rule: start/end/duration combos for reaction event (E.i.4/E.i.5/E.i.6)
	validate_temporal_markers_required_on_nodes(
		xpath,
		errors,
		"//hl7:observation[hl7:id and hl7:code[@code='29'] and hl7:code[@codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]",
		"ICH.E.i.4-6.CONDITIONAL",
		"Reaction requires start, end, or duration",
		reaction_temporal_markers,
	);

	// Rule: drug start/end/duration combos for dosage (G.k.4.r.4/5/8)
	validate_temporal_markers_required_on_nodes(
		xpath,
		errors,
		"//hl7:substanceAdministration/hl7:effectiveTime[@xsi:type='SXPR_TS' or @xsi:type='IVL_TS']",
		"ICH.G.k.4.r.4-8.CONDITIONAL",
		"Drug requires start, end, or duration",
		temporal_markers_from_children,
	);

	// Rule: SXPR_TS must have at least one comp (PIVL_TS or IVL_TS)
	validate_required_child_on_nodes(
		xpath,
		errors,
		"//hl7:effectiveTime[@xsi:type='SXPR_TS']",
		"comp",
		"ICH.XML.SXPR_TS.COMP.REQUIRED",
		"SXPR_TS must include comp elements",
	);

	// Rule: PIVL_TS must include period with value/unit
	validate_required_child_on_nodes(
		xpath,
		errors,
		"//hl7:comp[@xsi:type='PIVL_TS']",
		"period",
		"ICH.XML.PIVL_TS.PERIOD.REQUIRED",
		"PIVL_TS must include period",
	);
	validate_required_attrs_on_nodes(
		xpath,
		errors,
		"//hl7:comp[@xsi:type='PIVL_TS']/hl7:period",
		&["value", "unit"],
		"ICH.XML.PIVL_TS.PERIOD.VALUE_UNIT.REQUIRED",
		"PIVL_TS period must include value and unit",
	);

	// Rule: IVL_TS with operator='A' must include low/high or width
	validate_when_attr_equals_require_any_children(
		xpath,
		errors,
		"//hl7:comp[@xsi:type='IVL_TS']",
		"operator",
		"A",
		&["low", "high", "width"],
		"ICH.XML.IVL_TS.OPERATOR_A.BOUND_REQUIRED",
		"IVL_TS operator='A' must include low, high, or width",
	);

	// Rule: test result values must be structurally valid
	let test_result_value_xpath =
		"//hl7:organizer[hl7:code[@code='3']]/hl7:component/hl7:observation/hl7:value";
	validate_typed_children_attrs_or_nullflavor_on_nodes(
		xpath,
		errors,
		test_result_value_xpath,
		"IVL_PQ",
		&["low", "high", "center"],
		&["value", "unit"],
		"ICH.XML.TESTRESULT.IVL_PQ.COMPONENT.REQUIRED",
		"IVL_PQ must include low/high/center",
		"ICH.XML.TESTRESULT.IVL_PQ.VALUE_UNIT.REQUIRED",
		"IVL_PQ low/high/center must include value and unit",
	);
	validate_required_attrs_on_nodes(
		xpath,
		errors,
		"//hl7:organizer[hl7:code[@code='3']]/hl7:component/hl7:observation/hl7:value[@xsi:type='PQ']",
		&["value", "unit"],
		"ICH.XML.TESTRESULT.PQ.VALUE_UNIT.REQUIRED",
		"PQ must include value and unit",
	);
	validate_supported_xsi_types_on_nodes(
		xpath,
		errors,
		test_result_value_xpath,
		&["IVL_PQ", "PQ", "ED", "ST", "BL", "CE"],
		"ICH.XML.TESTRESULT.XSI_TYPE.UNSUPPORTED",
		"Unsupported test result xsi:type",
	);

	// Rule: doseQuantity/period must include value/unit
	validate_required_attrs_on_nodes(
		xpath,
		errors,
		"//hl7:doseQuantity",
		&["value", "unit"],
		"ICH.XML.DOSE_QUANTITY.VALUE_UNIT.REQUIRED",
		"doseQuantity must include value and unit",
	);
	validate_required_attrs_on_nodes(
		xpath,
		errors,
		"//hl7:period",
		&["value", "unit"],
		"ICH.XML.PERIOD.VALUE_UNIT.REQUIRED",
		"period must include value and unit",
	);

	for rule_code in [
		"ICH.XML.MEDDRA.CODE.FORMAT.REQUIRED",
		"ICH.XML.COUNTRY.CODE.FORMAT.REQUIRED",
	] {
		let Some(spec) = export_normalization_spec_for_rule(rule_code) else {
			continue;
		};
		let formatter = |code: &str| match rule_code {
			"ICH.XML.MEDDRA.CODE.FORMAT.REQUIRED" => {
				format!("MedDRA code must be 8 digits, got '{code}'")
			}
			"ICH.XML.COUNTRY.CODE.FORMAT.REQUIRED" => {
				format!("ISO country code must be 2 letters, got '{code}'")
			}
			_ => "Invalid coded value".to_string(),
		};
		let extra_required_attr = if rule_code == "ICH.XML.MEDDRA.CODE.FORMAT.REQUIRED"
		{
			Some((
				"codeSystemVersion",
				"ICH.XML.MEDDRA.VERSION.REQUIRED",
				"MedDRA code missing codeSystemVersion",
			))
		} else {
			None
		};
		validate_normalized_code_format_on_nodes(
			xpath,
			errors,
			rule_code,
			spec,
			formatter,
			extra_required_attr,
		);
	}
}

pub(crate) fn collect_ich_case_history_errors(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	// Rule: C.1.9.1 true requires linked prior case identifiers
	let has_true = xpath_any_node(
		xpath,
		"//hl7:investigationCharacteristic[hl7:code[@code='2' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.23']]/hl7:value",
		|n| {
			n.get_attribute("value")
				.map(|v| v == "true" || v == "1")
				.unwrap_or(false)
		},
	);
	if has_true {
		let has_ids = xpath_any_node(
			xpath,
			"//hl7:investigationEvent/hl7:subjectOf1/hl7:controlActEvent/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.3']",
			|n| {
				n.get_attribute("assigningAuthorityName")
					.map(|v| !v.trim().is_empty())
					.unwrap_or(false)
					&& n.get_attribute("extension")
						.map(|v| !v.trim().is_empty())
						.unwrap_or(false)
			},
		);
		validate_condition_rule_violation(
			errors,
			"ICH.C.1.9.1.CONDITIONAL",
			RuleFacts {
				ich_case_history_true_missing_prior_ids: Some(!has_ids),
				..RuleFacts::default()
			},
			"C.1.9.1 is true but C.1.9.1.r.1/.r.2 are missing",
		);
	}

	// Rule: D.7.2 required when coded medical history entry missing
	let has_coded = xpath_has_nodes(
		xpath,
		"//hl7:organizer[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.20']]/hl7:component/hl7:observation[hl7:code[@code!='18']]",
	);
	if has_coded {
		return;
	}
	let has_text = xpath_any_node(
		xpath,
		"//hl7:organizer[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.20']]/hl7:component/hl7:observation[hl7:code[@code='18']]/hl7:value",
		|n| {
			let content = n.get_content();
			!content.trim().is_empty() && !looks_placeholder(content.trim())
		},
	);
	validate_condition_rule_violation(
		errors,
		"ICH.D.7.2.CONDITIONAL",
		RuleFacts {
			ich_medical_history_missing_d72_text: Some(!has_text),
			..RuleFacts::default()
		},
		"D.7.2 must be provided when D.7.1.r.1b is not provided",
	);
}

fn reaction_temporal_markers(node: &Node) -> (bool, bool, bool) {
	let mut has_start = false;
	let mut has_end = false;
	let mut has_duration = false;

	for child in node.get_child_elements() {
		match child.get_name().as_str() {
			"effectiveTime" => {
				if child.get_attribute("value").is_some() {
					has_start = true;
				} else {
					let (s, e, d) = temporal_markers_from_children(&child);
					has_start |= s;
					has_end |= e;
					has_duration |= d;
				}
			}
			"value" => {
				if child.get_attribute("value").is_some() {
					has_duration = true;
				}
			}
			_ => {}
		}
	}

	(has_start, has_end, has_duration)
}

fn validate_temporal_markers_required_on_nodes(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	node_xpath: &str,
	rule_code: &str,
	message: &str,
	detector: fn(&Node) -> (bool, bool, bool),
) {
	if let Ok(nodes) = xpath.findnodes(node_xpath, None) {
		for node in nodes {
			let (has_start, has_end, has_duration) = detector(&node);
			if !has_start && !has_end && !has_duration {
				push_rule_error(errors, rule_code, message);
			}
		}
	}
}

fn temporal_markers_from_children(node: &Node) -> (bool, bool, bool) {
	let mut has_start = false;
	let mut has_end = false;
	let mut has_duration = false;

	for child in node.get_child_elements() {
		match child.get_name().as_str() {
			"low" => has_start = true,
			"high" => has_end = true,
			"width" => has_duration = true,
			"comp" => {
				let (s, e, d) = temporal_markers_from_children(&child);
				has_start |= s;
				has_end |= e;
				has_duration |= d;
			}
			_ => {}
		}
	}

	(has_start, has_end, has_duration)
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
	v.chars()
		.all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}
