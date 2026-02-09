use crate::model::reaction::Reaction;
use crate::xml::Result;
use sqlx::types::time::Date;

pub fn export_e_reactions_xml(reactions: &[Reaction]) -> Result<String> {
	let mut reactions_xml = String::new();
	for reaction in reactions {
		reactions_xml.push_str(&reaction_fragment(reaction));
	}
	let xml = base_e_reaction_skeleton().replace("{REACTIONS}", &reactions_xml);
	Ok(xml)
}

pub(crate) fn reaction_fragment(reaction: &Reaction) -> String {
	let mut out = String::new();
	out.push_str("<subjectOf2 typeCode=\"SBJ\"><observation classCode=\"OBS\" moodCode=\"EVN\">");
	out.push_str(
		"<code code=\"29\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/>",
	);

	let mut value_attrs = String::new();
	if let Some(code) = reaction.reaction_meddra_code.as_deref() {
		value_attrs.push_str(" code=\"");
		value_attrs.push_str(&xml_escape(code));
		value_attrs.push_str("\"");
	}
	if let Some(version) = reaction.reaction_meddra_version.as_deref() {
		value_attrs.push_str(" codeSystemVersion=\"");
		value_attrs.push_str(&xml_escape(version));
		value_attrs.push_str("\"");
	}
	out.push_str(&format!(
		"<value xsi:type=\"CE\"{value_attrs}><originalText",
	));
	if let Some(lang) = reaction.reaction_language.as_deref() {
		out.push_str(" language=\"");
		out.push_str(&xml_escape(lang));
		out.push_str("\"");
	}
	out.push_str(">\"");
	out.push_str(&xml_escape(&reaction.primary_source_reaction));
	out.push_str("\"</originalText></value>");

	if reaction.start_date.is_some()
		|| reaction.end_date.is_some()
		|| reaction.duration_value.is_some()
	{
		out.push_str("<effectiveTime>");
		if let Some(start) = reaction.start_date {
			out.push_str("<low value=\"");
			out.push_str(&fmt_date(start));
			out.push_str("\"/>");
		}
		if let Some(end) = reaction.end_date {
			out.push_str("<high value=\"");
			out.push_str(&fmt_date(end));
			out.push_str("\"/>");
		}
		if let Some(width) = reaction.duration_value.as_ref() {
			out.push_str("<comp operator=\"A\"><width value=\"");
			out.push_str(&xml_escape(&width.to_string()));
			out.push_str("\"");
			if let Some(unit) = reaction.duration_unit.as_deref() {
				out.push_str(" unit=\"");
				out.push_str(&xml_escape(unit));
				out.push_str("\"");
			}
			out.push_str("/></comp>");
		}
		out.push_str("</effectiveTime>");
	}

	if let Some(term_code) =
		term_highlight_code(reaction.term_highlighted, reaction.serious)
	{
		out.push_str(&observation_rel_code("37", &term_code));
	}

	if reaction.criteria_death {
		out.push_str(&observation_rel_bool("34", true));
	}
	if reaction.criteria_life_threatening {
		out.push_str(&observation_rel_bool("21", true));
	}
	if reaction.criteria_hospitalization {
		out.push_str(&observation_rel_bool("33", true));
	}
	if reaction.criteria_disabling {
		out.push_str(&observation_rel_bool("35", true));
	}
	if reaction.criteria_congenital_anomaly {
		out.push_str(&observation_rel_bool("12", true));
	}
	if reaction.criteria_other_medically_important {
		out.push_str(&observation_rel_bool("26", true));
	}

	if let Some(value) = reaction.required_intervention.as_deref() {
		out.push_str(&observation_rel_value("7", value));
	}

	if let Some(outcome) = reaction.outcome.as_deref() {
		out.push_str(&observation_rel_code("27", outcome));
	}
	if let Some(value) = reaction.medical_confirmation {
		out.push_str(&observation_rel_bool("24", value));
	}

	if let Some(country) = reaction.country_code.as_deref() {
		out.push_str("<location><locatedEntity><locatedPlace><code code=\"");
		out.push_str(&xml_escape(country));
		out.push_str("\"/></locatedPlace></locatedEntity></location>");
	}

	out.push_str("</observation></subjectOf2>");
	out
}

fn observation_rel_bool(code: &str, value: bool) -> String {
	let v = if value { "true" } else { "false" };
	format!(
		"<outboundRelationship2 typeCode=\"COMP\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"{code}\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value xsi:type=\"BL\" value=\"{v}\"/></observation></outboundRelationship2>"
	)
}

fn observation_rel_code(code: &str, value: &str) -> String {
	format!(
		"<outboundRelationship2 typeCode=\"COMP\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"{code}\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value xsi:type=\"CE\" code=\"{}\"/></observation></outboundRelationship2>",
		xml_escape(value)
	)
}

fn observation_rel_value(code: &str, value: &str) -> String {
	format!(
		"<outboundRelationship2 typeCode=\"COMP\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"{code}\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value xsi:type=\"ST\" value=\"{}\"/></observation></outboundRelationship2>",
		xml_escape(value)
	)
}

fn term_highlight_code(
	term_highlighted: Option<bool>,
	serious: Option<bool>,
) -> Option<String> {
	match (term_highlighted, serious) {
		(Some(true), Some(true)) => Some("3".to_string()),
		(Some(true), Some(false)) => Some("1".to_string()),
		(Some(false), Some(true)) => Some("4".to_string()),
		(Some(false), Some(false)) => Some("2".to_string()),
		_ => None,
	}
}

fn fmt_date(date: Date) -> String {
	format!(
		"{:04}{:02}{:02}",
		date.year(),
		u8::from(date.month()),
		date.day()
	)
}

fn xml_escape(value: &str) -> String {
	value
		.replace('&', "&amp;")
		.replace('<', "&lt;")
		.replace('>', "&gt;")
		.replace('"', "&quot;")
		.replace('\'', "&apos;")
}

fn base_e_reaction_skeleton() -> &'static str {
	"<?xml version=\"1.0\" encoding=\"utf-8\"?>\
<MCCI_IN200100UV01 xmlns=\"urn:hl7-org:v3\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" ITSVersion=\"XML_1.0\">\
\t<PORR_IN049016UV>\
\t\t<controlActProcess classCode=\"CACT\" moodCode=\"EVN\">\
\t\t\t<code code=\"PORR_TE049016UV\" codeSystem=\"2.16.840.1.113883.1.18\"/>\
\t\t\t<subject>\
\t\t\t\t<investigationEvent classCode=\"INVSTG\" moodCode=\"EVN\">\
\t\t\t\t\t<component typeCode=\"COMP\">\
\t\t\t\t\t\t<adverseEventAssessment classCode=\"INVSTG\" moodCode=\"EVN\">\
\t\t\t\t\t\t\t<subject1 typeCode=\"SBJ\">\
\t\t\t\t\t\t\t\t<primaryRole classCode=\"INVSBJ\">\
\t\t\t\t\t\t\t\t\t<player1 classCode=\"PSN\" determinerCode=\"INSTANCE\"><name/></player1>\
\t\t\t\t\t\t\t\t\t{REACTIONS}\
\t\t\t\t\t\t\t\t</primaryRole>\
\t\t\t\t\t\t\t</subject1>\
\t\t\t\t\t\t</adverseEventAssessment>\
\t\t\t\t\t</component>\
\t\t\t\t</investigationEvent>\
\t\t\t</subject>\
\t\t</controlActProcess>\
\t</PORR_IN049016UV>\
</MCCI_IN200100UV01>"
}
