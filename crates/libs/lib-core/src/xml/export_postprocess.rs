use libxml::tree::{Document, Node, NodeType};
use libxml::xpath::Context;

pub(crate) fn postprocess_export_doc(doc: &mut Document, xpath: &mut Context) {
	normalize_export_values(xpath);
	prune_optional_nodes(doc, xpath);
}

fn normalize_export_values(xpath: &mut Context) {
	if let Ok(nodes) =
		xpath.findnodes("//hl7:value[@codeSystem='2.16.840.1.113883.6.163']", None)
	{
		for mut node in nodes {
			let code = node.get_attribute("code").unwrap_or_default();
			let code_ok =
				code.len() == 8 && code.chars().all(|c| c.is_ascii_digit());
			if !code_ok {
				let _ = node.set_attribute("nullFlavor", "NI");
				let _ = node.remove_attribute("code");
			}
		}
	}

	if let Ok(nodes) =
		xpath.findnodes("//hl7:code[@codeSystem='1.0.3166.1.2.2']", None)
	{
		for mut node in nodes {
			let code = node.get_attribute("code").unwrap_or_default();
			let code_ok =
				code.len() == 2 && code.chars().all(|c| c.is_ascii_uppercase());
			if !code_ok {
				let _ = node.set_attribute("nullFlavor", "NI");
				let _ = node.remove_attribute("code");
			}
		}
	}

	if let Ok(nodes) = xpath.findnodes("//*[@type]", None) {
		for mut node in nodes {
			if let Some(value) = node.get_attribute("type") {
				let _ = node.remove_attribute("type");
				let _ = node.set_attribute("xsi:type", &value);
			}
		}
	}
}

fn prune_optional_nodes(_doc: &mut Document, xpath: &mut Context) {
	let optional_paths = include_str!("fda_optional_paths.txt");
	for raw in optional_paths
		.lines()
		.map(str::trim)
		.filter(|l| !l.is_empty())
	{
		let xp = path_to_xpath(raw);
		if let Ok(nodes) = xpath.findnodes(&xp, None) {
			for node in nodes {
				if node.get_name() == "substanceAdministration" {
					if let Some(parent) = node.get_parent() {
						if parent.get_name() == "outboundRelationship2" {
							if parent.get_attribute("typeCode").as_deref()
								== Some("FLFS")
							{
								continue;
							}
						}
					}
				}
				if !node_has_real_data(&node) {
					let mut n = node;
					n.unlink_node();
				}
			}
		}
	}

	prune_placeholder_nodes(xpath);
	prune_empty_structural_nodes(xpath);
}

