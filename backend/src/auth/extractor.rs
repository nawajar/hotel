use axum::{extract::FromRequestParts, http::request::Parts};
use tower_sessions::Session;
use uuid::Uuid;

use crate::{error::AppError, models::user::User, AppState};

const USER_ID_KEY: &str = "user_id";

/// Requires a valid session pointing at an existing user. 401 otherwise.
pub struct AuthUser(pub User);

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized)?;

        let user_id: Uuid = session
            .get(USER_ID_KEY)
            .await
            .map_err(|_| AppError::Unauthorized)?
            .ok_or(AppError::Unauthorized)?;

        let user = User::find_by_id(&state.pool, user_id)
            .await?
            .ok_or(AppError::Unauthorized)?;

        Ok(AuthUser(user))
    }
}

/// Requires a valid session AND an admin role. 401 if unauthenticated, 403 if not admin.
pub struct RequireAdmin(pub User);

impl FromRequestParts<AppState> for RequireAdmin {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let AuthUser(user) = AuthUser::from_request_parts(parts, state).await?;
        if !user.is_admin() {
            return Err(AppError::Forbidden);
        }
        Ok(RequireAdmin(user))
    }
}

pub async fn insert_user_session(session: &Session, user_id: Uuid) -> Result<(), AppError> {
    session
        .insert(USER_ID_KEY, user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))
}
