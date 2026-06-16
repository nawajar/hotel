use hotel_backend::auth::password::hash_password;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt().init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    seed_user(&pool, "ADMIN_EMAIL", "ADMIN_PASSWORD", "Admin", "admin").await?;
    seed_user(
        &pool,
        "EMPLOYEE_EMAIL",
        "EMPLOYEE_PASSWORD",
        "Employee",
        "employee",
    )
    .await?;

    Ok(())
}

async fn seed_user(
    pool: &sqlx::PgPool,
    email_var: &str,
    password_var: &str,
    name: &str,
    role: &str,
) -> anyhow::Result<()> {
    let email = match std::env::var(email_var) {
        Ok(v) => v,
        Err(_) => {
            tracing::info!("{email_var} not set, skipping {role} seed");
            return Ok(());
        }
    };
    let password = std::env::var(password_var)
        .unwrap_or_else(|_| panic!("{password_var} must be set when {email_var} is set"));

    let password_hash = hash_password(&password)?;

    let result = sqlx::query!(
        r#"insert into users (email, name, password_hash, role)
           values ($1, $2, $3, $4)
           on conflict (email) do nothing"#,
        email,
        name,
        password_hash,
        role
    )
    .execute(pool)
    .await?;

    if result.rows_affected() > 0 {
        tracing::info!("seeded {role} user: {email}");
    } else {
        tracing::info!("{role} user already exists, skipping: {email}");
    }

    Ok(())
}
