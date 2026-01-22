mod common;

use common::{demo_user_id, init_test_mm, set_current_user, Result};
use lib_core::ctx::Ctx;
use lib_core::model::terminology::{
	E2bCodeListBmc, IsoCountryBmc, MeddraTermBmc, WhodrugProductBmc,
};
use serial_test::serial;

#[serial]
#[tokio::test]
async fn test_terminology_queries() -> Result<()> {
	let mm = init_test_mm().await;
	let ctx = Ctx::root_ctx();

	set_current_user(&mm, demo_user_id()).await?;

	sqlx::query(
		"INSERT INTO meddra_terms (code, term, level, version, language)
		 VALUES ($1, $2, $3, $4, $5)",
	)
	.bind("10000001")
	.bind("TestTerm Alpha")
	.bind("PT")
	.bind("v1")
	.bind("en")
	.execute(mm.dbx().db())
	.await?;

	sqlx::query(
		"INSERT INTO whodrug_products (code, drug_name, atc_code, version, language)
		 VALUES ($1, $2, $3, $4, $5)",
	)
	.bind("WTEST1")
	.bind("TestDrug Alpha")
	.bind("A00")
	.bind("v1")
	.bind("en")
	.execute(mm.dbx().db())
	.await?;

	sqlx::query(
		"INSERT INTO iso_countries (code, name, active) VALUES ($1, $2, true)",
	)
	.bind("ZZ")
	.bind("Zedland")
	.execute(mm.dbx().db())
	.await?;

	let meddra_terms =
		MeddraTermBmc::search(&ctx, &mm, "TestTerm", Some("v1"), 5).await?;
	assert!(meddra_terms.iter().any(|t| t.code == "10000001"));

	let whodrug = WhodrugProductBmc::search(&ctx, &mm, "TestDrug", 5).await?;
	assert!(whodrug.iter().any(|p| p.code == "WTEST1"));

	let countries = IsoCountryBmc::list_all(&ctx, &mm).await?;
	assert!(countries.iter().any(|c| c.code == "ZZ"));

	let report_types =
		E2bCodeListBmc::get_by_list_name(&ctx, &mm, "report_type").await?;
	assert!(!report_types.is_empty());

	sqlx::query("DELETE FROM meddra_terms WHERE code = $1 AND version = $2")
		.bind("10000001")
		.bind("v1")
		.execute(mm.dbx().db())
		.await?;
	sqlx::query("DELETE FROM whodrug_products WHERE code = $1 AND version = $2")
		.bind("WTEST1")
		.bind("v1")
		.execute(mm.dbx().db())
		.await?;
	sqlx::query("DELETE FROM iso_countries WHERE code = $1")
		.bind("ZZ")
		.execute(mm.dbx().db())
		.await?;

	Ok(())
}
