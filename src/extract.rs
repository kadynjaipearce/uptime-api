use axum::{
    Json,
    extract::{FromRequest, Request, rejection::JsonRejection},
};
use serde::de::DeserializeOwned;

use crate::error::AppError;

/// Drop-in replacement for `axum::extract::Json` that renders a malformed or
/// mistyped body through the same `{ "status", "message" }` envelope as
/// every other error, instead of axum's default plain-text rejection body.
pub struct AppJson<T>(pub T);

impl<T, S> FromRequest<S> for AppJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection: JsonRejection| AppError::BadRequest(rejection.body_text()))?;

        Ok(AppJson(value))
    }
}
