use crate::{database::Database, secrets::Secrets};

pub mod health;

/// Shared state handed to every HTTP handler. Anything a handler needs to do
/// its job (database access, secrets, future clients) lives here so routes
/// stay pure request/response glue.
pub struct AppState {
    pub db: Database,
    pub secrets: Secrets,
}
