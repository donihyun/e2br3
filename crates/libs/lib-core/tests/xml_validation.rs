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

#[test]
fn test_fda_combination_product_requires_value_or_nullflavor() -> Result<(), Box<dyn Error>> {
	let Some(dir) = examples_dir() else {
		eprintln!("E2BR3_EXAMPLES_DIR not set; skipping XML validation examples");
		return Ok(());
	};

	let xml = read_example(&dir, "1-1_ExampleCase_literature_initial_v1_0.xml")?;
	let fda_xml = xml
		.replacen(
			"extension=\"ICHTEST\" root=\"2.16.840.1.113883.3.989.2.1.3.12\"",
			"extension=\"CDER\" root=\"2.16.840.1.113883.3.989.2.1.3.12\"",
			1,
		)
		.replacen(
			"extension=\"ICHTEST\" root=\"2.16.840.1.113883.3.989.2.1.3.14\"",
			"extension=\"ZZFDA\" root=\"2.16.840.1.113883.3.989.2.1.3.14\"",
			1,
		);
	let insert = "<subjectOf2 typeCode=\"SUBJ\">\
<investigationCharacteristic classCode=\"OBS\" moodCode=\"EVN\">\
<code code=\"1\" codeSystem=\"2.16.840.1.113883.3.989.5.1.2.2.1.3\"/>\
<value xsi:type=\"BL\"/>\
</investigationCharacteristic>\
</subjectOf2>";
	let broken = fda_xml.replacen("</investigationEvent>", &format!("{insert}</investigationEvent>"), 1);
	assert_ne!(broken, fda_xml, "failed to insert FDA.C.1.12 test node");
	let report = validate_e2b_xml(broken.as_bytes(), None)?;
	assert!(!report.ok, "expected FDA.C.1.12 error");
	let has_error = report
		.errors
		.iter()
		.any(|e| e.message.contains("FDA.C.1.12 combination product indicator"));
	assert!(has_error, "FDA.C.1.12 error not reported");

	Ok(())
}

#[test]
fn test_fda_local_criteria_requires_code_or_nullflavor() -> Result<(), Box<dyn Error>> {
	let Some(dir) = examples_dir() else {
		eprintln!("E2BR3_EXAMPLES_DIR not set; skipping XML validation examples");
		return Ok(());
	};

	let xml = read_example(&dir, "1-1_ExampleCase_literature_initial_v1_0.xml")?;
	let fda_xml = xml
		.replacen(
			"extension=\"ICHTEST\" root=\"2.16.840.1.113883.3.989.2.1.3.12\"",
			"extension=\"CDER\" root=\"2.16.840.1.113883.3.989.2.1.3.12\"",
			1,
		)
		.replacen(
			"extension=\"ICHTEST\" root=\"2.16.840.1.113883.3.989.2.1.3.14\"",
			"extension=\"ZZFDA\" root=\"2.16.840.1.113883.3.989.2.1.3.14\"",
			1,
		);
	let insert = "<subjectOf2 typeCode=\"SUBJ\">\
<investigationCharacteristic classCode=\"OBS\" moodCode=\"EVN\">\
<code code=\"2\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/>\
<value xsi:type=\"CE\"/>\
</investigationCharacteristic>\
</subjectOf2>";
	let broken = fda_xml.replacen("</investigationEvent>", &format!("{insert}</investigationEvent>"), 1);
	assert_ne!(broken, fda_xml, "failed to insert FDA.C.1.7.1 test node");
	let report = validate_e2b_xml(broken.as_bytes(), None)?;
	assert!(!report.ok, "expected FDA.C.1.7.1 error");
	let has_error = report
		.errors
		.iter()
		.any(|e| e.message.contains("FDA.C.1.7.1 local criteria report type"));
	assert!(has_error, "FDA.C.1.7.1 error not reported");

	Ok(())
}

