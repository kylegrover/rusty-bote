use crate::models::{Poll, Vote};
use crate::voting::{PollResults, VoteCount};
use std::collections::HashMap;

pub fn calculate_results(poll: &Poll, votes: &[Vote]) -> PollResults {
    // Group votes by user and option
    let mut user_votes: HashMap<String, HashMap<String, i32>> = HashMap::new();
    
    for vote in votes {
        user_votes
            .entry(vote.user_id.clone())
            .or_insert_with(HashMap::new)
            .insert(vote.option_id.clone(), vote.rating);
    }
    
    // Calculate scores for each option
    let mut option_scores: HashMap<String, f64> = HashMap::new();
    let mut option_text: HashMap<String, String> = HashMap::new();
    
    // Initialize scores to 0
    for option in &poll.options {
        option_scores.insert(option.id.clone(), 0.0);
        option_text.insert(option.id.clone(), option.text.clone());
    }
    
    // Sum up all scores
    for user_vote_map in user_votes.values() {
        for (option_id, rating) in user_vote_map {
            if let Some(score) = option_scores.get_mut(option_id) {
                *score += *rating as f64;
            }
        }
    }
    
    // Build vote counts
    let mut vote_counts: Vec<VoteCount> = option_scores
        .iter()
        .map(|(option_id, score)| {
            VoteCount {
                option_id: option_id.clone(),
                option_text: option_text.get(option_id).cloned().unwrap_or_default(),
                score: *score,
                rank: 0, // Will set this after sorting
            }
        })
        .collect();
    
    // Sort by score (highest first)
    vote_counts.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    
    // Assign ranks
    for (i, count) in vote_counts.iter_mut().enumerate() {
        count.rank = i + 1;
    }
    
    // STAR voting also needs a runoff between the top two candidates
    if vote_counts.len() >= 2 {
        // Get the top two options
        let first_id = vote_counts[0].option_id.clone();
        let second_id = vote_counts[1].option_id.clone();
        
        // Count voters who prefer first over second
        let mut prefer_first = 0;
        let mut prefer_second = 0;
        
        for user_votes in user_votes.values() {
            let first_rating = user_votes.get(&first_id).cloned().unwrap_or(0);
            let second_rating = user_votes.get(&second_id).cloned().unwrap_or(0);
            
            if first_rating > second_rating {
                prefer_first += 1;
            } else if second_rating > first_rating {
                prefer_second += 1;
            }
        }
        
        // Determine winner based on preferences
        let winner_id = if prefer_first >= prefer_second { first_id } else { second_id };
        let winner_text = option_text.get(&winner_id).cloned().unwrap_or_default();
        let winner_score = option_scores.get(&winner_id).cloned().unwrap_or(0.0);
        
        // Build summary text
        let mut summary = String::new();
        
        // First show the total scores
        summary.push_str("**Scores**\n");
        for count in &vote_counts {
            summary.push_str(&format!("{}: {} stars\n", count.option_text, count.score));
        }
        
        // Then show the runoff results
        summary.push_str("\n**Runoff**\n");
        let first_text = option_text.get(&vote_counts[0].option_id).cloned().unwrap_or_default();
        let second_text = option_text.get(&vote_counts[1].option_id).cloned().unwrap_or_default();
        summary.push_str(&format!(
            "{}: {} votes\n{}: {} votes\n\n",
            first_text, prefer_first,
            second_text, prefer_second
        ));
        
        let total_voters = user_votes.len();
        summary.push_str(&format!("Total voters: {}", total_voters));
        
        PollResults {
            winner: format!("{} ({} stars)", winner_text, winner_score),
            summary,
            winner_id,
            raw_results: vote_counts,
        }
    } else if !vote_counts.is_empty() {
        // If there's only one option, it's the winner
        let winner_id = vote_counts[0].option_id.clone();
        let winner_text = vote_counts[0].option_text.clone();
        let winner_score = vote_counts[0].score;
        
        // Simple summary
        let summary = format!("{} is the winner with {} stars.", winner_text, winner_score);
        
        PollResults {
            winner: format!("{} ({} stars)", winner_text, winner_score),
            summary,
            winner_id,
            raw_results: vote_counts,
        }
    } else {
        // No options or votes
        PollResults {
            winner: "No winner".to_string(),
            summary: "No votes were cast.".to_string(),
            winner_id: "".to_string(),
            raw_results: Vec::new(),
        }
    }
}
