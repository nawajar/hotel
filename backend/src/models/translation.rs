use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct TranslationOverride {
    pub key: String,
    pub locale: String,
    pub value: String,
}

impl TranslationOverride {
    pub async fn list_all(pool: &PgPool) -> Result<Vec<TranslationOverride>, sqlx::Error> {
        sqlx::query_as!(
            TranslationOverride,
            r#"select key, locale, value from translation_overrides order by key, locale"#
        )
        .fetch_all(pool)
        .await
    }

    pub async fn upsert(
        pool: &PgPool,
        key: &str,
        locale: &str,
        value: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"insert into translation_overrides (key, locale, value, updated_at)
               values ($1, $2, $3, now())
               on conflict (key, locale) do update set value = $3, updated_at = now()"#,
            key,
            locale,
            value
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn delete(pool: &PgPool, key: &str, locale: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"delete from translation_overrides where key = $1 and locale = $2"#,
            key,
            locale
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}