#[test]
fn test_fda_patient_race_requires_code_or_nullflavor() -> Result<(), Box<dyn Error>> {
	let Some(dir) = examples_dir() else {
		eprintln!("E2BR3_EXAMPLES_DIR not set; skipping XML validation examples");
		return Ok(());
	};

	let xml = read_example(&dir, "1-1_ExampleCase_literature_initial_v1_0.xml")?;
	let fda_xml = xml
		.replacen(
			"extension=\"ICHTEST\" root=\"2.16.840.1.113883.3.989.2.1.3.12\"",
			"extension=\"CDER\" root=\"2.16.840.1.113883.3.989.2.1.3.12\"",
			1,
		)
		.replacen(
			"extension=\"ICHTEST\" root=\"2.16.840.1.113883.3.989.2.1.3.14\"",
			"extension=\"ZZFDA\" root=\"2.16.840.1.113883.3.989.2.1.3.14\"",
			1,
		);
	let insert = "<player1 classCode=\"PSN\" determinerCode=\"INSTANCE\">\
<raceCode/>"; 
	let broken = fda_xml.replacen(
		"<player1 classCode=\"PSN\" determinerCode=\"INSTANCE\">",
		insert,
		1,
	);
	assert_ne!(broken, fda_xml, "failed to insert FDA.D.11 test node");
	let report = validate_e2b_xml(broken.as_bytes(), None)?;
	assert!(!report.ok, "expected FDA.D.11 error");
	let has_error = report
		.errors
		.iter()
		.any(|e| e.message.contains("FDA.D.11 patient race"));
	assert!(has_error, "FDA.D.11 error not reported");

	Ok(())
}

#[test]
fn test_fda_patient_ethnicity_requires_code_or_nullflavor() -> Result<(), Box<dyn Error>> {
	let Some(dir) = examples_dir() else {
		eprintln!("E2BR3_EXAMPLES_DIR not set; skipping XML validation examples");
		return Ok(());
	};

	let xml = read_example(&dir, "1-1_ExampleCase_literature_initial_v1_0.xml")?;
	let fda_xml = xml
		.replacen(
			"extension=\"ICHTEST\" root=\"2.16.840.1.113883.3.989.2.1.3.12\"",
			"extension=\"CDER\" root=\"2.16.840.1.113883.3.989.2.1.3.12\"",
			1,
		)
		.replacen(
			"extension=\"ICHTEST\" root=\"2.16.840.1.113883.3.989.2.1.3.14\"",
			"extension=\"ZZFDA\" root=\"2.16.840.1.113883.3.989.2.1.3.14\"",
			1,
		);
	let insert = "<subjectOf2 typeCode=\"SBJ\">\
<observation classCode=\"OBS\" moodCode=\"EVN\">\
<code code=\"C16564\" codeSystem=\"2.16.840.1.113883.3.26.1.1\"/>\
<value xsi:type=\"CE\"/>\
</observation>\
</subjectOf2>";
	let broken = fda_xml.replacen("<subjectOf2 typeCode=\"SBJ\">", insert, 1);
	assert_ne!(broken, fda_xml, "failed to insert FDA.D.12 test node");
	let report = validate_e2b_xml(broken.as_bytes(), None)?;
	assert!(!report.ok, "expected FDA.D.12 error");
	let has_error = report
		.errors
		.iter()
		.any(|e| e.message.contains("FDA.D.12 patient ethnicity"));
	assert!(has_error, "FDA.D.12 error not reported");

	Ok(())
}

