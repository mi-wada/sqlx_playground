use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

async fn connect_postgres() -> Result<Pool<Postgres>> {
    Ok(PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://user:password@localhost:5432/db")
        .await?)
}

#[tokio::main]
async fn main() -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use sqlx::{error::ErrorKind, Row};

    use super::*;

    async fn insert_user(pool: &Pool<Postgres>, name: &str, email: &str) -> Result<i32> {
        let row = sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id")
            .bind(name)
            .bind(email)
            .fetch_one(pool)
            .await?;

        Ok(row.get("id"))
    }

    #[tokio::test]
    async fn select_number() -> Result<()> {
        let pool = connect_postgres().await?;

        let row: (i64,) = sqlx::query_as("SELECT $1")
            .bind(150_i64)
            .fetch_one(&pool)
            .await?;

        assert_eq!(row.0, 150_i64);

        Ok(())
    }

    #[tokio::test]
    async fn insert_ok() -> Result<()> {
        let pool = connect_postgres().await?;

        let row = sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *")
            .bind("John Doe")
            .bind("hoge@example.com")
            .fetch_one(&pool)
            .await?;

        assert!(row.try_get::<i32, _>("id").is_ok());
        assert_eq!(row.get::<String, _>("name"), "John Doe");
        assert_eq!(row.get::<String, _>("email"), "hoge@example.com");
        assert_eq!(row.get::<Option<String>, _>("note"), None);
        assert_eq!(row.get::<bool, _>("is_active"), true);

        Ok(())
    }

    #[tokio::test]
    async fn select_one_record_ok() -> Result<()> {
        let pool = connect_postgres().await?;

        let user_id = insert_user(&pool, "John Doe", "hoge@example.com").await?;

        let row = sqlx::query("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await?;

        assert!(row.try_get::<i32, _>("id").is_ok());
        assert_eq!(row.get::<String, _>("name"), "John Doe");
        assert_eq!(row.get::<String, _>("email"), "hoge@example.com");
        assert_eq!(row.get::<Option<String>, _>("note"), None);
        assert_eq!(row.get::<bool, _>("is_active"), true);

        Ok(())
    }

    #[tokio::test]
    async fn select_multi_records_ok() -> Result<()> {
        let pool = connect_postgres().await?;

        let user_id_1 = insert_user(&pool, "John Doe", "hoge@example.com").await?;
        let user_id_2 = insert_user(&pool, "Hello", "hello@example.com").await?;

        let rows = sqlx::query("SELECT * FROM users WHERE id = ANY($1)")
            .bind(vec![user_id_1, user_id_2])
            .fetch_all(&pool)
            .await?;

        assert_eq!(rows.len(), 2);
        let row_1 = &rows[0];
        assert_eq!(row_1.get::<i32, _>("id"), user_id_1);
        assert_eq!(row_1.get::<String, _>("name"), "John Doe");
        assert_eq!(row_1.get::<String, _>("email"), "hoge@example.com");
        assert_eq!(row_1.get::<Option<String>, _>("note"), None);
        assert_eq!(row_1.get::<bool, _>("is_active"), true);

        let row_2 = &rows[1];
        assert_eq!(row_2.get::<i32, _>("id"), user_id_2);
        assert_eq!(row_2.get::<String, _>("name"), "Hello");
        assert_eq!(row_2.get::<String, _>("email"), "hello@example.com");
        assert_eq!(row_2.get::<Option<String>, _>("note"), None);
        assert_eq!(row_2.get::<bool, _>("is_active"), true);

        Ok(())
    }

    #[tokio::test]
    async fn select_not_found() -> Result<()> {
        let pool = connect_postgres().await?;

        let res = sqlx::query("SELECT * FROM users WHERE id = $1")
            .bind(-1)
            .fetch_one(&pool)
            .await;

        assert!(matches!(res, Err(sqlx::Error::RowNotFound)));

        Ok(())
    }

    #[tokio::test]
    async fn insert_err_null() -> Result<()> {
        let pool = connect_postgres().await?;

        let res = sqlx::query("INSERT INTO users (name) VALUES ($1) RETURNING id")
            .bind("John Doe")
            .fetch_one(&pool)
            .await;

        assert!(
            matches!(res, Err(sqlx::Error::Database(err)) if err.kind() == ErrorKind::NotNullViolation)
        );

        Ok(())
    }
}
