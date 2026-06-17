mod rooms;
mod users;

use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

use crate::{auth::extractor::RequireAdmin, translations, AppState};

async fn ping(RequireAdmin(_user): RequireAdmin) -> Json<Value> {
    Json(json!({ "ok": true }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ping", get(ping))
        .nest("/translations", translations::admin_router())
        .nest("/users", users::admin_router())
        .nest("/rooms", rooms::admin_router())
}
