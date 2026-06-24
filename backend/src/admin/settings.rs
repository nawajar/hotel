use axum::{extract::State, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::{auth::extractor::RequireAdmin, error::AppError, AppState};

#[derive(Debug, Serialize)]
pub struct SettingsResponse {
    pub timezone: String,
    pub price_symbol: String,
    pub date_format: String,
    pub font_size: String,
    pub number_format: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsInput {
    pub timezone: Option<String>,
    pub price_symbol: Option<String>,
    pub date_format: Option<String>,
    pub font_size: Option<String>,
    pub number_format: Option<String>,
}

async fn fetch_settings(pool: &sqlx::PgPool) -> Result<SettingsResponse, AppError> {
    let rows = sqlx::query(
        "SELECT key, value FROM system_settings WHERE key IN ('timezone', 'price_symbol', 'date_format', 'font_size', 'number_format')",
    )
    .fetch_all(pool)
    .await?;

    let mut timezone = "Asia/Bangkok".to_string();
    let mut price_symbol = "₭".to_string();
    let mut date_format = "DD/MM/YYYY".to_string();
    let mut font_size = "medium".to_string();
    let mut number_format = "1,234.56".to_string();

    for row in rows {
        let key: String = row.try_get("key")?;
        let value: String = row.try_get("value")?;
        match key.as_str() {
            "timezone" => timezone = value,
            "price_symbol" => price_symbol = value,
            "date_format" => date_format = value,
            "font_size" => font_size = value,
            "number_format" => number_format = value,
            _ => {}
        }
    }

    Ok(SettingsResponse {
        timezone,
        price_symbol,
        date_format,
        font_size,
        number_format,
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

    if let Some(ref value) = input.date_format {
        let valid = ["DD/MM/YYYY", "MM/DD/YYYY", "YYYY-MM-DD", "locale"];
        if !valid.contains(&value.as_str()) {
            return Err(AppError::Validation("Invalid date_format value.".to_string()));
        }
        sqlx::query(
            "INSERT INTO system_settings (key, value, updated_at, updated_by) \
             VALUES ($1, $2, now(), $3) \
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = now(), updated_by = EXCLUDED.updated_by",
        )
        .bind("date_format")
        .bind(value)
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    }

    if let Some(ref value) = input.font_size {
        let valid = ["small", "medium", "large"];
        if !valid.contains(&value.as_str()) {
            return Err(AppError::Validation("Invalid font_size value.".to_string()));
        }
        sqlx::query(
            "INSERT INTO system_settings (key, value, updated_at, updated_by) \
             VALUES ($1, $2, now(), $3) \
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = now(), updated_by = EXCLUDED.updated_by",
        )
        .bind("font_size")
        .bind(value)
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    }

    if let Some(ref value) = input.number_format {
        let valid = ["1,234.56", "1.234,56"];
        if !valid.contains(&value.as_str()) {
            return Err(AppError::Validation("Invalid number_format value.".to_string()));
        }
        sqlx::query(
            "INSERT INTO system_settings (key, value, updated_at, updated_by) \
             VALUES ($1, $2, now(), $3) \
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = now(), updated_by = EXCLUDED.updated_by",
        )
        .bind("number_format")
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
