use super::AppState;

/// Reports whether the service is up. Will grow to check the database pool
/// once there's something worth failing on.
pub async fn status(_state: &AppState) -> &'static str {
    "ok"
}
