use crate::{database::Database, secrets::Secrets};

pub mod handler;
pub mod health;
pub mod scheduler;

pub struct AppState {
    pub db: Database,
    pub secrets: Secrets,
}
