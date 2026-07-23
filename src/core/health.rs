use serde::Serialize;

use super::AppState;

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: &'static str,
    pub database: &'static str,
}

impl HealthStatus {
    pub fn is_healthy(&self) -> bool {
        self.status == "ok"
    }
}

/// Reports whether the service and its dependencies are up.
pub async fn status(state: &AppState) -> HealthStatus {
    let database = match sqlx::query("SELECT 1").execute(&state.db.pool).await {
        Ok(_) => "ok",
        Err(err) => {
            tracing::error!(error = ?err, "database health check failed");
            "unreachable"
        }
    };

    HealthStatus {
        status: if database == "ok" { "ok" } else { "degraded" },
        database,
    }
}
