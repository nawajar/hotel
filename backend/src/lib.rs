pub mod admin;
pub mod auth;
pub mod bookings;
pub mod calendar;
pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod translations;

use std::path::PathBuf;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub uploads_dir: PathBuf,
}
