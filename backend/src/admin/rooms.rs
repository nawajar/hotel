use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::extractor::RequireAdmin,
    error::AppError,
    models::room::Room,
    AppState,
};

async fn list_rooms(
    RequireAdmin(_user): RequireAdmin,
    State(state): State<AppState>,
) -> Result<Json<Vec<Room>>, AppError> {
    Ok(Json(Room::list_all(&state.pool).await?))
}

#[derive(Debug, Deserialize, Validate)]
struct CreateRoomInput {
    #[validate(length(min = 1, max = 50))]
    room_number: String,
    #[validate(length(min = 1, max = 255))]
    room_name: String,
    room_description: Option<String>,
    is_active: bool,
    price: f64,
}

#[derive(Debug, Deserialize, Validate)]
struct UpdateRoomInput {
    #[validate(length(min = 1, max = 50))]
    room_number: String,
    #[validate(length(min = 1, max = 255))]
    room_name: String,
    room_description: Option<String>,
    is_active: bool,
    price: f64,
}

async fn create_room(
    RequireAdmin(user): RequireAdmin,
    State(state): State<AppState>,
    Json(input): Json<CreateRoomInput>,
) -> Result<(StatusCode, Json<Room>), AppError> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    if input.price < 0.0 {
        return Err(AppError::Validation("price must be >= 0".into()));
    }

    match Room::create(
        &state.pool,
        &input.room_number,
        &input.room_name,
        input.room_description.as_deref(),
        input.is_active,
        input.price,
        user.id,
    )
    .await
    {
        Ok(room) => Ok((StatusCode::CREATED, Json(room))),
        Err(sqlx::Error::Database(db)) if db.is_unique_violation() => {
            Err(AppError::Conflict("room_number already in use".into()))
        }
        Err(e) => Err(e.into()),
    }
}

async fn update_room(
    RequireAdmin(user): RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateRoomInput>,
) -> Result<Json<Room>, AppError> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    if input.price < 0.0 {
        return Err(AppError::Validation("price must be >= 0".into()));
    }

    match Room::update(
        &state.pool,
        id,
        &input.room_number,
        &input.room_name,
        input.room_description.as_deref(),
        input.is_active,
        input.price,
        user.id,
    )
    .await
    {
        Ok(Some(room)) => Ok(Json(room)),
        Ok(None) => Err(AppError::NotFound("room not found".into())),
        Err(sqlx::Error::Database(db)) if db.is_unique_violation() => {
            Err(AppError::Conflict("room_number already in use".into()))
        }
        Err(e) => Err(e.into()),
    }
}

async fn toggle_room(
    RequireAdmin(user): RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Room>, AppError> {
    Room::toggle_active(&state.pool, id, user.id)
        .await?
        .map(Json)
        .ok_or_else(|| AppError::NotFound("room not found".into()))
}

async fn delete_room(
    RequireAdmin(_user): RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    if Room::delete(&state.pool, id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("room not found".into()))
    }
}

pub fn admin_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_rooms).post(create_room))
        .route("/{id}", put(update_room).delete(delete_room))
        .route("/{id}/toggle", put(toggle_room))
}
