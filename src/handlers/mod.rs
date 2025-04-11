mod vote;

use crate::db::Database;
use serenity::model::application::interaction::message_component::MessageComponentInteraction;
use serenity::prelude::*;
use regex::Regex;
use lazy_static::lazy_static;
use log::{info, error};

lazy_static! {
    // Updated pattern to match select menu custom_id format
    static ref VOTE_REGEX: Regex = Regex::new(r"vote_(.+)_(.+)").unwrap();
}

pub async fn handle_component(
    database: &Database,
    ctx: &Context,
    component: &MessageComponentInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let custom_id = component.data.custom_id.as_str();
    info!("Handling component interaction: {}", custom_id);
    
    // Check if it's a vote select menu
    if let Some(captures) = VOTE_REGEX.captures(custom_id) {
        if captures.len() >= 3 {
            let poll_id = captures.get(1).unwrap().as_str();
            let option_id = captures.get(2).unwrap().as_str();
            
            // For select menus, we get the rating from the selected values
            if let Some(value) = component.data.values.get(0) {
                let rating = value.parse::<i32>()?;
                
                info!("Matched vote select menu: poll_id={}, option_id={}, rating={}", poll_id, option_id, rating);
                return vote::handle_star_vote(database, ctx, component, poll_id, option_id, rating).await;
            }
        }
    }
    
    match custom_id {
        "vote_button" => {
            info!("Handling vote button interaction");
            vote::handle_vote_button(database, ctx, component).await
        },
        _ => {
            info!("Unknown component interaction: {}", custom_id);
            component
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(serenity::model::application::interaction::InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message
                                .content("Unknown component interaction")
                                .ephemeral(true)
                        })
                })
                .await?;
            Ok(())
        }
    }
}
