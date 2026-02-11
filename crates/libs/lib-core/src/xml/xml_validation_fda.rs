use crate::xml::types::XmlValidationError;
use crate::xml::validate::{has_text, RuleFacts};
use libxml::xpath::Context;

use super::xml_validation::{
	validate_condition_rule_violation, validate_presence_rule,
	validate_value_rule_on_nodes, xpath_any_value_prefix, xpath_has_nodes,
};

#[derive(Debug, Clone, Default)]
struct FdaXmlFacts {
	batch_receiver: Option<String>,
	msg_receiver: Option<String>,
	combination_product_indicator: Option<String>,
	fulfil_expedited_criteria: Option<String>,
	pre_anda: Option<String>,
	study_type: Option<String>,
	type_of_report: Option<String>,
	has_primary_source: bool,
	has_primary_source_email: bool,
}

impl FdaXmlFacts {
	fn is_fda(&self) -> bool {
		matches!(
			self.batch_receiver.as_deref(),
			Some("ZZFDA") | Some("ZZFDA_PREMKT")
		) || matches!(
			self.msg_receiver.as_deref(),
			Some("CDER")
				| Some("CBER")
				| Some("CDER_IND")
				| Some("CBER_IND")
				| Some("CDER_IND_EXEMPT_BA_BE")
		)
	}

	fn has_batch_receiver(&self) -> bool {
		has_text(self.batch_receiver.as_deref())
	}

	fn has_pre_anda(&self) -> bool {
		has_text(self.pre_anda.as_deref())
	}

	fn type_of_report_is_two(&self) -> bool {
		self.type_of_report.as_deref() == Some("2")
	}

	fn msg_receiver_is_cder_ind_exempt_ba_be(&self) -> bool {
		self.msg_receiver.as_deref() == Some("CDER_IND_EXEMPT_BA_BE")
	}

	fn msg_receiver_is_cder_or_cber(&self) -> bool {
		matches!(self.msg_receiver.as_deref(), Some("CDER") | Some("CBER"))
	}

	fn msg_receiver_is_premarket(&self) -> bool {
		matches!(
			self.msg_receiver.as_deref(),
			Some("CDER_IND") | Some("CBER_IND") | Some("CDER_IND_EXEMPT_BA_BE")
		)
	}

	fn batch_receiver_is_zzfda(&self) -> bool {
		self.batch_receiver.as_deref() == Some("ZZFDA")
	}

	fn batch_receiver_is_zzfda_premarket(&self) -> bool {
		self.batch_receiver.as_deref() == Some("ZZFDA_PREMKT")
	}

	fn study_type_is_1_2_3(&self) -> bool {
		self.study_type
			.as_deref()
			.map(|v| v == "1" || v == "2" || v == "3")
			.unwrap_or(false)
	}

	fn combination_product_true(&self) -> bool {
		self.combination_product_indicator
			.as_deref()
			.map(|v| v.eq_ignore_ascii_case("true"))
			.unwrap_or(false)
	}

	fn fulfil_expedited_true(&self) -> bool {
		self.fulfil_expedited_criteria
			.as_deref()
			.map(|v| v.eq_ignore_ascii_case("true"))
			.unwrap_or(false)
	}
}

struct FdaValueNodeRule {
	xpath: &'static str,
	value_attr: &'static str,
	rule_code: &'static str,
	fallback_message: &'static str,
}

