use std::collections::HashMap;

use axum::{extract::State, routing::get, Json, Router};
use serde::Deserialize;
use validator::Validate;

use crate::{
    auth::extractor::RequireAdmin, error::AppError, models::translation::TranslationOverride,
    AppState,
};

/// Public: every label on the frontend falls back to its bundled default, so
/// this only needs to ship the overrides an admin has saved. No auth required
/// since even the login page's labels must be overridable.
async fn list_public(State(state): State<AppState>) -> Result<Json<HashMap<String, HashMap<String, String>>>, AppError> {
    let overrides = TranslationOverride::list_all(&state.pool).await?;
    let mut by_locale: HashMap<String, HashMap<String, String>> = HashMap::new();
    for o in overrides {
        by_locale.entry(o.locale).or_default().insert(o.key, o.value);
    }
    Ok(Json(by_locale))
}

async fn list_admin(
    RequireAdmin(_user): RequireAdmin,
    State(state): State<AppState>,
) -> Result<Json<Vec<TranslationOverride>>, AppError> {
    Ok(Json(TranslationOverride::list_all(&state.pool).await?))
}

#[derive(Debug, Deserialize, Validate)]
struct UpsertInput {
    #[validate(length(min = 1, max = 200))]
    key: String,
    #[validate(length(equal = 2))]
    locale: String,
    #[validate(length(min = 1, max = 2000))]
    value: String,
}

async fn upsert_admin(
    RequireAdmin(_user): RequireAdmin,
    State(state): State<AppState>,
    Json(input): Json<UpsertInput>,
) -> Result<(), AppError> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    if input.locale != "en" && input.locale != "lo" {
        return Err(AppError::Validation("locale must be 'en' or 'lo'".into()));
    }
    TranslationOverride::upsert(&state.pool, &input.key, &input.locale, &input.value).await?;
    Ok(())
}

#[derive(Debug, Deserialize, Validate)]
struct DeleteInput {
    #[validate(length(min = 1, max = 200))]
    key: String,
    #[validate(length(equal = 2))]
    locale: String,
}

async fn delete_admin(
    RequireAdmin(_user): RequireAdmin,
    State(state): State<AppState>,
    Json(input): Json<DeleteInput>,
) -> Result<(), AppError> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    TranslationOverride::delete(&state.pool, &input.key, &input.locale).await?;
    Ok(())
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list_public))
}

pub fn admin_router() -> Router<AppState> {
    Router::new().route("/", get(list_admin).put(upsert_admin).delete(delete_admin))
}
