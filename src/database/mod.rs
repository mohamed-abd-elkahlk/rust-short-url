use std::env;

use actix_web::Error;

pub type DatabasePool = Pool<MySql>;

use sqlx::{MySql, MySqlPool, Pool};

/// Initializes the MySQL database connection pool and runs the migration script
pub async fn init_db() -> Result<DatabasePool, Error> {
    // Get the DATABASE_URL from the environment
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Connect to the MySQL database pool
    let database_pool = MySqlPool::connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    // Run the SQL statements to create the 'users' and 'short_urls' tables
    sqlx::query(
        r#"
   CREATE TABLE IF NOT EXISTS users (
    id VARCHAR(36) PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT FALSE,
    roles JSON
)"#,
    )
    .execute(&database_pool)
    .await
    .expect("Failed to create tables");

    sqlx::query(
        r#"
CREATE TABLE IF NOT EXISTS short_urls (
    id VARCHAR(36) PRIMARY KEY,
    original_url TEXT NOT NULL,
    short_code VARCHAR(10) UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expiration TIMESTAMP NULL,
    click_count BIGINT DEFAULT 0
);"#,
    )
    .execute(&database_pool)
    .await
    .expect("Failed to create tables");

    Ok(database_pool)
}
