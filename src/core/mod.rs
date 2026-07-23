use crate::{database::Database, secrets::Secrets};

pub mod health;
pub mod scheduler;

pub struct AppState {
    pub db: Database,
    pub secrets: Secrets,
}