const FDA_STATIC_VALUE_NODE_RULES: &[FdaValueNodeRule] = &[
	FdaValueNodeRule {
		xpath: "//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.5.1.2.2.1.3']]/hl7:value",
		value_attr: "value",
		rule_code: "FDA.C.1.12.REQUIRED",
		fallback_message:
			"FDA.C.1.12 combination product indicator missing value; nullFlavor is required",
	},
	FdaValueNodeRule {
		xpath: "//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='2' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value",
		value_attr: "code",
		rule_code: "FDA.C.1.7.1.REQUIRED.MISSING_CODE",
		fallback_message:
			"FDA.C.1.7.1 local criteria report type missing code; nullFlavor is required",
	},
	FdaValueNodeRule {
		xpath: "//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='C17049' and @codeSystem='2.16.840.1.113883.3.26.1.1']]/hl7:value",
		value_attr: "code",
		rule_code: "FDA.D.11.REQUIRED",
		fallback_message: "FDA.D.11 patient race missing code; nullFlavor is required",
	},
	FdaValueNodeRule {
		xpath: "//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='C16564' and @codeSystem='2.16.840.1.113883.3.26.1.1']]/hl7:value",
		value_attr: "code",
		rule_code: "FDA.D.12.REQUIRED",
		fallback_message:
			"FDA.D.12 patient ethnicity missing code; nullFlavor is required",
	},
	FdaValueNodeRule {
		xpath: "//hl7:observation[hl7:code[@code='29' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]//hl7:outboundRelationship2/hl7:observation[hl7:code[@code='726' and @codeSystem='2.16.840.1.113883.3.989.5.1.2.2.1.32']]/hl7:value",
		value_attr: "value",
		rule_code: "FDA.E.i.3.2h.REQUIRED",
		fallback_message:
			"FDA.E.i.3.2h required intervention missing value; nullFlavor is required",
	},
];

pub(crate) fn collect_fda_profile_errors(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	let facts = collect_fda_xml_facts(xpath);
	if !facts.is_fda() {
		return;
	}

	validate_presence_rule(
		errors,
		"FDA.N.1.4.REQUIRED",
		facts.has_batch_receiver(),
		RuleFacts::default(),
		"FDA.N.1.4 batch receiver identifier missing",
	);

	for rule in FDA_STATIC_VALUE_NODE_RULES {
		validate_value_rule_on_nodes(
			xpath,
			errors,
			rule.xpath,
			rule.value_attr,
			rule.rule_code,
			RuleFacts::default(),
			rule.fallback_message,
		);
	}

	validate_value_rule_on_nodes(
		xpath,
		errors,
		"//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='2' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value",
		"code",
		"FDA.C.1.7.1.REQUIRED",
		RuleFacts {
			fda_combination_product_true: Some(facts.combination_product_true()),
			fda_fulfil_expedited_criteria: Some(facts.fulfil_expedited_true()),
			..RuleFacts::default()
		},
		"FDA.C.1.7.1 local criteria report type is invalid for current expedited/combination product facts",
	);

	let gk10a_rule_facts = RuleFacts {
		fda_has_pre_anda: Some(facts.has_pre_anda()),
		..RuleFacts::default()
	};
	validate_presence_rule(
		errors,
		"FDA.G.k.10a.REQUIRED",
		xpath_has_nodes(
			xpath,
			"//hl7:organizer[hl7:code[@code='4' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.20']]/hl7:component/hl7:substanceAdministration/hl7:outboundRelationship2[@typeCode='REFR']/hl7:observation[hl7:code[@code='9']]/hl7:value",
		),
		gk10a_rule_facts,
		"FDA.G.k.10a missing: required when FDA.C.5.5b is present",
	);
	validate_value_rule_on_nodes(
		xpath,
		errors,
		"//hl7:organizer[hl7:code[@code='4' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.20']]/hl7:component/hl7:substanceAdministration/hl7:outboundRelationship2[@typeCode='REFR']/hl7:observation[hl7:code[@code='9']]/hl7:value",
		"code",
		"FDA.G.k.10a.REQUIRED",
		gk10a_rule_facts,
		"FDA.G.k.10a must be code 1/2 or nullFlavor NA when FDA.C.5.5b is present",
	);

	validate_presence_rule(
		errors,
		"FDA.C.2.r.2.EMAIL.REQUIRED",
		facts.has_primary_source_email,
		RuleFacts {
			fda_primary_source_present: Some(facts.has_primary_source),
			..RuleFacts::default()
		},
		"FDA requires reporter email when primary source is present",
	);

	let report_type_rule_facts = RuleFacts {
		fda_batch_receiver_is_zzfda_premarket: Some(
			facts.batch_receiver_is_zzfda_premarket(),
		),
		fda_msg_receiver_is_premarket: Some(facts.msg_receiver_is_premarket()),
		fda_has_pre_anda: Some(facts.has_pre_anda()),
		fda_study_type_is_1_2_3: Some(facts.study_type_is_1_2_3()),
		..RuleFacts::default()
	};
	validate_value_rule_on_nodes(
		xpath,
		errors,
		"//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.23']]/hl7:value",
		"code",
		"ICH.C.1.3.CONDITIONAL",
		report_type_rule_facts,
		"C.1.3 must be 2 when premarket receiver and FDA.C.5.5b present with study type 1/2/3",
	);

	validate_condition_rule_violation(
		errors,
		"FDA.C.5.5b.REQUIRED",
		RuleFacts {
			fda_type_of_report_is_two: Some(facts.type_of_report_is_two()),
			fda_msg_receiver_is_cder_ind_exempt_ba_be: Some(
				facts.msg_receiver_is_cder_ind_exempt_ba_be(),
			),
			fda_has_pre_anda: Some(facts.has_pre_anda()),
			..RuleFacts::default()
		},
		"FDA.C.5.5b required when C.1.3=2 and N.2.r.3=CDER_IND_EXEMPT_BA_BE",
	);

	validate_condition_rule_violation(
		errors,
		"FDA.C.5.5b.FORBIDDEN",
		RuleFacts {
			fda_has_pre_anda: Some(facts.has_pre_anda()),
			fda_batch_receiver_is_zzfda: Some(facts.batch_receiver_is_zzfda()),
			fda_msg_receiver_is_cder_or_cber: Some(
				facts.msg_receiver_is_cder_or_cber(),
			),
			..RuleFacts::default()
		},
		"FDA.C.5.5b must not be provided for postmarket (N.1.4=ZZFDA, N.2.r.3=CDER/CBER)",
	);
}

