use libxml::parser::Parser;
use libxml::xpath::Context;

use lib_core::xml::raw::patch::{patch_c_safety_report, CSafetyReportPatch};
use sqlx::types::time::Date;
use time::Month;

#[test]
fn patch_c_section_updates_values() {
	let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.parent()
		.and_then(|p| p.parent())
		.and_then(|p| p.parent())
		.expect("workspace root")
		.to_path_buf();
	let xml = std::fs::read(root.join("docs/refs/instances/FAERS2022Scenario1.xml"))
		.expect("read sample xml");
	let patch = CSafetyReportPatch {
		report_unique_id: "SR-TEST-123",
		transmission_date: Date::from_calendar_date(2024, Month::January, 15)
			.unwrap(),
		report_type: "1",
		date_first_received: Date::from_calendar_date(2024, Month::January, 10)
			.unwrap(),
		date_most_recent: Date::from_calendar_date(2024, Month::January, 15)
			.unwrap(),
		fulfil_expedited: true,
		worldwide_unique_id: Some("WW-TEST-999"),
		local_criteria_report_type: Some("1"),
		combination_product_indicator: Some("false"),
		nullification_code: None,
		nullification_reason: None,
	};

	let patched = patch_c_safety_report(&xml, &patch).expect("patch xml");
	let parser = Parser::default();
	let doc = parser.parse_string(&patched).expect("parse patched");
	let mut xpath = Context::new(&doc).expect("xpath");
	xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();

	let report_id = xpath
		.findvalue(
			"//hl7:investigationEvent/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.1']/@extension",
			None,
		)
		.unwrap();
	assert_eq!(report_id, "SR-TEST-123");

	let worldwide_id = xpath
		.findvalue(
			"//hl7:investigationEvent/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.2']/@extension",
			None,
		)
		.unwrap();
	assert_eq!(worldwide_id, "WW-TEST-999");
}
