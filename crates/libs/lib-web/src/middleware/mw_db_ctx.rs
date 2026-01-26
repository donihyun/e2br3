use crate::error::{Error, Result};
use crate::middleware::mw_auth::CtxW;
use axum::body::Body;
use axum::extract::State;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use lib_core::model::store::set_full_context_dbx;
use lib_core::model::{self, ModelManager};
use std::sync::Arc;

pub async fn mw_ctx_require_and_set_dbx(
	State(mm): State<ModelManager>,
	ctx: Result<CtxW>,
	req: Request<Body>,
	next: Next,
) -> Result<Response> {
	let ctx = ctx?;
	let dbx = mm.dbx();

	dbx.begin_txn()
		.await
		.map_err(model::Error::from)
		.map_err(Error::from)?;

	if let Err(err) = set_full_context_dbx(
		dbx,
		ctx.0.user_id(),
		ctx.0.organization_id(),
		ctx.0.role(),
	)
	.await
	{
		let _ = dbx.rollback_txn().await;
		return Err(Error::from(err));
	}

	let res = next.run(req).await;

	if res.extensions().get::<Arc<Error>>().is_some() {
		let _ = dbx.rollback_txn().await;
	} else {
		let _ = dbx
			.commit_txn()
			.await
			.map_err(model::Error::from)
			.map_err(Error::from)?;
	}

	Ok(res)
}
