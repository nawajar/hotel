use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::Deserialize;
use tower_sessions::Session;
use validator::Validate;

use crate::{
    auth::{
        extractor::{insert_user_session, AuthUser},
        password::verify_password,
    },
    error::AppError,
    models::user::{User, UserPublic},
    AppState,
};

#[derive(Debug, Deserialize, Validate)]
pub struct LoginInput {
    #[validate(length(min = 1, max = 255))]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
}

async fn login(
    State(state): State<AppState>,
    session: Session,
    Json(input): Json<LoginInput>,
) -> Result<Json<UserPublic>, AppError> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user = User::find_by_email(&state.pool, &input.email)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !verify_password(&input.password, &user.password_hash) {
        return Err(AppError::Unauthorized);
    }

    insert_user_session(&session, user.id).await?;

    Ok(Json(user.into()))
}

async fn logout(session: Session) -> Result<StatusCode, AppError> {
    session
        .flush()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn me(AuthUser(user): AuthUser) -> Json<UserPublic> {
    Json(user.into())
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", axum::routing::get(me))
}
