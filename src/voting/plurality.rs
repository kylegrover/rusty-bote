use crate::models::{Poll, Vote};
use crate::voting::PollResults;
use std::collections::HashMap;

pub fn calculate_results(poll: &Poll, votes: &[Vote]) -> PollResults {
    // Group votes by user to get each user's highest-rated option
    let mut user_votes: HashMap<String, Vec<(String, i32)>> = HashMap::new();
    
    // Organize votes by user
    for vote in votes {
        user_votes
            .entry(vote.user_id.clone())
            .or_insert_with(Vec::new)
            .push((vote.option_id.clone(), vote.rating));
    }
    
    // Count votes: each user's highest rated option gets their vote
    let mut vote_counts: HashMap<String, f64> = HashMap::new();
    
    // Initialize all options with 0 votes
    for option in &poll.options {
        vote_counts.insert(option.id.clone(), 0.0);
    }
    
    // Count unique voters for the summary
    let unique_voters = user_votes.len();
    
    // For each user, find their highest rated option(s) and count as vote(s)
    for (_, user_ratings) in user_votes {
        // Find the maximum rating this user gave
        let max_rating = user_ratings.iter()
            .map(|(_, rating)| *rating)
            .max()
            .unwrap_or(0);
        
        // If the user gave a non-zero rating
        if max_rating > 0 {
            // Count all options that received the max rating
            let top_options: Vec<String> = user_ratings.iter()
                .filter(|(_, rating)| *rating == max_rating)
                .map(|(option_id, _)| option_id.clone())
                .collect();
            
            // Distribute one vote among all top-rated options
            let vote_value = 1.0 / top_options.len() as f64;
            
            for option_id in top_options {
                *vote_counts.entry(option_id).or_insert(0.0) += vote_value;
            }
        }
    }
    
    // Sort options by vote count
    let mut sorted_votes: Vec<(String, f64)> = vote_counts.into_iter().collect();
    sorted_votes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    // If no votes were cast
    if unique_voters == 0 {
        return PollResults {
            winner: "No votes were cast".to_string(),
            summary: "No votes were cast in this poll.".to_string(),
            raw_results: "[]".to_string(),
        };
    }
    
    // Get the winner
    let winner_id = &sorted_votes[0].0;
    let winner_votes = sorted_votes[0].1;
    let winner_name = get_option_text(poll, winner_id);
    
    // Create a summary of the results
    let mut summary = String::new();
    
    for (option_id, votes) in &sorted_votes {
        let option_name = get_option_text(poll, option_id);
        let is_winner = option_id == winner_id;
        
        // Format the line differently for the winner
        let line = if is_winner {
            format!("**{}**: {:.1} votes ({}%)", option_name, votes, (votes * 100.0 / unique_voters as f64).round() / 10.0)
        } else {
            format!("{}: {:.1} votes ({}%)", option_name, votes, (votes * 100.0 / unique_voters as f64).round() / 10.0)
        };
        
        summary.push_str(&line);
        summary.push_str("\n");
    }
    
    summary.push_str(&format!("\n{} voters participated.", unique_voters));
    
    PollResults {
        winner: winner_name,
        summary,
        raw_results: serde_json::to_string(&sorted_votes).unwrap_or_default(),
    }
}

// Helper function to get option text from option ID
fn get_option_text(poll: &Poll, option_id: &str) -> String {
    poll.options
        .iter()
        .find(|option| option.id == *option_id)
        .map(|option| option.text.clone())
        .unwrap_or_else(|| "Unknown Option".to_string())
}
