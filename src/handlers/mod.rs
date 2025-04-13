mod vote;

use crate::db::Database;
use crate::models::Poll; // Add import for Poll
use serenity::model::application::interaction::{Interaction, InteractionResponseType}; // Added Interaction, InteractionResponseType
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::message_component::MessageComponentInteraction;
use serenity::prelude::*;
use log::{info, warn, error}; // Added info and warn

// Handle slash commands
pub async fn handle_command(
    database: &Database,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Received command: {}", command.data.name);
    match command.data.name.as_str() {
        "poll" => crate::commands::poll::handle_poll_command(database, ctx, command).await?,
        // Add other top-level commands here if any
        _ => {
            // Respond with an error for unknown commands
             command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content("Unknown command").ephemeral(true))
            }).await?;
        }
    }
    Ok(())
}

// Helper function to parse poll_id from custom_id
fn parse_poll_id_from_custom_id(custom_id: &str) -> Option<String> {
    let parts: Vec<&str> = custom_id.split('_').collect();
    if custom_id.starts_with("vote_") && parts.len() >= 3 {
        Some(parts[1].to_string())
    } else if custom_id.starts_with("star_") && parts.len() >= 4 {
        Some(parts[1].to_string())
    } else if custom_id.starts_with("plurality_") && parts.len() >= 3 {
        Some(parts[1].to_string())
    } 
    // Moved approval_submit_ condition before generic approval_ to avoid mis-parsing
    else if custom_id.starts_with("approval_submit_") && parts.len() >= 3 {
        Some(parts[2].to_string())
    } else if custom_id.starts_with("approval_") && parts.len() >= 3 {
        Some(parts[1].to_string())
    } else if custom_id.starts_with("rank_up_") && parts.len() >= 4 {
        Some(parts[2].to_string())
    } else if custom_id.starts_with("rank_down_") && parts.len() >= 4 {
        Some(parts[2].to_string())
    } else if custom_id.starts_with("rank_remove_") && parts.len() >= 4 {
        Some(parts[2].to_string())
    } else if custom_id.starts_with("rank_submit_") && parts.len() >= 3 {
        Some(parts[2].to_string())
    } else {
        None
    }
}

