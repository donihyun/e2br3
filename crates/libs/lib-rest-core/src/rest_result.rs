use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct DataRestResult<T>
where
	T: Serialize,
{
	pub data: T,
}

impl<T> From<T> for DataRestResult<T>
where
	T: Serialize,
{
	fn from(val: T) -> Self {
		Self { data: val }
	}
}
impl<T> IntoResponse for DataRestResult<T>
where
	T: Serialize,
{
	fn into_response(self) -> axum::response::Response {
		Json(self).into_response()
	}
}

pub fn created<T: Serialize>(data: T) -> impl IntoResponse {
	(StatusCode::CREATED, Json(DataRestResult { data }))
}

pub fn ok<T: Serialize>(data: T) -> impl IntoResponse {
	(StatusCode::OK, Json(DataRestResult { data }))
}

pub fn no_content() -> impl IntoResponse {
	StatusCode::NO_CONTENT
}
