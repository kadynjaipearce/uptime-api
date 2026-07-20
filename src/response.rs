use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// Uniform success envelope for handlers: `{ "status": <code>, "body": <T> }`.
/// Pairs with `AppError`, which renders `{ "status": <code>, "message": <string> }`
/// on failure, so every response from the API has the same top-level shape.
pub struct ApiResponse<T: Serialize> {
    status: StatusCode,
    body: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(status: StatusCode, body: T) -> Self {
        Self { status, body }
    }

    pub fn ok(body: T) -> Self {
        Self::new(StatusCode::OK, body)
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct Envelope<T> {
            status: u16,
            body: T,
        }

        (
            self.status,
            Json(Envelope {
                status: self.status.as_u16(),
                body: self.body,
            }),
        )
            .into_response()
    }
}
