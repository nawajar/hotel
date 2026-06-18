use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use time::{format_description, Date, OffsetDateTime, Time};

use crate::{
    auth::extractor::AuthUser,
    error::AppError,
    models::booking::{Booking, CalendarBooking},
    models::room::Room,
    AppState,
};

#[derive(Debug, Deserialize)]
struct CalendarBookingsParams {
    start: String,
    end: String,
}

async fn get_calendar_bookings(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Query(params): Query<CalendarBookingsParams>,
) -> Result<Json<Vec<CalendarBooking>>, AppError> {
    let fmt = format_description::parse_owned::<2>("[year]-[month]-[day]")
        .map_err(|_| AppError::Validation("invalid date format".into()))?;

    let start_date = Date::parse(&params.start, &fmt)
        .map_err(|_| AppError::Validation("start must be YYYY-MM-DD".into()))?;
    let end_date = Date::parse(&params.end, &fmt)
        .map_err(|_| AppError::Validation("end must be YYYY-MM-DD".into()))?;

    let start_bound = OffsetDateTime::new_utc(start_date, Time::MIDNIGHT);
    let end_bound = OffsetDateTime::new_utc(
        end_date,
        Time::from_hms(23, 59, 59).map_err(|_| AppError::Validation("invalid time".into()))?,
    );

    let bookings = Booking::calendar_list(&state.pool, start_bound, end_bound).await?;
    Ok(Json(bookings))
}

async fn get_calendar_rooms(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Room>>, AppError> {
    let rooms = Room::list_all(&state.pool).await?;
    let active: Vec<Room> = rooms.into_iter().filter(|r| r.is_active).collect();
    Ok(Json(active))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/bookings", get(get_calendar_bookings))
        .route("/rooms", get(get_calendar_rooms))
}
