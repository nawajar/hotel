use std::{env, path::PathBuf};

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub session_secret: String,
    /// Sets the session cookie's `Secure` attribute. Must be `true` once served over
    /// HTTPS; defaults to `false` so plain-HTTP local/dev setups still work.
    pub cookie_secure: bool,
    pub uploads_dir: PathBuf,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            session_secret: env::var("SESSION_SECRET").expect("SESSION_SECRET must be set"),
            cookie_secure: env::var("COOKIE_SECURE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            uploads_dir: env::var("UPLOADS_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("/app/uploads")),
        }
    }
}
