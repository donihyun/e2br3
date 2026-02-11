use lib_core::xml::validate::{
	find_canonical_rule, is_rule_condition_satisfied, is_rule_value_valid,
	normalize_drug_characterization, normalize_outcome_code, outcome_display_name,
	should_emit_required_intervention_null_flavor_ni, ExportDirective, RuleFacts,
};

#[test]
fn cross_profile_rule_source_parity_matrix() {
	// ICH: exporter default for E.i.7 must match catalog directive.
	let ich_outcome_rule = find_canonical_rule("ICH.E.i.7.REQUIRED")
		.expect("canonical ICH.E.i.7.REQUIRED rule");
	assert_eq!(
		ich_outcome_rule.export_directive,
		Some(ExportDirective::OutcomeDefaultCode3)
	);
	assert_eq!(normalize_outcome_code(None), "3");
	assert_eq!(
		outcome_display_name(normalize_outcome_code(None)),
		"not recovered/not resolved/ongoing"
	);

	// ICH: exporter default for G.k.1 must match catalog directive.
	let ich_drug_rule = find_canonical_rule("ICH.G.k.1.REQUIRED")
		.expect("canonical ICH.G.k.1.REQUIRED rule");
	assert_eq!(
		ich_drug_rule.export_directive,
		Some(ExportDirective::DrugRoleDefaultConcomitant)
	);
	assert_eq!(normalize_drug_characterization(""), "2");

	// FDA: validator condition + exporter directive should agree for E.i.3.2h.
	let fda_ei_rule = find_canonical_rule("FDA.E.i.3.2h.REQUIRED")
		.expect("canonical FDA.E.i.3.2h.REQUIRED rule");
	assert_eq!(
		fda_ei_rule.export_directive,
		Some(ExportDirective::RequiredInterventionNullFlavorNi)
	);
	assert!(is_rule_condition_satisfied(
		"FDA.E.i.3.2h.REQUIRED",
		RuleFacts {
			fda_reaction_other_medically_important: Some(true),
			..RuleFacts::default()
		}
	));
	assert!(should_emit_required_intervention_null_flavor_ni());

	// MFDS: validator condition + value checks come from the same canonical rule source.
	assert!(is_rule_condition_satisfied(
		"MFDS.KR.DOMESTIC.PRODUCTCODE.REQUIRED",
		RuleFacts {
			mfds_drug_domestic_kr: Some(true),
			..RuleFacts::default()
		}
	));
	assert!(!is_rule_value_valid(
		"MFDS.KR.DOMESTIC.PRODUCTCODE.REQUIRED",
		Some(""),
		None,
		RuleFacts::default()
	));
	assert!(is_rule_value_valid(
		"MFDS.KR.DOMESTIC.PRODUCTCODE.REQUIRED",
		Some("MFDS-MPID-1"),
		None,
		RuleFacts::default()
	));
}
