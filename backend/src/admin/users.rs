use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use serde::Deserialize;
use validator::Validate;

use crate::{
    auth::{extractor::RequireAdmin, password::hash_password},
    error::AppError,
    models::user::{User, UserListRow},
    AppState,
};

async fn list_users(
    RequireAdmin(_user): RequireAdmin,
    State(state): State<AppState>,
) -> Result<Json<Vec<UserListRow>>, AppError> {
    Ok(Json(User::list_all(&state.pool).await?))
}

#[derive(Debug, Deserialize, Validate)]
struct CreateUserInput {
    #[validate(email)]
    email: String,
    #[validate(length(min = 1, max = 200))]
    name: String,
    #[validate(length(min = 8, max = 200))]
    password: String,
    role: String,
}

async fn create_user(
    RequireAdmin(_user): RequireAdmin,
    State(state): State<AppState>,
    Json(input): Json<CreateUserInput>,
) -> Result<(StatusCode, Json<UserListRow>), AppError> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    if input.role != "admin" && input.role != "employee" {
        return Err(AppError::Validation("role must be 'admin' or 'employee'".into()));
    }

    let password_hash = hash_password(&input.password)?;

    match User::create(&state.pool, &input.email, &input.name, &password_hash, &input.role).await
    {
        Ok(user) => Ok((StatusCode::CREATED, Json(user))),
        Err(sqlx::Error::Database(db)) if db.is_unique_violation() => {
            Err(AppError::Conflict("email already in use".into()))
        }
        Err(e) => Err(e.into()),
    }
}

pub fn admin_router() -> Router<AppState> {
    Router::new().route("/", get(list_users).post(create_user))
}
