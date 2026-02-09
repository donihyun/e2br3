use lib_core::xml::import_sections::e_reaction::parse_e_reactions;

#[test]
fn import_e_reaction_basic() {
	let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.parent()
		.and_then(|p| p.parent())
		.and_then(|p| p.parent())
		.expect("workspace root")
		.to_path_buf();
	let xml = std::fs::read(root.join("docs/refs/instances/FAERS2022Scenario1.xml"))
		.expect("read sample xml");

	let reactions = parse_e_reactions(&xml).expect("parse");
	assert!(!reactions.is_empty());
}
