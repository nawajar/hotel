pub mod admin;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod models;

use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}
