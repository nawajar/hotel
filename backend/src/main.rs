use axum::Router;
use hotel_backend::{admin, auth, bookings, calendar, config::Config, db, translations, AppState};
use tower_sessions::{cookie::Key, cookie::SameSite, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = Config::from_env();
    let pool = db::connect(&config.database_url).await;

    tokio::fs::create_dir_all(&config.uploads_dir)
        .await
        .expect("failed to create uploads directory");

    let session_store = PostgresStore::new(pool.clone());
    let session_key = Key::derive_from(config.session_secret.as_bytes());
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(config.cookie_secure)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(7)))
        .with_signed(session_key);

    let state = AppState { pool, uploads_dir: config.uploads_dir };

    let app = Router::new()
        .nest("/api/auth", auth::routes::router())
        .nest("/api/admin", admin::router())
        .nest("/api/bookings", bookings::router())
        .nest("/api/calendar", calendar::router())
        .nest("/api/translations", translations::router())
        .with_state(state)
        .layer(session_layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to 0.0.0.0:3000");
    tracing::info!("listening on 0.0.0.0:3000");
    axum::serve(listener, app).await.expect("server error");
}
