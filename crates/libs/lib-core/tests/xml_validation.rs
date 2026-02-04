use lib_core::xml::validate_e2b_xml;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

fn examples_dir() -> Option<PathBuf> {
	std::env::var("E2BR3_EXAMPLES_DIR")
		.ok()
		.map(PathBuf::from)
}

fn read_example(dir: &Path, filename: &str) -> Result<String, Box<dyn Error>> {
	let path = dir.join(filename);
	let content = fs::read_to_string(&path)?;
	Ok(content)
}

#[test]
fn test_examples_validate_ok() -> Result<(), Box<dyn Error>> {
	let Some(dir) = examples_dir() else {
		eprintln!("E2BR3_EXAMPLES_DIR not set; skipping XML validation examples");
		return Ok(());
	};

	let files = [
		"1-1_ExampleCase_literature_initial_v1_0.xml",
		"1-2_ExampleCase_nullification_v1_0.xml",
		"2-1_ExampleCase_newNum_initial_v1_0.xml",
		"3_ExampleCase_Clinical_trial_v1_0.xml",
	];

	for file in files {
		let xml = read_example(&dir, file)?;
		let report = validate_e2b_xml(xml.as_bytes(), None)?;
		assert!(report.ok, "{file} failed validation: {:?}", report.errors);
	}

	Ok(())
}

#[test]
fn test_invalid_telecom_fails() -> Result<(), Box<dyn Error>> {
	let Some(dir) = examples_dir() else {
		eprintln!("E2BR3_EXAMPLES_DIR not set; skipping XML validation examples");
		return Ok(());
	};

	let xml = read_example(&dir, "1-1_ExampleCase_literature_initial_v1_0.xml")?;
	let broken = xml.replace("tel:", "phone:");
	let report = validate_e2b_xml(broken.as_bytes(), None)?;
	assert!(!report.ok, "expected telecom error");
	let has_error = report
		.errors
		.iter()
		.any(|e| e.message.contains("telecom value must start"));
	assert!(has_error, "telecom error not reported");

	Ok(())
}

#[test]
fn test_invalid_reaction_term_fails() -> Result<(), Box<dyn Error>> {
	let Some(dir) = examples_dir() else {
		eprintln!("E2BR3_EXAMPLES_DIR not set; skipping XML validation examples");
		return Ok(());
	};

	let xml = read_example(&dir, "1-1_ExampleCase_literature_initial_v1_0.xml")?;
	let broken = xml.replace("code=\"10022617\"", "");
	let report = validate_e2b_xml(broken.as_bytes(), None)?;
	assert!(!report.ok, "expected reaction term error");
	let has_error = report
		.errors
		.iter()
		.any(|e| e.message.contains("reaction term missing code"));
	assert!(has_error, "reaction term error not reported");

	Ok(())
}

#[test]
fn test_missing_schema_location_fails() -> Result<(), Box<dyn Error>> {
	let Some(dir) = examples_dir() else {
		eprintln!("E2BR3_EXAMPLES_DIR not set; skipping XML validation examples");
		return Ok(());
	};

	let xml = read_example(&dir, "1-1_ExampleCase_literature_initial_v1_0.xml")?;
	let broken = xml.replace("xsi:schemaLocation=\"urn:hl7-org:v3 MCCI_IN200100UV01.xsd\"", "");
	let report = validate_e2b_xml(broken.as_bytes(), None)?;
	assert!(!report.ok, "expected schemaLocation error");
	let has_schema_error = report
		.errors
		.iter()
		.any(|e| e.message.contains("schemaLocation"));
	assert!(has_schema_error, "missing schemaLocation error not reported");

	Ok(())
}
