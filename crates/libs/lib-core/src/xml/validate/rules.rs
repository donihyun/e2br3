use super::ValidationProfile;

#[derive(Debug, Clone, Copy)]
pub struct ValidationRuleMetadata {
	pub code: &'static str,
	pub profile: ValidationProfile,
	pub section: &'static str,
	pub blocking: bool,
	pub message: &'static str,
}

pub const VALIDATION_RULES: &[ValidationRuleMetadata] = &[
	// ICH core
	ValidationRuleMetadata {
		code: "ICH.C.1.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "case-identification",
		blocking: true,
		message: "Safety report identification is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.N.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "case-identification",
		blocking: true,
		message: "Message header is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.C.1.3.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "case-identification",
		blocking: false,
		message: "[C.1.3] is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.C.2.r.4.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "reporter",
		blocking: false,
		message: "[C.2.r.4] is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.D.1.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "patient",
		blocking: false,
		message: "[D.1] This Element is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.E.i.1.1a.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "reactions",
		blocking: false,
		message: "[E.i.1.1a] is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.E.i.7.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "reactions",
		blocking: false,
		message: "[E.i.7] is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.G.k.1.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "drugs",
		blocking: false,
		message: "[G.k.1] is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.G.k.2.2.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "drugs",
		blocking: false,
		message: "[G.k.2.2] is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.H.1.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "narrative",
		blocking: false,
		message: "[H.1] This Element is required.",
	},
	// FDA profile overlays
	ValidationRuleMetadata {
		code: "FDA.C.1.7.1.REQUIRED",
		profile: ValidationProfile::Fda,
		section: "case-identification",
		blocking: false,
		message: "FDA requires [C.1.7.1] when expedited criteria is fulfilled.",
	},
	ValidationRuleMetadata {
		code: "FDA.C.1.12.RECOMMENDED",
		profile: ValidationProfile::Fda,
		section: "case-identification",
		blocking: false,
		message: "FDA recommends [C.1.12] combination product report indicator.",
	},
	ValidationRuleMetadata {
		code: "FDA.C.2.r.2.EMAIL.REQUIRED",
		profile: ValidationProfile::Fda,
		section: "reporter",
		blocking: false,
		message: "FDA requires reporter email when primary source is present.",
	},
	ValidationRuleMetadata {
		code: "FDA.E.i.3.2h.REQUIRED",
		profile: ValidationProfile::Fda,
		section: "reactions",
		blocking: false,
		message:
			"FDA requires [E.i.3.2h] when other medically important condition is selected.",
	},
	// MFDS profile overlays
	ValidationRuleMetadata {
		code: "MFDS.C.3.1.KR.1.REQUIRED",
		profile: ValidationProfile::Mfds,
		section: "case-identification",
		blocking: true,
		message: "MFDS KR profile does not allow sender type 3.",
	},
	ValidationRuleMetadata {
		code: "MFDS.KR.DOMESTIC.PRODUCTCODE.REQUIRED",
		profile: ValidationProfile::Mfds,
		section: "drugs",
		blocking: true,
		message: "MFDS domestic cases require KR product coding for the drug.",
	},
	ValidationRuleMetadata {
		code: "MFDS.KR.FOREIGN.WHOMPID.RECOMMENDED",
		profile: ValidationProfile::Mfds,
		section: "drugs",
		blocking: false,
		message:
			"MFDS foreign-use products should provide WHO MPID/KR product coding.",
	},
	ValidationRuleMetadata {
		code: "MFDS.KR.DOMESTIC.INGREDIENTCODE.REQUIRED",
		profile: ValidationProfile::Mfds,
		section: "drugs",
		blocking: true,
		message:
			"MFDS domestic cases require KR ingredient coding for each active substance.",
	},
	ValidationRuleMetadata {
		code: "MFDS.G.k.9.i.2.r.2.KR.1.REQUIRED",
		profile: ValidationProfile::Mfds,
		section: "drugs",
		blocking: true,
		message:
			"MFDS requires KR method of assessment when source of assessment is present.",
	},
	ValidationRuleMetadata {
		code: "MFDS.G.k.9.i.2.r.3.KR.1.REQUIRED",
		profile: ValidationProfile::Mfds,
		section: "drugs",
		blocking: true,
		message:
			"MFDS requires KR result of assessment when source of assessment is present.",
	},
	ValidationRuleMetadata {
		code: "MFDS.G.k.9.i.2.r.1.REQUIRED",
		profile: ValidationProfile::Mfds,
		section: "drugs",
		blocking: true,
		message:
			"MFDS requires source of assessment when KR method/result values are provided.",
	},
	// XML-level coded checks
	ValidationRuleMetadata {
		code: "FDA.N.1.4.REQUIRED",
		profile: ValidationProfile::Fda,
		section: "xml",
		blocking: true,
		message: "FDA.N.1.4 batch receiver identifier missing.",
	},
	ValidationRuleMetadata {
		code: "FDA.C.1.7.1.REQUIRED",
		profile: ValidationProfile::Fda,
		section: "xml",
		blocking: true,
		message: "FDA.C.1.7.1 local criteria report type is required.",
	},
	ValidationRuleMetadata {
		code: "FDA.C.1.12.REQUIRED",
		profile: ValidationProfile::Fda,
		section: "xml",
		blocking: true,
		message: "FDA.C.1.12 combination product report indicator is required.",
	},
	ValidationRuleMetadata {
		code: "FDA.D.11.REQUIRED",
		profile: ValidationProfile::Fda,
		section: "xml",
		blocking: true,
		message: "FDA.D.11 patient race is required.",
	},
	ValidationRuleMetadata {
		code: "FDA.D.12.REQUIRED",
		profile: ValidationProfile::Fda,
		section: "xml",
		blocking: true,
		message: "FDA.D.12 patient ethnicity is required.",
	},
	ValidationRuleMetadata {
		code: "FDA.E.i.3.2h.REQUIRED",
		profile: ValidationProfile::Fda,
		section: "xml",
		blocking: true,
		message: "FDA.E.i.3.2h required intervention is required.",
	},
	ValidationRuleMetadata {
		code: "FDA.G.k.10a.REQUIRED",
		profile: ValidationProfile::Fda,
		section: "xml",
		blocking: true,
		message: "FDA.G.k.10a is required when FDA.C.5.5b is present.",
	},
	ValidationRuleMetadata {
		code: "FDA.C.5.5b.REQUIRED",
		profile: ValidationProfile::Fda,
		section: "xml",
		blocking: true,
		message:
			"FDA.C.5.5b is required when C.1.3=2 and N.2.r.3=CDER_IND_EXEMPT_BA_BE.",
	},
	ValidationRuleMetadata {
		code: "FDA.C.5.5b.FORBIDDEN",
		profile: ValidationProfile::Fda,
		section: "xml",
		blocking: true,
		message:
			"FDA.C.5.5b must not be provided for postmarket (N.1.4=ZZFDA, N.2.r.3=CDER/CBER).",
	},
	ValidationRuleMetadata {
		code: "ICH.C.1.3.CONDITIONAL",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message:
			"C.1.3 must be 2 when premarket receiver and FDA.C.5.5b are present with study type 1/2/3.",
	},
	ValidationRuleMetadata {
		code: "ICH.D.7.2.CONDITIONAL",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message: "D.7.2 must be provided when D.7.1.r.1b is not provided.",
	},
	ValidationRuleMetadata {
		code: "ICH.C.1.9.1.CONDITIONAL",
		profile: ValidationProfile::Ich,
		section: "case-identification",
		blocking: true,
		message: "C.1.9.1 is true but C.1.9.1.r.1/.r.2 are missing.",
	},
	ValidationRuleMetadata {
		code: "ICH.E.i.4-6.CONDITIONAL",
		profile: ValidationProfile::Ich,
		section: "reactions",
		blocking: true,
		message: "Reaction requires start, end, or duration.",
	},
	ValidationRuleMetadata {
		code: "ICH.G.k.4.r.4-8.CONDITIONAL",
		profile: ValidationProfile::Ich,
		section: "drugs",
		blocking: true,
		message: "Drug requires start, end, or duration.",
	},
	ValidationRuleMetadata {
		code: "ICH.XML.EFFECTIVETIME.WIDTH.REQUIRES_BOUND",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message: "effectiveTime with width must include low/high.",
	},
	ValidationRuleMetadata {
		code: "ICH.XML.SXPR_TS.COMP.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message: "SXPR_TS must include at least one comp element.",
	},
	ValidationRuleMetadata {
		code: "ICH.XML.PIVL_TS.PERIOD.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message: "PIVL_TS must include period.",
	},
	ValidationRuleMetadata {
		code: "ICH.XML.PIVL_TS.PERIOD.VALUE_UNIT.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message: "PIVL_TS period must include value and unit.",
	},
	ValidationRuleMetadata {
		code: "ICH.XML.IVL_TS.OPERATOR_A.BOUND_REQUIRED",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message: "IVL_TS operator='A' must include low, high, or width.",
	},
	ValidationRuleMetadata {
		code: "ICH.XML.DOSE_QUANTITY.VALUE_UNIT.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message: "doseQuantity must include value and unit.",
	},
	ValidationRuleMetadata {
		code: "ICH.XML.PERIOD.VALUE_UNIT.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message: "period must include value and unit.",
	},
	ValidationRuleMetadata {
		code: "ICH.D.5.SEX.CONDITIONAL",
		profile: ValidationProfile::Ich,
		section: "patient",
		blocking: true,
		message: "administrativeGenderCode missing code; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.XML.TELECOM.FORMAT.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message:
			"telecom value must start with tel:, fax:, or mailto:.",
	},
	ValidationRuleMetadata {
		code: "ICH.XML.TELECOM.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message: "telecom missing value; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.XML.TELECOM.NULLFLAVOR.FORBIDDEN",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message:
			"telecom has value and nullFlavor; nullFlavor must be absent when value present.",
	},
	ValidationRuleMetadata {
		code: "ICH.E.i.7.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "reactions",
		blocking: true,
		message: "reaction outcome value missing code; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.E.i.7.NULLFLAVOR.FORBIDDEN",
		profile: ValidationProfile::Ich,
		section: "reactions",
		blocking: true,
		message:
			"reaction outcome value has value and nullFlavor; nullFlavor must be absent when value present.",
	},
	ValidationRuleMetadata {
		code: "ICH.E.i.2.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "reactions",
		blocking: true,
		message: "reaction term missing code; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.E.i.2.NULLFLAVOR.FORBIDDEN",
		profile: ValidationProfile::Ich,
		section: "reactions",
		blocking: true,
		message:
			"reaction term has code and nullFlavor; nullFlavor must be absent when value present.",
	},
	ValidationRuleMetadata {
		code: "ICH.XML.CODE.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message:
			"code missing code/codeSystem; nullFlavor is required when originalText is absent.",
	},
	ValidationRuleMetadata {
		code: "ICH.XML.CODE.NULLFLAVOR.FORBIDDEN",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message:
			"code has value and nullFlavor; nullFlavor must be absent when value present.",
	},
	ValidationRuleMetadata {
		code: "ICH.G.k.4.r.11.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "drugs",
		blocking: true,
		message: "routeCode missing code; originalText or nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.G.k.4.r.10.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "drugs",
		blocking: true,
		message:
			"formCode missing code/codeSystem/originalText; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.XML.BL.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message: "BL value missing value; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.XML.BL.NULLFLAVOR.FORBIDDEN",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message:
			"BL value has value and nullFlavor; nullFlavor must be absent when value present.",
	},
	ValidationRuleMetadata {
		code: "ICH.XML.INV_CHAR_BL.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message:
			"investigationCharacteristic BL missing value; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.XML.INV_CHAR_BL.NULLFLAVOR.FORBIDDEN",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message:
			"investigationCharacteristic BL has value and nullFlavor; nullFlavor must be absent when value present.",
	},
	ValidationRuleMetadata {
		code: "ICH.E.i.0.RELATIONSHIP.CODE.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "reactions",
		blocking: true,
		message:
			"relatedInvestigation/code missing code; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.E.i.0.RELATIONSHIP.CODE.NULLFLAVOR.FORBIDDEN",
		profile: ValidationProfile::Ich,
		section: "reactions",
		blocking: true,
		message:
			"relatedInvestigation/code has value and nullFlavor; nullFlavor must be absent when value present.",
	},
	ValidationRuleMetadata {
		code: "ICH.E.i.9.COUNTRY.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "reactions",
		blocking: true,
		message: "reaction country missing code; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.G.k.2.3.NAME.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "drugs",
		blocking: true,
		message: "ingredientSubstance/name is empty; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.G.k.2.3.NAME.NULLFLAVOR.FORBIDDEN",
		profile: ValidationProfile::Ich,
		section: "drugs",
		blocking: true,
		message:
			"ingredientSubstance/name has value and nullFlavor; nullFlavor must be absent when value present.",
	},
	ValidationRuleMetadata {
		code: "ICH.C.2.r.2.NAME.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "reporter",
		blocking: true,
		message: "primaryRole name element is empty; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.C.2.r.2.NAME.NULLFLAVOR.FORBIDDEN",
		profile: ValidationProfile::Ich,
		section: "reporter",
		blocking: true,
		message:
			"primaryRole name element has value and nullFlavor; nullFlavor must be absent when value present.",
	},
	ValidationRuleMetadata {
		code: "ICH.C.2.r.3.ORG_NAME.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "reporter",
		blocking: true,
		message:
			"representedOrganization/name is empty; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.C.2.r.3.ORG_NAME.NULLFLAVOR.FORBIDDEN",
		profile: ValidationProfile::Ich,
		section: "reporter",
		blocking: true,
		message:
			"representedOrganization/name has value and nullFlavor; nullFlavor must be absent when value present.",
	},
	ValidationRuleMetadata {
		code: "ICH.C.2.r.1.ID.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "reporter",
		blocking: true,
		message: "primaryRole/id missing extension; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.C.2.r.1.ID.NULLFLAVOR.FORBIDDEN",
		profile: ValidationProfile::Ich,
		section: "reporter",
		blocking: true,
		message:
			"primaryRole/id has extension and nullFlavor; nullFlavor must be absent when value present.",
	},
	ValidationRuleMetadata {
		code: "ICH.D.2.BIRTHTIME.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "patient",
		blocking: true,
		message: "birthTime missing value; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.D.2.BIRTHTIME.NULLFLAVOR.FORBIDDEN",
		profile: ValidationProfile::Ich,
		section: "patient",
		blocking: true,
		message:
			"birthTime has value and nullFlavor; nullFlavor must be absent when value present.",
	},
	ValidationRuleMetadata {
		code: "ICH.D.PARENT.NAME.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "patient",
		blocking: true,
		message:
			"associatedPerson name element is empty; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.D.PARENT.NAME.NULLFLAVOR.FORBIDDEN",
		profile: ValidationProfile::Ich,
		section: "patient",
		blocking: true,
		message:
			"associatedPerson name element has value and nullFlavor; nullFlavor must be absent when value present.",
	},
	ValidationRuleMetadata {
		code: "ICH.D.PARENT.BIRTHTIME.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "patient",
		blocking: true,
		message:
			"associatedPerson birthTime missing value; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.D.PARENT.BIRTHTIME.NULLFLAVOR.FORBIDDEN",
		profile: ValidationProfile::Ich,
		section: "patient",
		blocking: true,
		message:
			"associatedPerson birthTime has value and nullFlavor; nullFlavor must be absent when value present.",
	},
	ValidationRuleMetadata {
		code: "ICH.C.5.TITLE.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "study",
		blocking: true,
		message: "researchStudy/title is empty; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.C.5.TITLE.NULLFLAVOR.FORBIDDEN",
		profile: ValidationProfile::Ich,
		section: "study",
		blocking: true,
		message:
			"researchStudy/title has value and nullFlavor; nullFlavor must be absent when value present.",
	},
	ValidationRuleMetadata {
		code: "ICH.G.k.9.i.2.ID.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "drugs",
		blocking: true,
		message:
			"adverseEventAssessment/id missing extension; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.G.k.9.i.2.ID.NULLFLAVOR.FORBIDDEN",
		profile: ValidationProfile::Ich,
		section: "drugs",
		blocking: true,
		message:
			"adverseEventAssessment/id has extension and nullFlavor; nullFlavor must be absent when value present.",
	},
	ValidationRuleMetadata {
		code: "ICH.XML.TEXT.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message: "text/originalText is empty; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.XML.TEXT.NULLFLAVOR.FORBIDDEN",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message:
			"text/originalText has value and nullFlavor; nullFlavor must be absent when value present.",
	},
	ValidationRuleMetadata {
		code: "ICH.XML.LOW_HIGH.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message: "low/high missing value; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.XML.LOW_HIGH.NULLFLAVOR.FORBIDDEN",
		profile: ValidationProfile::Ich,
		section: "xml",
		blocking: true,
		message:
			"low/high has value and nullFlavor; nullFlavor must be absent when value present.",
	},
	ValidationRuleMetadata {
		code: "ICH.E.i.4-5.LOW_HIGH.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "reactions",
		blocking: true,
		message:
			"reaction effectiveTime low/high missing value; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.G.k.4.r.4-5.LOW_HIGH.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "drugs",
		blocking: true,
		message:
			"drug effectiveTime low/high missing value; nullFlavor is required.",
	},
	ValidationRuleMetadata {
		code: "ICH.D.EFFECTIVETIME.LOW_HIGH.NULLFLAVOR.REQUIRED",
		profile: ValidationProfile::Ich,
		section: "patient",
		blocking: true,
		message:
			"patient effectiveTime low/high missing value; nullFlavor is required.",
	},
];

pub fn find_validation_rule(code: &str) -> Option<&'static ValidationRuleMetadata> {
	VALIDATION_RULES.iter().find(|rule| rule.code == code)
}

pub fn validation_rules_for_profile(
	profile: ValidationProfile,
) -> Vec<&'static ValidationRuleMetadata> {
	VALIDATION_RULES
		.iter()
		.filter(|rule| {
			matches!(rule.profile, ValidationProfile::Ich)
				|| rule.profile == profile
		})
		.collect()
}
