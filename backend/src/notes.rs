use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{auth::extractor::AuthUser, error::AppError, models::note::Note, AppState};

#[derive(Debug, Deserialize)]
struct CreateNoteInput {
    body: String,
}

#[derive(Debug, Deserialize)]
struct UpdateNoteInput {
    done: bool,
}

async fn list_notes(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Note>>, AppError> {
    let notes = Note::list_all(&state.pool).await?;
    Ok(Json(notes))
}

async fn create_note(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Json(input): Json<CreateNoteInput>,
) -> Result<StatusCode, AppError> {
    let body = input.body.trim().to_string();
    if body.is_empty() {
        return Err(AppError::Validation("body is required".into()));
    }
    Note::create(&state.pool, user.id, &body).await?;
    Ok(StatusCode::CREATED)
}

async fn update_note(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateNoteInput>,
) -> Result<StatusCode, AppError> {
    let updated = Note::set_done(&state.pool, id, input.done).await?;
    if updated {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("note not found".into()))
    }
}

async fn delete_note(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let deleted = Note::delete(&state.pool, id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("note not found".into()))
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_notes))
        .route("/", axum::routing::post(create_note))
        .route("/{id}", patch(update_note))
        .route("/{id}", axum::routing::delete(delete_note))
}
