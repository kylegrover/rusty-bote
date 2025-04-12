use crate::models::{Poll, Vote};
use crate::voting::PollResults;
use std::collections::{HashMap, HashSet};
use serde_json::json;

pub fn calculate_results(poll: &Poll, votes: &[Vote]) -> PollResults {
    // For ranked choice, we interpret higher ratings as higher rank
    // Group votes by user
    let mut user_votes: HashMap<String, Vec<(String, i32)>> = HashMap::new();
    
    // Organize votes by user
    for vote in votes {
        user_votes
            .entry(vote.user_id.clone())
            .or_insert_with(Vec::new)
            .push((vote.option_id.clone(), vote.rating));
    }
    
    // Convert ratings to rankings for each user
    // The higher the rating, the higher the rank
    let mut user_rankings: HashMap<String, Vec<String>> = HashMap::new();
    
    for (user_id, ratings) in user_votes {
        // Sort by rating in descending order
        let mut sorted_ratings = ratings;
        sorted_ratings.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Extract just the option IDs in order of preference
        // Only include options with ratings > 0
        let rankings: Vec<String> = sorted_ratings
            .into_iter()
            .filter(|(_, rating)| *rating > 0)
            .map(|(option_id, _)| option_id)
            .collect();
        
        if !rankings.is_empty() {
            user_rankings.insert(user_id, rankings);
        }
    }
    
    // Count unique voters
    let voters_count = user_rankings.len();
    
    // If no votes were cast
    if voters_count == 0 {
        return PollResults {
            winner: "No votes were cast".to_string(),
            summary: "No votes were cast in this poll.".to_string(),
            raw_results: "[]".to_string(),
        };
    }
    
    // Create a mapping from option ID to option text for easier reference
    let mut option_names: HashMap<String, String> = HashMap::new();
    for option in &poll.options {
        option_names.insert(option.id.clone(), option.text.clone());
    }
    
    // Begin the ranked choice algorithm
    let mut candidates: HashSet<String> = poll.options.iter().map(|o| o.id.clone()).collect();
    let mut eliminated: HashSet<String> = HashSet::new();
    let mut round = 1;
    let mut winner = None;
    let mut rounds_summary = Vec::new();
    
    // Continue until we have a winner or no candidates left
    while !candidates.is_empty() && winner.is_none() {
        // Count first-choice votes for each remaining candidate
        let mut vote_counts: HashMap<String, usize> = HashMap::new();
        for candidate in &candidates {
            vote_counts.insert(candidate.clone(), 0);
        }
        
        // Count first choices that haven't been eliminated
        for rankings in user_rankings.values() {
            for option_id in rankings {
                if candidates.contains(option_id) && !eliminated.contains(option_id) {
                    *vote_counts.entry(option_id.clone()).or_insert(0) += 1;
                    break; // Only count first choice
                }
            }
        }
        
        // Sort candidates by vote count
        let mut sorted_candidates: Vec<(String, usize)> = vote_counts.into_iter().collect();
        sorted_candidates.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by votes in descending order
        
        // Store round results for summary
        let round_result = sorted_candidates.clone();
        rounds_summary.push((round, round_result));
        
        // Check if any candidate has majority
        if !sorted_candidates.is_empty() && sorted_candidates[0].1 > voters_count / 2 {
            winner = Some(sorted_candidates[0].0.clone());
        } else if sorted_candidates.len() <= 2 {
            // If we're down to two candidates, the one with more votes wins
            if !sorted_candidates.is_empty() {
                winner = Some(sorted_candidates[0].0.clone());
            }
        } else {
            // Eliminate candidate with fewest votes - FIX: Clone loser before moving
            let loser = sorted_candidates.last().unwrap().0.clone();
            eliminated.insert(loser.clone()); // Clone when inserting
            candidates.remove(&loser);
        }
        
        round += 1;
    }
    
    // Prepare the summary
    let mut summary = String::from("Ranked Choice Voting Results:\n\n");
    
    // FIX: Iterate over reference to avoid moving rounds_summary
    for (round_num, results) in &rounds_summary {
        summary.push_str(&format!("Round {}:\n", round_num));
        
        for (option_id, votes) in results {
            let option_name = option_names.get(option_id).unwrap_or(option_id);
            let percentage = (*votes as f64 * 100.0) / (voters_count as f64);
            
            // Highlight the winner in the final round
            if Some(option_id.clone()) == winner && *round_num == rounds_summary.len() {
                summary.push_str(&format!("**{}**: {} votes ({:.1}%)\n", option_name, votes, percentage));
            } else {
                summary.push_str(&format!("{}: {} votes ({:.1}%)\n", option_name, votes, percentage));
            }
        }
        summary.push_str("\n");
    }
    
    summary.push_str(&format!("\n{} voters participated.", voters_count));
    
    // Get the winner's name - FIX: Use ref pattern to avoid moving from winner
    let winner_name = match winner {
        Some(ref winner_id) => option_names.get(winner_id).cloned().unwrap_or_else(|| "Unknown Option".to_string()),
        None => "No winner determined".to_string(),
    };
    
    // Prepare the raw results for potential further processing
    let raw_results = json!({
        "rounds": &rounds_summary, // FIX: Reference rounds_summary here
        "winner": &winner,         // FIX: Reference winner here
        "voters_count": voters_count
    });
    
    PollResults {
        winner: winner_name,
        summary,
        raw_results: serde_json::to_string(&raw_results).unwrap_or_default(),
    }
}
