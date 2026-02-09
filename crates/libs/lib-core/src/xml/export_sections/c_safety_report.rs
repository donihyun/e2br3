// Section C exporter (Safety Report Identification) - FDA mapping.

use crate::model::case::Case;
use crate::model::safety_report::SafetyReportIdentification;
use crate::xml::raw::patch::{patch_c_safety_report, CSafetyReportPatch};
use crate::xml::Result;
use libxml::parser::Parser;

pub fn export_c_safety_report_patch(
	raw_xml: &[u8],
	case: &Case,
	report: &SafetyReportIdentification,
) -> Result<String> {
	let patch = CSafetyReportPatch {
		report_unique_id: &case.safety_report_id,
		transmission_date: report.transmission_date,
		report_type: &report.report_type,
		date_first_received: report.date_first_received_from_source,
		date_most_recent: report.date_of_most_recent_information,
		fulfil_expedited: report.fulfil_expedited_criteria,
		worldwide_unique_id: report.worldwide_unique_id.as_deref(),
		local_criteria_report_type: report.local_criteria_report_type.as_deref(),
		combination_product_indicator: report
			.combination_product_report_indicator
			.as_deref(),
		nullification_code: report.nullification_code.as_deref(),
		nullification_reason: report.nullification_reason.as_deref(),
	};

	patch_c_safety_report(raw_xml, &patch)
}

/// Build a minimal ICSR XML skeleton and populate Section C using mapping-driven patching.
pub fn export_c_safety_report_xml(
	case: &Case,
	report: &SafetyReportIdentification,
) -> Result<String> {
	let base_xml = base_icrs_skeleton();
	let parser = Parser::default();
	let doc = parser.parse_string(base_xml).map_err(|err| {
		crate::xml::error::Error::InvalidXml {
			message: format!("XML parse error (base skeleton): {err}"),
			line: None,
			column: None,
		}
	})?;
	let raw = doc.to_string();
	export_c_safety_report_patch(raw.as_bytes(), case, report)
}

fn base_icrs_skeleton() -> &'static str {
	"<?xml version=\"1.0\" encoding=\"utf-8\"?>\
<MCCI_IN200100UV01 xmlns=\"urn:hl7-org:v3\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" ITSVersion=\"XML_1.0\">\
\t<PORR_IN049016UV>\
\t\t<controlActProcess classCode=\"CACT\" moodCode=\"EVN\">\
\t\t\t<code code=\"PORR_TE049016UV\" codeSystem=\"2.16.840.1.113883.1.18\"/>\
\t\t\t<subject>\
\t\t\t\t<investigationEvent classCode=\"INVSTG\" moodCode=\"EVN\">\
\t\t\t\t</investigationEvent>\
\t\t\t</subject>\
\t\t</controlActProcess>\
\t</PORR_IN049016UV>\
</MCCI_IN200100UV01>"
}
