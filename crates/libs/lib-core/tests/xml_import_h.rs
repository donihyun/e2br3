use lib_core::xml::import_sections::h_narrative::parse_h_narrative;

#[test]
fn import_h_narrative_basic() {
	let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.parent()
		.and_then(|p| p.parent())
		.and_then(|p| p.parent())
		.expect("workspace root")
		.to_path_buf();
	let xml = std::fs::read(root.join("docs/refs/instances/FAERS2022Scenario1.xml"))
		.expect("read sample xml");

	let narrative = parse_h_narrative(&xml).expect("parse").unwrap();
	assert_ne!(narrative.case_narrative, "Imported narrative not provided.");
}
