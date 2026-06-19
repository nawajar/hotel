use serde::Serialize;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct Note {
    pub id: Uuid,
    pub user_id: Uuid,
    pub body: String,
    pub done: bool,
    pub created_by_name: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl Note {
    pub async fn list_all(pool: &PgPool) -> Result<Vec<Note>, sqlx::Error> {
        sqlx::query_as!(
            Note,
            r#"select n.id, n.user_id, n.body, n.done,
                      u.name as created_by_name,
                      n.created_at, n.updated_at
               from notes n
               join users u on u.id = n.user_id
               order by n.done asc, n.created_at desc"#
        )
        .fetch_all(pool)
        .await
    }

    pub async fn create(pool: &PgPool, user_id: Uuid, body: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "insert into notes (user_id, body) values ($1, $2)",
            user_id,
            body
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn set_done(
        pool: &PgPool,
        id: Uuid,
        done: bool,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "update notes set done = $1, updated_at = now() where id = $2",
            done,
            id,
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!("delete from notes where id = $1", id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