#[test]
fn test_fda_required_intervention_requires_value_or_nullflavor() -> Result<(), Box<dyn Error>> {
	let Some(dir) = examples_dir() else {
		eprintln!("E2BR3_EXAMPLES_DIR not set; skipping XML validation examples");
		return Ok(());
	};

	let xml = read_example(&dir, "1-1_ExampleCase_literature_initial_v1_0.xml")?;
	let fda_xml = xml
		.replacen(
			"extension=\"ICHTEST\" root=\"2.16.840.1.113883.3.989.2.1.3.12\"",
			"extension=\"CDER\" root=\"2.16.840.1.113883.3.989.2.1.3.12\"",
			1,
		)
		.replacen(
			"extension=\"ICHTEST\" root=\"2.16.840.1.113883.3.989.2.1.3.14\"",
			"extension=\"ZZFDA\" root=\"2.16.840.1.113883.3.989.2.1.3.14\"",
			1,
		);
	let insert = "<outboundRelationship2 typeCode=\"PERT\">\
<observation classCode=\"OBS\" moodCode=\"EVN\">\
<code code=\"726\" codeSystem=\"2.16.840.1.113883.3.989.5.1.2.2.1.32\"/>\
<value xsi:type=\"BL\"/>\
</observation>\
</outboundRelationship2>";
	let broken = fda_xml.replacen("</outboundRelationship2>", &format!("{insert}</outboundRelationship2>"), 1);
	assert_ne!(broken, fda_xml, "failed to insert FDA.E.i.3.2h test node");
	let report = validate_e2b_xml(broken.as_bytes(), None)?;
	assert!(!report.ok, "expected FDA.E.i.3.2h error");
	let has_error = report
		.errors
		.iter()
		.any(|e| e.message.contains("FDA.E.i.3.2h required intervention"));
	assert!(has_error, "FDA.E.i.3.2h error not reported");

	Ok(())
}

#[test]
fn test_fda_gk10a_requires_code_or_na_when_pre_anda_present() -> Result<(), Box<dyn Error>> {
	let Some(dir) = examples_dir() else {
		eprintln!("E2BR3_EXAMPLES_DIR not set; skipping XML validation examples");
		return Ok(());
	};

	let xml = read_example(&dir, "1-1_ExampleCase_literature_initial_v1_0.xml")?;

	let fda_xml = xml
		.replacen(
			"extension=\"ICHTEST\" root=\"2.16.840.1.113883.3.989.2.1.3.12\"",
			"extension=\"CDER_IND_EXEMPT_BA_BE\" root=\"2.16.840.1.113883.3.989.2.1.3.12\"",
			1,
		)
		.replacen(
			"extension=\"ICHTEST\" root=\"2.16.840.1.113883.3.989.2.1.3.14\"",
			"extension=\"ZZFDA_PREMKT\" root=\"2.16.840.1.113883.3.989.2.1.3.14\"",
			1,
		);
	let pre_anda = "<subjectOf1 typeCode=\"SBJ\"><researchStudy classCode=\"CLNTRL\" moodCode=\"EVN\"><authorization typeCode=\"AUTH\"><studyRegistration classCode=\"ACT\" moodCode=\"EVN\"><id root=\"2.16.840.1.113883.3.989.5.1.2.2.1.2.2\" extension=\"234567\"/></studyRegistration></authorization></researchStudy></subjectOf1>";
	let with_pre_anda = fda_xml.replacen("</investigationEvent>", &format!("{pre_anda}</investigationEvent>"), 1);
	assert_ne!(with_pre_anda, fda_xml, "failed to insert FDA.C.5.5b test node");

	let bad_gk10a = "<outboundRelationship2 typeCode=\"REFR\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"9\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value xsi:type=\"CE\" code=\"9\"/></observation></outboundRelationship2>";
	let broken = with_pre_anda.replacen("</substanceAdministration>", &format!("{bad_gk10a}</substanceAdministration>"), 1);
	assert_ne!(broken, with_pre_anda, "failed to insert FDA.G.k.10a test node");

	let report = validate_e2b_xml(broken.as_bytes(), None)?;
	assert!(!report.ok, "expected FDA.G.k.10a error");
	let has_error = report
		.errors
		.iter()
		.any(|e| e.message.contains("FDA.G.k.10a"));
	assert!(has_error, "FDA.G.k.10a error not reported");

	Ok(())
}

