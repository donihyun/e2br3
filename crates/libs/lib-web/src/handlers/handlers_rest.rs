use axum::routing::MethodRouter;
use axum::Router;

pub fn rest_collection_item_routes<S>(
	collection_path: &'static str,
	item_path: &'static str,
	collection: MethodRouter<S>,
	item: MethodRouter<S>,
) -> Router<S>
where
	S: Clone + Send + Sync + 'static,
{
	Router::new()
		.route(collection_path, collection)
		.route(item_path, item)
}

pub fn rest_singleton_routes<S>(
	path: &'static str,
	methods: MethodRouter<S>,
) -> Router<S>
where
	S: Clone + Send + Sync + 'static,
{
	Router::new().route(path, methods)
}
