use hotel_backend::auth::password::hash_password;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

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

    if let Some(admin_id) = get_admin_id(&pool).await? {
        seed_rooms(&pool, admin_id).await?;
    } else {
        tracing::warn!("no admin user found, skipping room seed");
    }

    Ok(())
}

async fn get_admin_id(pool: &sqlx::PgPool) -> anyhow::Result<Option<Uuid>> {
    let row = sqlx::query!("select id from users where role = 'admin' limit 1")
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.id))
}

async fn seed_rooms(pool: &sqlx::PgPool, updated_by: Uuid) -> anyhow::Result<()> {
    struct RoomSeed {
        number: &'static str,
        name: &'static str,
        description: &'static str,
        price: f64,
    }

    let rooms = [
        // Floor 1 — Standard
        RoomSeed { number: "101", name: "Standard Single",  description: "Cozy single room with garden view",          price: 250_000.0 },
        RoomSeed { number: "102", name: "Standard Single",  description: "Cozy single room with garden view",          price: 250_000.0 },
        RoomSeed { number: "103", name: "Standard Double",  description: "Comfortable double room with city view",     price: 320_000.0 },
        RoomSeed { number: "104", name: "Standard Double",  description: "Comfortable double room with city view",     price: 320_000.0 },
        RoomSeed { number: "105", name: "Standard Twin",    description: "Twin beds, ideal for two guests",            price: 340_000.0 },
        RoomSeed { number: "106", name: "Standard Twin",    description: "Twin beds, ideal for two guests",            price: 340_000.0 },
        RoomSeed { number: "107", name: "Standard Single",  description: "Cozy single room with courtyard view",       price: 250_000.0 },
        RoomSeed { number: "108", name: "Standard Double",  description: "Comfortable double room with pool view",     price: 350_000.0 },
        RoomSeed { number: "109", name: "Standard Twin",    description: "Twin beds with balcony access",              price: 360_000.0 },
        RoomSeed { number: "110", name: "Standard Double",  description: "Corner room with large windows",             price: 370_000.0 },
        // Floor 2 — Deluxe
        RoomSeed { number: "201", name: "Deluxe King",      description: "Spacious king room with city panorama",      price: 480_000.0 },
        RoomSeed { number: "202", name: "Deluxe King",      description: "Spacious king room with pool view",          price: 500_000.0 },
        RoomSeed { number: "203", name: "Deluxe Double",    description: "Deluxe double with upgraded amenities",      price: 460_000.0 },
        RoomSeed { number: "204", name: "Deluxe Double",    description: "Deluxe double with upgraded amenities",      price: 460_000.0 },
        RoomSeed { number: "205", name: "Deluxe Twin",      description: "Deluxe twin with private balcony",           price: 490_000.0 },
        RoomSeed { number: "206", name: "Deluxe Twin",      description: "Deluxe twin with private balcony",           price: 490_000.0 },
        RoomSeed { number: "207", name: "Deluxe King",      description: "Corner king room with wraparound view",      price: 550_000.0 },
        RoomSeed { number: "208", name: "Deluxe Family",    description: "Family room with extra sofa bed",            price: 580_000.0 },
        RoomSeed { number: "209", name: "Deluxe Family",    description: "Family room with extra sofa bed",            price: 580_000.0 },
        RoomSeed { number: "210", name: "Deluxe King",      description: "End-of-hall king with quiet garden view",    price: 520_000.0 },
        // Floor 3 — Suites & Premium
        RoomSeed { number: "301", name: "Junior Suite",     description: "Separate living area and king bedroom",      price: 800_000.0 },
        RoomSeed { number: "302", name: "Junior Suite",     description: "Separate living area and king bedroom",      price: 800_000.0 },
        RoomSeed { number: "303", name: "Executive Suite",  description: "Full living room, dining area, king bed",    price: 1_100_000.0 },
        RoomSeed { number: "304", name: "Executive Suite",  description: "Full living room, dining area, king bed",    price: 1_100_000.0 },
        RoomSeed { number: "305", name: "Junior Suite",     description: "Corner suite with panoramic windows",        price: 900_000.0 },
        RoomSeed { number: "306", name: "Junior Suite",     description: "Corner suite with panoramic windows",        price: 900_000.0 },
        RoomSeed { number: "307", name: "Executive Suite",  description: "Two-bedroom suite, ideal for families",      price: 1_200_000.0 },
        RoomSeed { number: "308", name: "Premium Room",     description: "Premium room with private terrace",          price: 750_000.0 },
        RoomSeed { number: "309", name: "Premium Room",     description: "Premium room with private terrace",          price: 750_000.0 },
        RoomSeed { number: "310", name: "Penthouse Suite",  description: "Top-floor suite, private rooftop access",    price: 2_500_000.0 },
    ];

    let mut inserted = 0u32;
    let mut skipped = 0u32;

    for r in &rooms {
        let result = sqlx::query!(
            r#"insert into rooms (room_number, room_name, room_description, is_active, price, updated_by)
               values ($1, $2, $3, true, $4, $5)
               on conflict (room_number) do nothing"#,
            r.number,
            r.name,
            r.description,
            r.price,
            updated_by,
        )
        .execute(pool)
        .await?;

        if result.rows_affected() > 0 {
            inserted += 1;
        } else {
            skipped += 1;
        }
    }

    tracing::info!("rooms: {inserted} inserted, {skipped} already existed");
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
