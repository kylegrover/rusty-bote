use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Poll {
    pub id: String,
    pub guild_id: String,
    pub channel_id: String,
    pub creator_id: String,
    pub question: String,
    pub options: Vec<PollOption>,
    pub voting_method: VotingMethod,
    pub created_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollOption {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VotingMethod {
    Star,
    Plurality,
    Ranked,
    Approval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub user_id: String,
    pub poll_id: String,
    pub ratings: Vec<OptionRating>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionRating {
    pub option_id: String,
    pub rating: i32,
}

impl Poll {
    pub fn new(
        guild_id: String,
        channel_id: String,
        creator_id: String,
        question: String,
        options: Vec<String>,
        voting_method: VotingMethod,
        duration_minutes: Option<i64>,
    ) -> Self {
        let now = Utc::now();
        let ends_at = duration_minutes.map(|mins| now + chrono::Duration::minutes(mins));
        
        let options = options
            .into_iter()
            .map(|text| PollOption {
                id: Uuid::new_v4().to_string(),
                text,
            })
            .collect();

        Self {
            id: Uuid::new_v4().to_string(),
            guild_id,
            channel_id,
            creator_id,
            question,
            options,
            voting_method,
            created_at: now,
            ends_at,
            is_active: true,
        }
    }
}
