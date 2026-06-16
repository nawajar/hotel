use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub session_secret: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            session_secret: env::var("SESSION_SECRET").expect("SESSION_SECRET must be set"),
        }
    }
}
