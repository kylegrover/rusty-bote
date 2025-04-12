mod vote;

use crate::db::Database;
use serenity::model::application::interaction::message_component::MessageComponentInteraction;
use serenity::prelude::*;
use log::error;

// Main component handler that routes to specific handlers based on the component ID
pub async fn handle_component(
    database: &Database,
    ctx: &Context,
    component: &MessageComponentInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let custom_id = &component.data.custom_id;
    
    if custom_id == "vote_button" {
        // Handle the main vote button click
        vote::handle_vote_button(database, ctx, component).await?;
    } 
    // Handle STAR voting ratings
    else if custom_id.starts_with("vote_") {
        // Extract poll_id, option_id, and rating from the custom_id
        // Format: vote_<poll_id>_<option_id>
        let parts: Vec<&str> = custom_id.split('_').collect();
        if parts.len() >= 3 {
            let poll_id = parts[1];
            let option_id = parts[2];
            
            // The selected value is in the component data
            // Check if values exist and get the first one
            if !component.data.values.is_empty() {
                if let Some(rating_str) = component.data.values.first() {
                    if let Ok(rating) = rating_str.parse::<i32>() {
                        vote::handle_star_vote(database, ctx, component, poll_id, option_id, rating).await?;
                    }
                }
            }
        }
    } 
    // Handle plurality voting
    else if custom_id.starts_with("plurality_") {
        // Format: plurality_<poll_id>_<option_id>
        let parts: Vec<&str> = custom_id.split('_').collect();
        if parts.len() >= 3 {
            let poll_id = parts[1];
            let option_id = parts[2];
            vote::handle_plurality_vote(database, ctx, component, poll_id, option_id).await?;
        }
    }
    // Handle approval voting toggle
    else if custom_id.starts_with("approval_") && !custom_id.starts_with("approval_submit_") {
        // Format: approval_<poll_id>_<option_id>_<current_value>
        let parts: Vec<&str> = custom_id.split('_').collect();
        if parts.len() >= 4 {
            let poll_id = parts[1];
            let option_id = parts[2];
            let current_value: i32 = parts[3].parse().unwrap_or(0);
            vote::handle_approval_vote_toggle(database, ctx, component, poll_id, option_id, current_value).await?;
        }
    }
    // Handle approval voting submit
    else if custom_id.starts_with("approval_submit_") {
        // Format: approval_submit_<poll_id>
        let parts: Vec<&str> = custom_id.split('_').collect();
        if parts.len() >= 3 {
            let poll_id = parts[2];
            vote::handle_approval_submit(database, ctx, component, poll_id).await?;
        }
    }
    // Handle ranked choice voting actions
    else if custom_id.starts_with("rank_up_") || 
            custom_id.starts_with("rank_down_") || 
            custom_id.starts_with("rank_remove_") {
        let action = if custom_id.starts_with("rank_up_") {
            "up"
        } else if custom_id.starts_with("rank_down_") {
            "down"
        } else {
            "remove"
        };
        
        // Format: rank_(up|down|remove)_<poll_id>_<option_id>
        let parts: Vec<&str> = custom_id.split('_').collect();
        if parts.len() >= 4 {
            let poll_id = parts[2];
            let option_id = parts[3];
            vote::handle_rank_action(database, ctx, component, poll_id, option_id, action).await?;
        }
    }
    // Handle ranked choice submit
    else if custom_id.starts_with("rank_submit_") {
        // Format: rank_submit_<poll_id>
        let parts: Vec<&str> = custom_id.split('_').collect();
        if parts.len() >= 3 {
            let poll_id = parts[2];
            vote::handle_rank_submit(database, ctx, component, poll_id).await?;
        }
    }

    Ok(())
}
