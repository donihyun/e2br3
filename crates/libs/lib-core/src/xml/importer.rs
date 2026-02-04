use crate::ctx::Ctx;
use crate::model::audit::{CaseVersionBmc, CaseVersionForCreate};
use crate::model::store::set_full_context_dbx_or_rollback;
use crate::model::ModelManager;
use crate::xml::error::Error;
use crate::xml::types::XmlImportResult;
use crate::xml::validator::validate_e2b_xml;
use crate::xml::{parse_e2b_xml, Result};
use libxml::parser::Parser;
use libxml::xpath::Context;
use serde_json::json;
use sqlx::types::Uuid;

#[derive(Debug, Clone)]
pub struct XmlImportRequest {
	pub xml: Vec<u8>,
	pub filename: Option<String>,	
}

pub async fn import_e2b_xml(
	ctx: &Ctx,
	mm: &ModelManager,
	req: XmlImportRequest,
) -> Result<XmlImportResult> {
	let report = validate_e2b_xml(&req.xml, None)?;
	if !report.ok {
		return Err(Error::XsdValidationFailed {
			errors: report.errors,
		});
	}

	let parsed = parse_e2b_xml(&req.xml)?;
	let safety_report_id = extract_safety_report_id(&req.xml)?;

	mm.dbx()
		.begin_txn()
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))?;
	set_full_context_dbx_or_rollback(
		mm.dbx(),
		ctx.user_id(),
		ctx.organization_id(),
		ctx.role(),
	)
	.await
	.map_err(|e| Error::Model(e))?;

	let case_id = match mm
		.dbx()
		.fetch_one(
			sqlx::query_as::<_, (Uuid,)>(
				"INSERT INTO cases (organization_id, safety_report_id, status, created_by, updated_by) VALUES ($1, $2, $3, $4, $5) RETURNING id",
			)
			.bind(ctx.organization_id())
			.bind(&safety_report_id)
			.bind("draft")
			.bind(ctx.user_id())
			.bind(ctx.user_id()),
		)
		.await
	{
		Ok((id,)) => id,
		Err(err) => {
			mm.dbx()
				.rollback_txn()
				.await
				.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))?;
			return Err(Error::Model(crate::model::Error::Store(format!("{err}"))));
		}
	};

	let snapshot = json!({
		"parsed": parsed.json,
		"raw_xml": String::from_utf8_lossy(&req.xml),
	});

	let version_id = match CaseVersionBmc::create(
		ctx,
		mm,
		CaseVersionForCreate {
			case_id,
			version: 1,
			snapshot,
			change_reason: Some("XML import".to_string()),
		},
	)
		.await
	{
		Ok(id) => id,
		Err(err) => {
			mm.dbx()
				.rollback_txn()
				.await
				.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))?;
			return Err(err.into());
		}
	};

	mm.dbx()
		.commit_txn()
		.await
		.map_err(|e| Error::Model(crate::model::Error::Store(format!("{e}"))))?;

	Ok(XmlImportResult {
		case_id: Some(case_id.to_string()),
		case_version: Some(1),
		xml_key: None,
		parsed_json_id: Some(version_id.to_string()),
	})
}

fn extract_safety_report_id(xml: &[u8]) -> Result<String> {
	let xml_str = std::str::from_utf8(xml).map_err(|err| Error::InvalidXml {
		message: format!("XML not valid UTF-8: {err}"),
		line: None,
		column: None,
	})?;
	let parser = Parser::default();
	let doc = parser.parse_string(xml_str).map_err(|err| Error::InvalidXml {
		message: format!("XML parse error: {err}"),
		line: None,
		column: None,
	})?;
	let mut xpath = Context::new(&doc).map_err(|_| Error::InvalidXml {
		message: "Failed to initialize XPath context".to_string(),
		line: None,
		column: None,
	})?;
	let _ = xpath.register_namespace("hl7", "urn:hl7-org:v3");

	let candidates = xpath
		.findvalues(
			"//hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.1']/@extension",
			None,
		)
		.map_err(|_| Error::InvalidXml {
			message: "Failed to query safety_report_id".to_string(),
			line: None,
			column: None,
		})?;
	for value in candidates {
		if !value.trim().is_empty() {
			return Ok(value);
		}
	}

	Err(Error::InvalidXml {
		message: "safety_report_id not found".to_string(),
		line: None,
		column: None,
	})
}
