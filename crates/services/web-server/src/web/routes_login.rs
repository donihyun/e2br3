use axum::middleware;
use axum::routing::post;
use axum::Router;
use lib_core::model::ModelManager;
use lib_web::handlers::handlers_login;
use lib_web::middleware::mw_db_ctx::mw_ctx_require_and_set_dbx;

pub fn routes(mm: ModelManager) -> Router {
	let routes_public = Router::new()
		.route("/login", post(handlers_login::api_login_handler))
		.route("/logoff", post(handlers_login::api_logoff_handler));

	let routes_authed = Router::new()
		.route("/refresh", post(handlers_login::api_refresh_handler))
		.route_layer(middleware::from_fn_with_state(
			mm.clone(),
			mw_ctx_require_and_set_dbx,
		));

	Router::new()
		.merge(routes_public)
		.merge(routes_authed)
		.with_state(mm)
}
