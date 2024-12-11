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

    Ok(database_pool)
}
