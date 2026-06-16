use serde::Serialize;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserPublic {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
}

impl From<User> for UserPublic {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            name: u.name,
            role: u.role,
        }
    }
}

/// A user row for admin listing/creation responses — omits password_hash.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct UserListRow {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"select id, email::text as "email!", name, password_hash, role
               from users where email = $1"#,
            email
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"select id, email::text as "email!", name, password_hash, role
               from users where id = $1"#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn list_all(pool: &PgPool) -> Result<Vec<UserListRow>, sqlx::Error> {
        sqlx::query_as!(
            UserListRow,
            r#"select id, email::text as "email!", name, role, created_at
               from users order by created_at desc"#
        )
        .fetch_all(pool)
        .await
    }

    pub async fn create(
        pool: &PgPool,
        email: &str,
        name: &str,
        password_hash: &str,
        role: &str,
    ) -> Result<UserListRow, sqlx::Error> {
        sqlx::query_as!(
            UserListRow,
            r#"insert into users (email, name, password_hash, role)
               values ($1, $2, $3, $4)
               returning id, email::text as "email!", name, role, created_at"#,
            email,
            name,
            password_hash,
            role
        )
        .fetch_one(pool)
        .await
    }
}
