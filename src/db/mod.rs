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
        
        // Initialize schema
        Self::init_schema(&pool).await?;
        
        Ok(Self { pool })
    }
    
    // Get a reference to the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
    
    // Initialize the database schema
    async fn init_schema(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS polls (
                id TEXT PRIMARY KEY,
                guild_id TEXT NOT NULL,
                channel_id TEXT NOT NULL,
                creator_id TEXT NOT NULL,
                question TEXT NOT NULL,
                voting_method TEXT NOT NULL,
                created_at TEXT NOT NULL,
                ends_at TEXT,
                is_active BOOLEAN NOT NULL DEFAULT TRUE
            );
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS poll_options (
                id TEXT PRIMARY KEY,
                poll_id TEXT NOT NULL,
                text TEXT NOT NULL,
                position INTEGER NOT NULL,
                FOREIGN KEY (poll_id) REFERENCES polls(id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS votes (
                user_id TEXT NOT NULL,
                poll_id TEXT NOT NULL,
                option_id TEXT NOT NULL,
                rating INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                PRIMARY KEY (user_id, poll_id, option_id),
                FOREIGN KEY (poll_id) REFERENCES polls(id) ON DELETE CASCADE,
                FOREIGN KEY (option_id) REFERENCES poll_options(id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(pool)
        .await?;

        Ok(())
    }
    
    // Create a new poll in the database
    pub async fn create_poll(
        &self,
        poll: &crate::models::Poll,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let tx = self.pool.begin().await?;

        // Insert poll
        sqlx::query(
            r#"
            INSERT INTO polls (id, guild_id, channel_id, creator_id, question, voting_method, created_at, ends_at, is_active)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&poll.id)
        .bind(&poll.guild_id)
        .bind(&poll.channel_id)
        .bind(&poll.creator_id)
        .bind(&poll.question)
        .bind(match poll.voting_method {
            crate::models::VotingMethod::Star => "star",
            crate::models::VotingMethod::Plurality => "plurality",
            crate::models::VotingMethod::Ranked => "ranked",
            crate::models::VotingMethod::Approval => "approval",
        })
        .bind(poll.created_at.to_rfc3339())
        .bind(poll.ends_at.map(|dt| dt.to_rfc3339()))
        .bind(poll.is_active)
        .execute(&self.pool)
        .await?;

        // Insert poll options
        for (i, option) in poll.options.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO poll_options (id, poll_id, text, position)
                VALUES (?, ?, ?, ?)
                "#,
            )
            .bind(&option.id)
            .bind(&poll.id)
            .bind(&option.text)
            .bind(i as i64)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }
}