fn collect_fda_xml_facts(xpath: &mut Context) -> FdaXmlFacts {
	FdaXmlFacts {
		batch_receiver: first_xpath_value(
			xpath,
			"/hl7:MCCI_IN200100UV01/hl7:receiver/hl7:device/hl7:id/@extension",
		),
		msg_receiver: first_xpath_value(
			xpath,
			"/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV/hl7:receiver/hl7:device/hl7:id/@extension",
		),
		combination_product_indicator: first_xpath_value(
			xpath,
			"//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.5.1.2.2.1.3']]/hl7:value/@value",
		),
		fulfil_expedited_criteria: first_xpath_value(
			xpath,
			"//hl7:component/hl7:observationEvent[hl7:code[@code='23' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value/@value",
		),
		pre_anda: first_xpath_value(
			xpath,
			"//hl7:researchStudy/hl7:authorization/hl7:studyRegistration/hl7:id[@root='2.16.840.1.113883.3.989.5.1.2.2.1.2.2']/@extension",
		),
		study_type: first_xpath_value(xpath, "//hl7:researchStudy/hl7:code/@code"),
		type_of_report: first_xpath_value(
			xpath,
			"//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.23']]/hl7:value/@code",
		),
		has_primary_source: xpath_has_nodes(
			xpath,
			"//hl7:outboundRelationship[@typeCode='SPRT']/hl7:relatedInvestigation/hl7:subjectOf2/hl7:controlActEvent/hl7:author/hl7:assignedEntity",
		),
		has_primary_source_email: xpath_any_value_prefix(
			xpath,
			"//hl7:outboundRelationship[@typeCode='SPRT']/hl7:relatedInvestigation/hl7:subjectOf2/hl7:controlActEvent/hl7:author/hl7:assignedEntity/hl7:telecom/@value",
			"mailto:",
		),
	}
}

fn first_xpath_value(xpath: &mut Context, expr: &str) -> Option<String> {
	xpath
		.findvalues(expr, None)
		.ok()
		.and_then(|vals| vals.first().cloned())
}