fn prune_placeholder_nodes(xpath: &mut Context) {
	let placeholder_value_nodes = [
		"//hl7:observation/hl7:value[@code='G.k.10.r']",
		"//hl7:investigationCharacteristic[hl7:code[@code='3' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.23']]/hl7:value[@code='C.1.11.1']",
		"//hl7:observation/hl7:value[@code='D.2.3']",
		"//hl7:observation/hl7:value[@unit='D.2.2.1b']",
	];
	for path in placeholder_value_nodes {
		if let Ok(nodes) = xpath.findnodes(path, None) {
			for mut node in nodes {
				if let Some(mut parent) = node.get_parent() {
					parent.unlink_node();
				} else {
					node.unlink_node();
				}
			}
		}
	}

	let placeholder_attr_nodes = "//hl7:observation/hl7:value[@codeSystemVersion='D.8.r.6a' or @codeSystemVersion='D.8.r.7a' or @codeSystemVersion='D.9.2.r.1a' or @codeSystemVersion='D.9.4.r.1a']";
	if let Ok(nodes) = xpath.findnodes(placeholder_attr_nodes, None) {
		for mut node in nodes {
			let _ = node.remove_attribute("codeSystemVersion");
		}
	}

	let race_ni = "//hl7:observation[hl7:code[@code='C17049' and @codeSystem='2.16.840.1.113883.3.26.1.1']]/hl7:value[@code='NI']";
	if let Ok(nodes) = xpath.findnodes(race_ni, None) {
		for mut node in nodes {
			if let Some(mut parent) = node.get_parent() {
				parent.unlink_node();
			} else {
				node.unlink_node();
			}
		}
	}
	let race_empty = "//hl7:observation[hl7:code[@code='C17049' and @codeSystem='2.16.840.1.113883.3.26.1.1']]/hl7:value[not(@code) or @nullFlavor]";
	if let Ok(nodes) = xpath.findnodes(race_empty, None) {
		for mut node in nodes {
			if let Some(mut parent) = node.get_parent() {
				parent.unlink_node();
			} else {
				node.unlink_node();
			}
		}
	}

	let gk11_empty = "//hl7:outboundRelationship2[hl7:observation/hl7:code[@code='2'] and (not(hl7:observation/hl7:value) or normalize-space(hl7:observation/hl7:value)='')]";
	if let Ok(nodes) = xpath.findnodes(gk11_empty, None) {
		for mut node in nodes {
			node.unlink_node();
		}
	}

	let doc_text_with_compression = "//hl7:document/hl7:text[@compression]";
	if let Ok(nodes) = xpath.findnodes(doc_text_with_compression, None) {
		for mut node in nodes {
			let _ = node.remove_attribute("compression");
		}
	}

	let summary_lang = "//hl7:component/hl7:observationEvent[hl7:code[@code='36']]/hl7:value[@language='JA']";
	if let Ok(nodes) = xpath.findnodes(summary_lang, None) {
		for mut node in nodes {
			let _ = node.remove_attribute("language");
		}
	}

	let required_intervention = "//hl7:observation[hl7:code[@code='7']]/hl7:value";
	if let Ok(nodes) = xpath.findnodes(required_intervention, None) {
		for mut node in nodes {
			let val = node.get_attribute("value").unwrap_or_default();
			if looks_placeholder(val.trim()) {
				let _ = node.remove_attribute("value");
			}
			if node.get_attribute("nullFlavor").is_none() {
				let _ = node.set_attribute("nullFlavor", "NI");
			}
		}
	}
}

fn prune_empty_structural_nodes(xpath: &mut Context) {
	let structural_paths = [
		"//hl7:outboundRelationship2",
		"//hl7:component",
		"//hl7:component1",
		"//hl7:subjectOf2",
	];
	for path in structural_paths {
		if let Ok(nodes) = xpath.findnodes(path, None) {
			for node in nodes {
				let is_empty = !node_has_real_data(&node)
					&& !node
						.get_child_nodes()
						.into_iter()
						.any(|c| c.get_type() == Some(NodeType::ElementNode));
				if is_empty {
					let mut n = node;
					n.unlink_node();
				}
			}
		}
	}

	let clean_paths = ["//hl7:observation", "//hl7:organizer"];
	for path in clean_paths {
		if let Ok(nodes) = xpath.findnodes(path, None) {
			for node in nodes {
				let has_element_children = node
					.get_child_nodes()
					.into_iter()
					.any(|c| c.get_type() == Some(NodeType::ElementNode));
				if !has_element_children && !node_has_real_data(&node) {
					let mut n = node;
					n.unlink_node();
				}
			}
		}
	}
}

fn path_to_xpath(path: &str) -> String {
	let mut out = String::from("/");
	for (i, part) in path.split('/').enumerate() {
		if i > 0 {
			out.push('/');
		}
		out.push_str("hl7:");
		out.push_str(part);
	}
	out
}

fn node_has_real_data(node: &Node) -> bool {
	if node.get_type() != Some(NodeType::ElementNode) {
		return false;
	}
	let content = node.get_content();
	if !content.trim().is_empty() && !looks_placeholder(content.trim()) {
		return true;
	}
	for attr in [
		"code",
		"value",
		"extension",
		"root",
		"unit",
		"displayName",
		"codeSystem",
		"codeSystemVersion",
		"nullFlavor",
	] {
		if let Some(val) = node.get_attribute(attr) {
			if !val.trim().is_empty() && !looks_placeholder(val.trim()) {
				return true;
			}
		}
	}
	for child in node.get_child_nodes() {
		if child.get_type() == Some(NodeType::ElementNode)
			&& node_has_real_data(&child)
		{
			return true;
		}
	}
	false
}

fn looks_placeholder(value: &str) -> bool {
	let v = value.trim();
	if v.is_empty() {
		return true;
	}
	if let Some(first) = v.chars().next() {
		if first.is_ascii_uppercase() && v.contains('.') {
			return true;
		}
	}
	false
}
