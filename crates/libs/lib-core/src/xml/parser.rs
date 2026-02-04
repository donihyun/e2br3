use crate::xml::error::Error;
use crate::xml::types::ParsedE2b;
use crate::xml::validator::{validate_e2b_xml, XmlValidatorConfig};
use crate::xml::Result;
use serde_json::json;

pub fn parse_e2b_xml(xml: &[u8]) -> Result<ParsedE2b> {
	let report = validate_e2b_xml(xml, Some(XmlValidatorConfig::default()))?;
	if !report.ok {
		return Err(Error::XsdValidationFailed {
			errors: report.errors,
		});
	}
	let root = report.root_element.unwrap_or_else(|| "ichicsr".to_string());
	let json = json!({
		"root": root,
		"size_bytes": xml.len(),
	});

	Ok(ParsedE2b {
		root_element: root,
		json,
	})
}
