use axum::{extract::State, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::{auth::extractor::RequireAdmin, error::AppError, AppState};

#[derive(Debug, Serialize)]
pub struct SettingsResponse {
    pub timezone: String,
    pub price_symbol: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsInput {
    pub timezone: Option<String>,
    pub price_symbol: Option<String>,
}

async fn fetch_settings(pool: &sqlx::PgPool) -> Result<SettingsResponse, AppError> {
    let rows = sqlx::query(
        "SELECT key, value FROM system_settings WHERE key IN ('timezone', 'price_symbol')",
    )
    .fetch_all(pool)
    .await?;

    let mut timezone = "Asia/Bangkok".to_string();
    let mut price_symbol = "₭".to_string();

    for row in rows {
        let key: String = row.try_get("key")?;
        let value: String = row.try_get("value")?;
        match key.as_str() {
            "timezone" => timezone = value,
            "price_symbol" => price_symbol = value,
            _ => {}
        }
    }

    Ok(SettingsResponse {
        timezone,
        price_symbol,
    })
}

async fn get_settings(
    RequireAdmin(_user): RequireAdmin,
    State(state): State<AppState>,
) -> Result<Json<SettingsResponse>, AppError> {
    Ok(Json(fetch_settings(&state.pool).await?))
}

async fn update_settings(
    RequireAdmin(user): RequireAdmin,
    State(state): State<AppState>,
    Json(input): Json<UpdateSettingsInput>,
) -> Result<Json<SettingsResponse>, AppError> {
    if let Some(ref value) = input.timezone {
        sqlx::query(
            "INSERT INTO system_settings (key, value, updated_at, updated_by) \
             VALUES ($1, $2, now(), $3) \
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = now(), updated_by = EXCLUDED.updated_by",
        )
        .bind("timezone")
        .bind(value)
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    }

    if let Some(ref value) = input.price_symbol {
        sqlx::query(
            "INSERT INTO system_settings (key, value, updated_at, updated_by) \
             VALUES ($1, $2, now(), $3) \
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = now(), updated_by = EXCLUDED.updated_by",
        )
        .bind("price_symbol")
        .bind(value)
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    }

    Ok(Json(fetch_settings(&state.pool).await?))
}

pub fn admin_router() -> Router<AppState> {
    Router::new().route("/", get(get_settings).put(update_settings))
}
