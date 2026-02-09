// Case-level validation contract shared by regional validators.

mod rules;

use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
pub use rules::{
	find_validation_rule, validation_rules_for_profile, ValidationRuleMetadata,
	VALIDATION_RULES,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ValidationProfile {
	Ich,
	Fda,
	Mfds,
}

impl ValidationProfile {
	pub fn as_str(self) -> &'static str {
		match self {
			Self::Ich => "ich",
			Self::Fda => "fda",
			Self::Mfds => "mfds",
		}
	}

	pub fn parse(value: &str) -> Option<Self> {
		match value.trim().to_ascii_lowercase().as_str() {
			"ich" => Some(Self::Ich),
			"fda" => Some(Self::Fda),
			"mfds" => Some(Self::Mfds),
			_ => None,
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
	pub code: String,
	pub message: String,
	pub path: String,
	pub section: String,
	pub blocking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseValidationReport {
	pub profile: String,
	pub case_id: Uuid,
	pub ok: bool,
	pub blocking_count: usize,
	pub non_blocking_count: usize,
	pub issues: Vec<ValidationIssue>,
}

pub fn has_text(value: Option<&str>) -> bool {
	value.map(|v| !v.trim().is_empty()).unwrap_or(false)
}

pub fn push_issue(
	issues: &mut Vec<ValidationIssue>,
	code: &str,
	message: &str,
	path: impl Into<String>,
	section: &str,
	blocking: bool,
) {
	issues.push(ValidationIssue {
		code: code.to_string(),
		message: message.to_string(),
		path: path.into(),
		section: section.to_string(),
		blocking,
	});
}

pub fn push_issue_by_code(
	issues: &mut Vec<ValidationIssue>,
	code: &str,
	path: impl Into<String>,
) {
	if let Some(rule) = find_validation_rule(code) {
		issues.push(ValidationIssue {
			code: rule.code.to_string(),
			message: rule.message.to_string(),
			path: path.into(),
			section: rule.section.to_string(),
			blocking: rule.blocking,
		});
	} else {
		issues.push(ValidationIssue {
			code: code.to_string(),
			message: code.to_string(),
			path: path.into(),
			section: "unknown".to_string(),
			blocking: false,
		});
	}
}

pub fn build_report(
	profile: ValidationProfile,
	case_id: Uuid,
	issues: Vec<ValidationIssue>,
) -> CaseValidationReport {
	let blocking_count = issues.iter().filter(|issue| issue.blocking).count();
	let non_blocking_count = issues.len().saturating_sub(blocking_count);
	CaseValidationReport {
		profile: profile.as_str().to_string(),
		case_id,
		ok: blocking_count == 0,
		blocking_count,
		non_blocking_count,
		issues,
	}
}
