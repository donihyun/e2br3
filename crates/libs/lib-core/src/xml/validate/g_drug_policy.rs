// Shared Section G policy used by exporter + case validators.
use super::{has_export_directive, ExportDirective};

pub fn has_drug_characterization(value: &str) -> bool {
	!value.trim().is_empty()
}

pub fn has_medicinal_product(value: &str) -> bool {
	!value.trim().is_empty()
}

pub fn normalize_drug_characterization(value: &str) -> &'static str {
	match value.trim() {
		"1" => "1",
		"2" => "2",
		"3" => "3",
		_ => {
			let default_is_concomitant = has_export_directive(
				"ICH.G.k.1.REQUIRED",
				ExportDirective::DrugRoleDefaultConcomitant,
			);
			if default_is_concomitant { "2" } else { "1" }
		}
	}
}

pub fn drug_characterization_display_name(code: &str) -> &'static str {
	match code {
		"1" => "Suspect",
		"2" => "Concomitant",
		"3" => "Interacting",
		_ => "Concomitant",
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn normalize_drug_characterization_defaults_to_2() {
		assert_eq!(normalize_drug_characterization(""), "2");
		assert_eq!(normalize_drug_characterization("99"), "2");
	}

	#[test]
	fn normalize_drug_characterization_preserves_valid() {
		assert_eq!(normalize_drug_characterization("1"), "1");
		assert_eq!(normalize_drug_characterization("2"), "2");
		assert_eq!(normalize_drug_characterization("3"), "3");
	}
}
