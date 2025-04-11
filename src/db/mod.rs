use sqlx::{migrate::MigrateDatabase, sqlite::{SqlitePool, SqlitePoolOptions}, Sqlite};
use std::env;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Get database URL from environment or use a default
        let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:rusty_bote.db".to_string());
        
        // Create database if it doesn't exist
        if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
            Sqlite::create_database(&db_url).await?;
        }
        
        // Connect to the database
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;
        
        // In a complete implementation, we would run migrations here
        // sqlx::migrate!("./migrations").run(&pool).await?;
        
        Ok(Self { pool })
    }
    
    // Get a reference to the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
