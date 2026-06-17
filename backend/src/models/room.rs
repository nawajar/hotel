use serde::Serialize;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Room {
    pub id: Uuid,
    pub room_number: String,
    pub room_name: String,
    pub room_description: Option<String>,
    pub is_active: bool,
    pub price: f64,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub updated_by: Uuid,
}

impl Room {
    pub async fn list_all(pool: &PgPool) -> Result<Vec<Room>, sqlx::Error> {
        sqlx::query_as!(
            Room,
            r#"select id, room_number, room_name, room_description,
                      is_active, price, updated_at, updated_by
               from rooms
               order by room_number asc"#
        )
        .fetch_all(pool)
        .await
    }

    pub async fn create(
        pool: &PgPool,
        room_number: &str,
        room_name: &str,
        room_description: Option<&str>,
        is_active: bool,
        price: f64,
        updated_by: Uuid,
    ) -> Result<Room, sqlx::Error> {
        sqlx::query_as!(
            Room,
            r#"insert into rooms (room_number, room_name, room_description, is_active, price, updated_by)
               values ($1, $2, $3, $4, $5, $6)
               returning id, room_number, room_name, room_description,
                         is_active, price, updated_at, updated_by"#,
            room_number,
            room_name,
            room_description,
            is_active,
            price,
            updated_by
        )
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        room_number: &str,
        room_name: &str,
        room_description: Option<&str>,
        is_active: bool,
        price: f64,
        updated_by: Uuid,
    ) -> Result<Option<Room>, sqlx::Error> {
        sqlx::query_as!(
            Room,
            r#"update rooms
               set room_number      = $2,
                   room_name        = $3,
                   room_description = $4,
                   is_active        = $5,
                   price            = $6,
                   updated_by       = $7,
                   updated_at       = now()
               where id = $1
               returning id, room_number, room_name, room_description,
                         is_active, price, updated_at, updated_by"#,
            id,
            room_number,
            room_name,
            room_description,
            is_active,
            price,
            updated_by
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn toggle_active(
        pool: &PgPool,
        id: Uuid,
        updated_by: Uuid,
    ) -> Result<Option<Room>, sqlx::Error> {
        sqlx::query_as!(
            Room,
            r#"update rooms
               set is_active  = not is_active,
                   updated_by = $2,
                   updated_at = now()
               where id = $1
               returning id, room_number, room_name, room_description,
                         is_active, price, updated_at, updated_by"#,
            id,
            updated_by
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!("delete from rooms where id = $1", id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