#[test]
fn test_fda_reporter_email_required_when_primary_source_present() -> Result<(), Box<dyn Error>> {
	let Some(dir) = examples_dir() else {
		eprintln!("E2BR3_EXAMPLES_DIR not set; skipping XML validation examples");
		return Ok(());
	};

	let xml = read_example(&dir, "1-1_ExampleCase_literature_initial_v1_0.xml")?;
	let fda_xml = xml
		.replacen(
			"extension=\"ICHTEST\" root=\"2.16.840.1.113883.3.989.2.1.3.12\"",
			"extension=\"CDER\" root=\"2.16.840.1.113883.3.989.2.1.3.12\"",
			1,
		)
		.replacen(
			"extension=\"ICHTEST\" root=\"2.16.840.1.113883.3.989.2.1.3.14\"",
			"extension=\"ZZFDA\" root=\"2.16.840.1.113883.3.989.2.1.3.14\"",
			1,
		);
	let broken = fda_xml.replace("mailto:", "mail:");
	let report = validate_e2b_xml(broken.as_bytes(), None)?;
	assert!(!report.ok, "expected reporter email error");
	let has_error = report
		.errors
		.iter()
		.any(|e| e.message.contains("reporter email"));
	assert!(has_error, "reporter email error not reported");

	Ok(())
}

#[test]
fn test_fda_pre_anda_required_for_ind_exempt() -> Result<(), Box<dyn Error>> {
	let Some(dir) = examples_dir() else {
		eprintln!("E2BR3_EXAMPLES_DIR not set; skipping XML validation examples");
		return Ok(());
	};

	let xml = read_example(&dir, "1-1_ExampleCase_literature_initial_v1_0.xml")?;
	let broken = xml
		.replacen(
			"extension=\"ICHTEST\" root=\"2.16.840.1.113883.3.989.2.1.3.12\"",
			"extension=\"CDER_IND_EXEMPT_BA_BE\" root=\"2.16.840.1.113883.3.989.2.1.3.12\"",
			1,
		)
		.replacen(
			"extension=\"ICHTEST\" root=\"2.16.840.1.113883.3.989.2.1.3.14\"",
			"extension=\"ZZFDA_PREMKT\" root=\"2.16.840.1.113883.3.989.2.1.3.14\"",
			1,
		)
		.replacen(
			"code=\"1\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.2\"",
			"code=\"2\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.2\"",
			1,
		);
	let report = validate_e2b_xml(broken.as_bytes(), None)?;
	assert!(!report.ok, "expected FDA.C.5.5b required error");
	let has_error = report
		.errors
		.iter()
		.any(|e| e.message.contains("FDA.C.5.5b required"));
	assert!(has_error, "FDA.C.5.5b required error not reported");

	Ok(())
}

#[test]
fn test_fda_pre_anda_not_allowed_postmarket() -> Result<(), Box<dyn Error>> {
	let Some(dir) = examples_dir() else {
		eprintln!("E2BR3_EXAMPLES_DIR not set; skipping XML validation examples");
		return Ok(());
	};

	let xml = read_example(&dir, "1-1_ExampleCase_literature_initial_v1_0.xml")?;
	let pre_anda = "<subjectOf1 typeCode=\"SBJ\"><researchStudy classCode=\"CLNTRL\" moodCode=\"EVN\"><authorization typeCode=\"AUTH\"><studyRegistration classCode=\"ACT\" moodCode=\"EVN\"><id root=\"2.16.840.1.113883.3.989.5.1.2.2.1.2.2\" extension=\"234567\"/></studyRegistration></authorization></researchStudy></subjectOf1>";
	let with_pre_anda = xml.replacen("</investigationEvent>", &format!("{pre_anda}</investigationEvent>"), 1);
	let broken = with_pre_anda
		.replacen(
			"extension=\"ICHTEST\" root=\"2.16.840.1.113883.3.989.2.1.3.12\"",
			"extension=\"CDER\" root=\"2.16.840.1.113883.3.989.2.1.3.12\"",
			1,
		)
		.replacen(
			"extension=\"ICHTEST\" root=\"2.16.840.1.113883.3.989.2.1.3.14\"",
			"extension=\"ZZFDA\" root=\"2.16.840.1.113883.3.989.2.1.3.14\"",
			1,
		);
	let report = validate_e2b_xml(broken.as_bytes(), None)?;
	assert!(!report.ok, "expected FDA.C.5.5b not allowed error");
	let has_error = report
		.errors
		.iter()
		.any(|e| e.message.contains("FDA.C.5.5b must not be provided"));
	assert!(has_error, "FDA.C.5.5b not allowed error not reported");

	Ok(())
}
