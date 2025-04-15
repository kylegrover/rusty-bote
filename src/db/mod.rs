use sqlx::{migrate::MigrateDatabase, sqlite::{SqlitePool, SqlitePoolOptions}, Sqlite, Row};
use chrono::{DateTime, Utc};
use std::env;
use crate::models::{Poll, PollOption, Vote, VotingMethod};

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
                is_active BOOLEAN NOT NULL DEFAULT TRUE,
                message_id TEXT
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
        sqlx::query(
            r#"
            INSERT INTO polls (id, guild_id, channel_id, creator_id, question, voting_method, created_at, ends_at, is_active, message_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, NULL)
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

    // Update the message ID for a poll
    pub async fn update_poll_message_id(
        &self,
        poll_id: &str,
        message_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            r#"
            UPDATE polls
            SET message_id = ?
            WHERE id = ?
            "#,
        )
        .bind(message_id)
        .bind(poll_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    
    // Get a poll by ID
    pub async fn get_poll(
        &self,
        poll_id: &str,
    ) -> Result<crate::models::Poll, Box<dyn std::error::Error + Send + Sync>> {
        // Get the poll
        let poll_row = sqlx::query(
            r#"
            SELECT id, guild_id, channel_id, creator_id, question, voting_method, created_at, ends_at, is_active, message_id 
            FROM polls 
            WHERE id = ?
            "#,
        )
        .bind(poll_id)
        .fetch_one(&self.pool)
        .await?;
        
        // Extract poll data
        let id = poll_row.get::<String, _>("id");
        let guild_id = poll_row.get::<String, _>("guild_id");
        let channel_id = poll_row.get::<String, _>("channel_id");
        let creator_id = poll_row.get::<String, _>("creator_id");
        let question = poll_row.get::<String, _>("question");
        let voting_method_str = poll_row.get::<String, _>("voting_method");
        let created_at_str = poll_row.get::<String, _>("created_at");
        let ends_at_str: Option<String> = poll_row.get("ends_at");
        let is_active = poll_row.get::<bool, _>("is_active");
        let message_id: Option<String> = poll_row.get("message_id");
        
        // Parse dates
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| format!("Failed to parse created_at: {}", e))?
            .with_timezone(&Utc);
        
        let ends_at = if let Some(ends_at_str) = ends_at_str {
            Some(
                DateTime::parse_from_rfc3339(&ends_at_str)
                    .map_err(|e| format!("Failed to parse ends_at: {}", e))?
                    .with_timezone(&Utc),
            )
        } else {
            None
        };
        
        // Parse voting method
        let voting_method = match voting_method_str.as_str() {
            "star" => crate::models::VotingMethod::Star,
            "plurality" => crate::models::VotingMethod::Plurality,
            "ranked" => crate::models::VotingMethod::Ranked,
            "approval" => crate::models::VotingMethod::Approval,
            _ => return Err(format!("Unknown voting method: {}", voting_method_str).into()),
        };
        
        // Get options
        let options = sqlx::query(
            r#"
            SELECT id, text, position
            FROM poll_options
            WHERE poll_id = ?
            ORDER BY position
            "#,
        )
        .bind(poll_id)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| crate::models::PollOption {
            id: row.get::<String, _>("id"),
            text: row.get::<String, _>("text"),
        })
        .collect();
        
        // Create poll object
        let poll = crate::models::Poll {
            id,
            guild_id,
            channel_id,
            creator_id,
            question,
            options,
            voting_method,
            created_at,
            ends_at,
            is_active,
            message_id,
        };
        
        Ok(poll)
    }
    
    // End a poll (set is_active = false)
    pub async fn end_poll(
        &self,
        poll_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            r#"
            UPDATE polls
            SET is_active = FALSE
            WHERE id = ? AND is_active = TRUE
            "#,
        )
        .bind(poll_id)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    // Get polls that have passed their end time and are still active
    pub async fn get_expired_polls(
        &self,
        now: DateTime<Utc>,
    ) -> Result<Vec<(String, String, Option<String>)>, Box<dyn std::error::Error + Send + Sync>> {
        let polls = sqlx::query(
            r#"
            SELECT id, channel_id, message_id
            FROM polls
            WHERE ends_at IS NOT NULL AND ends_at < ? AND is_active = TRUE
            "#,
        )
        .bind(now.to_rfc3339())
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| {
            (
                row.get::<String, _>("id"),
                row.get::<String, _>("channel_id"),
                row.get::<Option<String>, _>("message_id"),
            )
        })
        .collect();
        Ok(polls)
    }

    // Get active polls for a specific guild
    pub async fn get_active_polls_by_guild(
        &self,
        guild_id: &str,
    ) -> Result<Vec<Poll>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query(
            r#"
            SELECT id, question, ends_at
            FROM polls
            WHERE guild_id = ? AND is_active = TRUE
            ORDER BY created_at DESC
            "#,
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await?;

        let partial_polls = rows.into_iter().map(|row| {
            Poll {
                id: row.get("id"),
                question: row.get("question"),
                ends_at: row.get::<Option<String>, _>("ends_at").and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                guild_id: guild_id.to_string(),
                channel_id: String::new(),
                creator_id: String::new(),
                options: Vec::new(),
                voting_method: VotingMethod::Plurality,
                created_at: Utc::now(),
                is_active: true,
                message_id: None,
            }
        }).collect();

        Ok(partial_polls)
    }

    // Get recently ended polls for a specific guild
    pub async fn get_recently_ended_polls_by_guild(
        &self,
        guild_id: &str,
        limit: u32,
    ) -> Result<Vec<Poll>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query(
            r#"
            SELECT id, question, ends_at
            FROM polls
            WHERE guild_id = ? AND is_active = FALSE
            ORDER BY ends_at DESC
            LIMIT ?
            "#,
        )
        .bind(guild_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let partial_polls = rows.into_iter().map(|row| {
            Poll {
                id: row.get("id"),
                question: row.get("question"),
                ends_at: row.get::<Option<String>, _>("ends_at").and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                guild_id: guild_id.to_string(),
                channel_id: String::new(),
                creator_id: String::new(),
                options: Vec::new(),
                voting_method: VotingMethod::Plurality,
                created_at: Utc::now(),
                is_active: false,
                message_id: None,
            }
        }).collect();
        Ok(partial_polls)
    }

    // Get votes for a poll
    pub async fn get_poll_votes(
        &self,
        poll_id: &str,
    ) -> Result<Vec<crate::models::Vote>, Box<dyn std::error::Error + Send + Sync>> {
        let votes = sqlx::query(
            r#"
            SELECT user_id, poll_id, option_id, rating, timestamp
            FROM votes
            WHERE poll_id = ?
            "#,
        )
        .bind(poll_id)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| crate::models::Vote {
            user_id: row.get::<String, _>("user_id"),
            poll_id: row.get::<String, _>("poll_id"),
            option_id: row.get::<String, _>("option_id"),
            rating: row.get::<i32, _>("rating"),
            timestamp: DateTime::parse_from_rfc3339(&row.get::<String, _>("timestamp"))
                .unwrap()
                .with_timezone(&Utc),
        })
        .collect();
        
        Ok(votes)
    }

    // Get votes for a specific user and poll
    pub async fn get_user_poll_votes(
        &self,
        poll_id: &str,
        user_id: &str,
    ) -> Result<Vec<crate::models::Vote>, Box<dyn std::error::Error + Send + Sync>> {
        let votes = sqlx::query(
            r#"
            SELECT user_id, poll_id, option_id, rating, timestamp
            FROM votes
            WHERE poll_id = ? AND user_id = ?
            "#,
        )
        .bind(poll_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| crate::models::Vote {
            user_id: row.get::<String, _>("user_id"),
            poll_id: row.get::<String, _>("poll_id"),
            option_id: row.get::<String, _>("option_id"),
            rating: row.get::<i32, _>("rating"),
            timestamp: DateTime::parse_from_rfc3339(&row.get::<String, _>("timestamp"))
                .unwrap()
                .with_timezone(&Utc),
        })
        .collect();
        
        Ok(votes)
    }

    // Save a vote (replacing any existing vote for the same user, poll and option)
    pub async fn save_vote(
        &self,
        vote: &crate::models::Vote,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // First verify the poll and option exist
        let poll_exists = sqlx::query("SELECT 1 FROM polls WHERE id = ?")
            .bind(&vote.poll_id)
            .fetch_optional(&self.pool)
            .await?
            .is_some();

        if !poll_exists {
            return Err("Poll not found".into());
        }

        let option_exists = sqlx::query("SELECT 1 FROM poll_options WHERE id = ? AND poll_id = ?")
            .bind(&vote.option_id)
            .bind(&vote.poll_id)
            .fetch_optional(&self.pool)
            .await?
            .is_some();

        if !option_exists {
            return Err("Poll option not found".into());
        }

        sqlx::query(
            r#"
            INSERT INTO votes (user_id, poll_id, option_id, rating, timestamp)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(user_id, poll_id, option_id) 
            DO UPDATE SET rating = excluded.rating, timestamp = excluded.timestamp
            "#,
        )
        .bind(&vote.user_id)
        .bind(&vote.poll_id)
        .bind(&vote.option_id)
        .bind(vote.rating)
        .bind(vote.timestamp.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
