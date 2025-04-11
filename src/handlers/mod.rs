mod vote;

use crate::db::Database;
use serenity::model::application::interaction::message_component::MessageComponentInteraction;
use serenity::prelude::*;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref VOTE_REGEX: Regex = Regex::new(r"vote_(.+)_(.+)_(\d+)").unwrap();
}

pub async fn handle_component(
    database: &Database,
    ctx: &Context,
    component: &MessageComponentInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let custom_id = component.data.custom_id.as_str();
    
    // Check if it's a vote button
    if let Some(captures) = VOTE_REGEX.captures(custom_id) {
        if captures.len() >= 4 {
            let poll_id = captures.get(1).unwrap().as_str();
            let option_id = captures.get(2).unwrap().as_str();
            let rating = captures.get(3).unwrap().as_str().parse::<i32>()?;
            
            return vote::handle_star_vote(database, ctx, component, poll_id, option_id, rating).await;
        }
    }
    
    // Handle other component types
    match custom_id {
        "vote_button" => vote::handle_vote_button(database, ctx, component).await,
        _ => {
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
