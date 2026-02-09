use lib_core::xml::import_sections::g_drug::parse_g_drugs;

#[test]
fn import_g_drug_basic() {
	let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.parent()
		.and_then(|p| p.parent())
		.and_then(|p| p.parent())
		.expect("workspace root")
		.to_path_buf();
	let xml = std::fs::read(root.join("docs/refs/instances/FAERS2022Scenario1.xml"))
		.expect("read sample xml");

	let drugs = parse_g_drugs(&xml).expect("parse");
	assert!(!drugs.is_empty());
}