// Main component handler that routes to specific handlers based on the component ID
pub async fn handle_component(
    database: &Database,
    ctx: &Context,
    component: &MessageComponentInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let custom_id = &component.data.custom_id;
    info!("Received component interaction: {}", custom_id); // Log component clicks

    // --- Poll ID Identification ---
    let poll_id_opt: Option<String> = if custom_id == "vote_button" {
        // For the initial button, get ID from the embed
        component
            .message
            .embeds
            .get(0)
            .and_then(|embed| {
                embed.fields.iter().find(|field| field.name == "Poll ID").map(|field| field.value.clone())
            })
    } else {
        // For subsequent interactions, parse from custom_id
        parse_poll_id_from_custom_id(custom_id)
    };

    // --- Poll Fetching and Status Check ---
    let poll: Option<Poll> = if let Some(ref poll_id) = poll_id_opt {
        match database.get_poll(poll_id).await {
            Ok(p) => Some(p),
            Err(e) => {
                // Log the actual poll_id that failed
                error!("Failed to fetch poll ID '{}' for component interaction '{}': {}", poll_id, custom_id, e);
                component.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content("Error fetching poll data.").ephemeral(true))
                }).await?;
                return Ok(()); // Stop processing if fetch fails
            }
        }
    } else {
        // Handle cases where poll_id couldn't be determined but might be needed
        if !custom_id.starts_with("label_") && !custom_id.starts_with("rank_label_") {
             error!("Could not determine Poll ID for component interaction: {}", custom_id);
             component.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content("Error identifying the poll for this action.").ephemeral(true))
            }).await?;
             return Ok(()); // Stop processing
        }
        None // Poll not needed for label clicks
    };

    // Check if poll is active (only if poll was fetched)
    if let Some(ref p) = poll {
        if !p.is_active && custom_id != "vote_button" { // Allow clicking vote_button on inactive poll to show message
             component.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content("This poll has ended.").ephemeral(true))
            }).await?;
            return Ok(()); // Stop processing if poll is inactive for voting actions
        }
    } else if custom_id != "vote_button" && !custom_id.starts_with("label_") && !custom_id.starts_with("rank_label_") {
        // If poll is None here, it means poll_id_opt was None earlier, error already sent.
        // This condition is likely redundant but safe.
        return Ok(());
    }


    // --- Interaction Routing ---
    if custom_id == "vote_button" {
        if let Some(p) = poll {
            // Pass the fetched poll object AND database
            vote::handle_vote_button(database, ctx, component, &p).await?; // Added database argument
        } else {
             // This case should be rare, means poll_id was in embed but fetch failed (handled above)
             error!("Poll object unavailable for vote_button interaction after successful ID extraction.");
             // Send generic error
             component.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content("An error occurred processing this action.").ephemeral(true))
            }).await?;
        }
    }
    // Handle STAR voting ratings (now recognizing "star_" IDs)
    else if custom_id.starts_with("star_") {
        // Format: star_<poll_id>_<option_id>_<rating>
        let parts: Vec<&str> = custom_id.split('_').collect();
        if parts.len() >= 4 {
            let poll_id = parts[1];
            let option_id = parts[2];
            if let Ok(rating) = parts[3].parse::<i32>() {
                vote::handle_star_vote(database, ctx, component, poll_id, option_id, rating).await?;
            } else {
                error!("Failed to parse rating in star vote custom_id: {}", custom_id);
            }
        } else {
            error!("Invalid custom_id format for star vote: {}", custom_id);
        }
    }
    else if custom_id.starts_with("vote_") {
        // Extract poll_id, option_id, and rating from the custom_id
        // Format: vote_<poll_id>_<option_id>
        let parts: Vec<&str> = custom_id.split('_').collect();
        // poll_id is already parsed and poll fetched/checked
        if parts.len() >= 3 {
            let poll_id = parts[1]; // Still need poll_id string for the function call
            let option_id = parts[2];

            // The selected value is in the component data
            // Check if values exist and get the first one
            if !component.data.values.is_empty() {
                if let Some(rating_str) = component.data.values.first() {
                    if let Ok(rating) = rating_str.parse::<i32>() {
                        vote::handle_star_vote(database, ctx, component, poll_id, option_id, rating).await?;
                    } else { error!("Failed to parse rating: {}", rating_str); }
                } else { error!("Rating value missing in component data values."); }
            } else { error!("Component data values are empty for star vote."); }
        } else { error!("Invalid custom_id format for star vote: {}", custom_id); }
    }
    // Handle plurality voting
    else if custom_id.starts_with("plurality_") {
        // Format: plurality_<poll_id>_<option_id>
        let parts: Vec<&str> = custom_id.split('_').collect();
        if parts.len() >= 3 {
            let poll_id = parts[1];
            let option_id = parts[2];
            // Pass the fetched poll object to avoid re-fetching
            if let Some(p) = poll {
                vote::handle_plurality_vote(database, ctx, component, poll_id, option_id, &p).await?;
            } else { error!("Poll object unavailable for plurality vote."); /* Handle error */ }
        } else { error!("Invalid custom_id format for plurality vote: {}", custom_id); }
    }
    // Handle approval voting toggle
    else if custom_id.starts_with("approval_") && !custom_id.starts_with("approval_submit_") {
        // Format: approval_<poll_id>_<option_id>_<current_value>
        let parts: Vec<&str> = custom_id.split('_').collect();
        if parts.len() >= 4 {
            let poll_id = parts[1];
            let option_id = parts[2];
            let current_value: i32 = parts[3].parse().unwrap_or(0);
            // Pass the fetched poll object
            if let Some(p) = poll {
                vote::handle_approval_vote_toggle(ctx, component, poll_id, option_id, current_value, &p).await?;
            } else { error!("Poll object unavailable for approval toggle."); /* Handle error */ }
        } else { error!("Invalid custom_id format for approval toggle: {}", custom_id); }
    }
    // Handle approval voting submit
    else if custom_id.starts_with("approval_submit_") {
        // Format: approval_submit_<poll_id>
        let parts: Vec<&str> = custom_id.split('_').collect();
        if parts.len() >= 3 {
            let poll_id = parts[2];
            vote::handle_approval_submit(database, ctx, component, poll_id).await?;
        } else { error!("Invalid custom_id format for approval submit: {}", custom_id); }
    }
    // Handle ranked choice voting actions
    else if custom_id.starts_with("rank_up_") ||
            custom_id.starts_with("rank_down_") ||
            custom_id.starts_with("rank_remove_") {
        let action = if custom_id.starts_with("rank_up_") { "up" }
                     else if custom_id.starts_with("rank_down_") { "down" }
                     else { "remove" };

        // Format: rank_(up|down|remove)_<poll_id>_<option_id>
        let parts: Vec<&str> = custom_id.split('_').collect();
        if parts.len() >= 4 {
            let poll_id = parts[2];
            let option_id = parts[3];
            // Pass the fetched poll object
            if let Some(p) = poll {
                vote::handle_rank_action(database, ctx, component, poll_id, option_id, action, &p).await?;
            } else { error!("Poll object unavailable for rank action."); /* Handle error */ }
        } else { error!("Invalid custom_id format for rank action: {}", custom_id); }
    }
    // Handle ranked choice submit
    else if custom_id.starts_with("rank_submit_") {
        // Format: rank_submit_<poll_id>
        let parts: Vec<&str> = custom_id.split('_').collect();
        if parts.len() >= 3 {
            let poll_id = parts[2];
            vote::handle_rank_submit(database, ctx, component, poll_id).await?;
        } else { error!("Invalid custom_id format for rank submit: {}", custom_id); }
    } else if custom_id.starts_with("label_") || custom_id.starts_with("rank_label_") {
         // Ignore clicks on disabled label buttons, acknowledge to prevent "Interaction failed"
         component.create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::DeferredUpdateMessage)
        }).await?;
    } else {
         warn!("Unhandled component custom_id: {}", custom_id);
          component.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content("Unknown button action.").ephemeral(true))
        }).await?;
    }

    Ok(())
}

// Add Interaction handler entry point
pub async fn handle_interaction(
    database: &Database,
    ctx: &Context,
    interaction: Interaction,
) {
    let result = match interaction {
        Interaction::ApplicationCommand(command) => {
            handle_command(database, ctx, &command).await
        }
        Interaction::MessageComponent(component) => {
            handle_component(database, ctx, &component).await
        }
        // Handle other interaction types like ModalSubmit if needed
        _ => {
            warn!("Unhandled interaction type: {:?}", interaction.kind());
            Ok(())
        }
    };

    if let Err(why) = result {
        error!("Interaction handler error: {:?}", why);
        // Optionally, try to inform the user about the error if possible
    }
}
