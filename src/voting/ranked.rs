use crate::models::{Poll, Vote};
use crate::voting::{PollResults, VoteCount};
use std::collections::{HashMap, HashSet};

pub fn calculate_results(poll: &Poll, votes: &[Vote]) -> PollResults {
    // Group votes by user
    let mut user_rankings: HashMap<String, HashMap<String, i32>> = HashMap::new();
    let mut option_text: HashMap<String, String> = HashMap::new();
    
    // Store option text for reference
    for option in &poll.options {
        option_text.insert(option.id.clone(), option.text.clone());
    }
    
    // Collect all rankings
    for vote in votes {
        if vote.rating > 0 {  // Only consider options that were ranked
            user_rankings
                .entry(vote.user_id.clone())
                .or_insert_with(HashMap::new)
                .insert(vote.option_id.clone(), vote.rating);
        }
    }
    
    // If no votes, return early
    if user_rankings.is_empty() {
        return PollResults {
            winner: "No winner".to_string(),
            summary: "No votes were cast.".to_string(),
            winner_id: "".to_string(),
            raw_results: Vec::new(),
        };
    }
    
    // Calculate results using instant-runoff voting
    let mut eliminated = HashSet::new();
    let total_voters = user_rankings.len();
    let mut round = 1;
    let mut summary = String::new();
    
    // Iteratively eliminate lowest-ranked candidates until someone has a majority
    loop {
        // Count first preferences for each candidate that hasn't been eliminated
        let mut first_preferences: HashMap<String, i32> = HashMap::new();
        
        // For each voter, find their highest-ranked non-eliminated candidate
        for user_votes in user_rankings.values() {
            let mut best_option = None;
            let mut best_rank = std::i32::MAX;
            
            // Find the highest ranked option that hasn't been eliminated
            for (option_id, rank) in user_votes {
                if !eliminated.contains(option_id) && *rank < best_rank {
                    best_option = Some(option_id);
                    best_rank = *rank;
                }
            }
            
            // Count this as a first preference
            if let Some(option_id) = best_option {
                *first_preferences.entry(option_id.clone()).or_insert(0) += 1;
            }
        }
        
        // Build vote counts for this round
        let mut round_counts: Vec<VoteCount> = first_preferences
            .iter()
            .map(|(option_id, count)| {
                VoteCount {
                    option_id: option_id.clone(),
                    option_text: option_text.get(option_id).cloned().unwrap_or_default(),
                    score: *count as f64,
                    rank: 0, // Will set after sorting
                }
            })
            .collect();
        
        // Sort by score (highest first)
        round_counts.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Assign ranks for this round
        for (i, count) in round_counts.iter_mut().enumerate() {
            count.rank = i + 1;
        }
        
        // Add round results to summary
        summary.push_str(&format!("**Round {}**\n", round));
        for count in &round_counts {
            let percentage = if total_voters > 0 {
                (count.score as f64 / total_voters as f64) * 100.0
            } else {
                0.0
            };
            
            summary.push_str(&format!(
                "{}: {} votes ({:.1}%)\n",
                count.option_text, count.score, percentage
            ));
        }
        summary.push('\n');
        
        // Check if we have a majority winner
        if !round_counts.is_empty() && round_counts[0].score > (total_voters as f64 / 2.0) {
            // We have a winner with majority
            let winner_id = round_counts[0].option_id.clone();
            let winner_text = round_counts[0].option_text.clone();
            let winner_votes = round_counts[0].score as i32;
            
            summary.push_str(&format!(
                "{} wins with {} votes ({:.1}% of {}).",
                winner_text, 
                winner_votes,
                (winner_votes as f64 / total_voters as f64) * 100.0,
                total_voters
            ));
            
            return PollResults {
                winner: format!("{} ({} votes)", winner_text, winner_votes),
                summary,
                winner_id,
                raw_results: round_counts,
            };
        }
        
        // If we only have one or zero options left, this is the winner
        if round_counts.len() <= 1 {
            if round_counts.is_empty() {
                return PollResults {
                    winner: "No winner".to_string(),
                    summary,
                    winner_id: "".to_string(),
                    raw_results: Vec::new(),
                };
            } else {
                let winner_id = round_counts[0].option_id.clone();
                let winner_text = round_counts[0].option_text.clone();
                let winner_votes = round_counts[0].score as i32;
                
                summary.push_str(&format!(
                    "{} is the last remaining candidate with {} votes.",
                    winner_text, winner_votes
                ));
                
                return PollResults {
                    winner: format!("{} ({} votes)", winner_text, winner_votes),
                    summary,
                    winner_id,
                    raw_results: round_counts,
                };
            }
        }
        
        // Eliminate the lowest-ranked candidate
        let to_eliminate = round_counts.last().unwrap().option_id.clone();
        let eliminated_text = option_text.get(&to_eliminate).cloned().unwrap_or_default();
        
        summary.push_str(&format!("Eliminating: {}\n\n", eliminated_text));
        eliminated.insert(to_eliminate);
        
        round += 1;
    }
}
