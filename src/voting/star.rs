use crate::models::{Poll, Vote};
use crate::voting::PollResults;
use std::collections::HashMap;

pub fn calculate_results(poll: &Poll, votes: &[Vote]) -> PollResults {
    // Step 1: Calculate total scores for each option
    let mut scores: HashMap<String, f64> = HashMap::new();
    let _total_votes = 0;
    
    // Initialize scores to 0
    for option in &poll.options {
        scores.insert(option.id.clone(), 0.0);
    }
    
    // Count unique voters
    let unique_voters: std::collections::HashSet<String> = votes.iter()
        .map(|vote| vote.user_id.clone())
        .collect();
    
    // Calculate scores
    for vote in votes {
        *scores.entry(vote.option_id.clone()).or_insert(0.0) += vote.rating as f64;
    }
    
    // Find top two scoring candidates
    let mut sorted_scores: Vec<(String, f64)> = scores.into_iter().collect();
    sorted_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    // If there are less than 2 candidates, just return the top scorer
    if sorted_scores.len() < 2 {
        let winner_id = if sorted_scores.is_empty() { 
            "No votes were cast".to_string()
        } else {
            sorted_scores[0].0.clone()
        };
        
        let winner_name = if winner_id == "No votes were cast" {
            winner_id.clone()
        } else {
            get_option_text(poll, &winner_id)
        };
        
        return PollResults {
            winner: winner_name,
            summary: format!("No runoff needed. {} voters participated.", unique_voters.len()),
            raw_results: serde_json::to_string(&sorted_scores).unwrap_or_default(),
        };
    }
    
    // Get the top two candidates for runoff
    let finalist1_id = sorted_scores[0].0.clone();
    let finalist2_id = sorted_scores[1].0.clone();
    
    // Step 2: Runoff between top two candidates
    let mut prefer_finalist1 = 0;
    let mut prefer_finalist2 = 0;
    
    // Group votes by user
    let mut user_votes: HashMap<String, HashMap<String, i32>> = HashMap::new();
    
    for vote in votes {
        user_votes
            .entry(vote.user_id.clone())
            .or_insert_with(HashMap::new)
            .insert(vote.option_id.clone(), vote.rating);
    }
    
    // For each user, determine preference between finalists
    for (_, user_vote) in user_votes {
        let score1 = *user_vote.get(&finalist1_id).unwrap_or(&0);
        let score2 = *user_vote.get(&finalist2_id).unwrap_or(&0);
        
        if score1 > score2 {
            prefer_finalist1 += 1;
        } else if score2 > score1 {
            prefer_finalist2 += 1;
        }
        // If scores are equal, it's a tie for this voter
    }
    
    // Determine winner
    let (winner_id, _runoff_winner_votes, _runoff_loser_votes) = if prefer_finalist1 >= prefer_finalist2 {
        (finalist1_id.clone(), prefer_finalist1, prefer_finalist2)
    } else {
        (finalist2_id.clone(), prefer_finalist2, prefer_finalist1)
    };
    
    let winner_name = get_option_text(poll, &winner_id);
    let finalist1_name = get_option_text(poll, &finalist1_id);
    let finalist2_name = get_option_text(poll, &finalist2_id);
    let finalist1_score = sorted_scores[0].1;
    let finalist2_score = sorted_scores[1].1;
    
    // Create summary
    let summary = format!(
        "**Scoring Round:**\n{}: {:.1} stars\n{}: {:.1} stars\n\n**Runoff Round:**\n{}: {} votes\n{}: {} votes\n\n{} voters participated.",
        finalist1_name, finalist1_score,
        finalist2_name, finalist2_score,
        if finalist1_id == winner_id { "**".to_string() + &finalist1_name + "**" } else { finalist1_name.clone() },
        prefer_finalist1,
        if finalist2_id == winner_id { "**".to_string() + &finalist2_name + "**" } else { finalist2_name.clone() },
        prefer_finalist2,
        unique_voters.len()
    );
    
    PollResults {
        winner: winner_name,
        summary,
        raw_results: serde_json::to_string(&sorted_scores).unwrap_or_default(),
    }
}

fn get_option_text(poll: &Poll, option_id: &str) -> String {
    poll.options
        .iter()
        .find(|option| option.id == *option_id)
        .map(|option| option.text.clone())
        .unwrap_or_else(|| "Unknown Option".to_string())
}
