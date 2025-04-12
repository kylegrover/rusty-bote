use crate::models::{Poll, Vote};
use crate::voting::PollResults;
use std::collections::HashMap;

pub fn calculate_results(poll: &Poll, votes: &[Vote]) -> PollResults {
    // For approval voting, we consider any option rated 3 or higher (out of 5) as "approved"
    const APPROVAL_THRESHOLD: i32 = 3;
    
    // Track approvals for each option
    let mut approval_counts: HashMap<String, i32> = HashMap::new();
    
    // Initialize all options with 0 approvals
    for option in &poll.options {
        approval_counts.insert(option.id.clone(), 0);
    }
    
    // Count unique voters for the summary
    let unique_voters: std::collections::HashSet<String> = votes.iter()
        .map(|vote| vote.user_id.clone())
        .collect();
    
    // Count approvals
    for vote in votes {
        if vote.rating >= APPROVAL_THRESHOLD {
            *approval_counts.entry(vote.option_id.clone()).or_insert(0) += 1;
        }
    }
    
    // Sort options by approval count
    let mut sorted_approvals: Vec<(String, i32)> = approval_counts.into_iter().collect();
    sorted_approvals.sort_by(|a, b| b.1.cmp(&a.1));
    
    // If no votes were cast
    if unique_voters.is_empty() {
        return PollResults {
            winner: "No votes were cast".to_string(),
            summary: "No votes were cast in this poll.".to_string(),
            raw_results: "[]".to_string(),
        };
    }
    
    // Get the winner
    let winner_id = &sorted_approvals[0].0;
    let winner_approvals = sorted_approvals[0].1;
    let winner_name = get_option_text(poll, winner_id);
    
    // Create a summary of the results
    let mut summary = String::new();
    
    summary.push_str(&format!("Options rated {} or higher stars count as approved.\n\n", APPROVAL_THRESHOLD));
    
    for (option_id, approvals) in &sorted_approvals {
        let option_name = get_option_text(poll, option_id);
        let is_winner = option_id == winner_id;
        let approval_percentage = if unique_voters.len() > 0 {
            (*approvals as f64 * 100.0 / unique_voters.len() as f64).round() / 10.0
        } else {
            0.0
        };
        
        // Format the line differently for the winner
        let line = if is_winner {
            format!("**{}**: {} approvals ({}%)", option_name, approvals, approval_percentage)
        } else {
            format!("{}: {} approvals ({}%)", option_name, approvals, approval_percentage)
        };
        
        summary.push_str(&line);
        summary.push_str("\n");
    }
    
    summary.push_str(&format!("\n{} voters participated.", unique_voters.len()));
    
    PollResults {
        winner: winner_name,
        summary,
        raw_results: serde_json::to_string(&sorted_approvals).unwrap_or_default(),
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
