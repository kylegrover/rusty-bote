use crate::db::Database;
use serenity::model::application::interaction::{
    message_component::MessageComponentInteraction, InteractionResponseType,
};
use serenity::prelude::*;
use chrono::Utc;
use log::{info, error};

pub async fn handle_vote_button(
    database: &Database,
    ctx: &Context,
    component: &MessageComponentInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Extract poll_id from the message
    let poll_id = component
        .message
        .embeds
        .get(0)
        .and_then(|embed| {
            embed.fields.iter().find(|field| field.name == "Poll ID").map(|field| field.value.clone())
        })
        .ok_or("Could not find poll ID in message")?;
    
    // Fetch the poll from the database
    let poll = database.get_poll(&poll_id).await?;
    
    // Create voting interface based on voting method
    match poll.voting_method {
        crate::models::VotingMethod::Star => {
            // For STAR voting, respond with an ephemeral message showing voting options
            component
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message
                                .ephemeral(true)
                                .content(format!("**{}**\nRate each option from 0 to 5 stars:", poll.question))
                                .components(|c| {
                                    // Create a select menu for each option
                                    for option in &poll.options {
                                        c.create_action_row(|row| {
                                            row.create_select_menu(|menu| {
                                                menu.custom_id(format!("vote_{}_{}", poll_id, option.id))
                                                    .placeholder(format!("Rate: {}", option.text))
                                                    .options(|opts| {
                                                        // Add option for 0 stars
                                                        opts.create_option(|opt| 
                                                            opt.label("0 stars").value("0").default_selection(true)
                                                        );
                                                        
                                                        for i in 1..=5 {
                                                            // Add options for 1 to 5 stars
                                                            opts.create_option(|opt| 
                                                                opt.label(format!("{} ", "⭐".repeat(i))).value(i.to_string())
                                                            );
                                                        }
                                                        
                                                        opts
                                                    })
                                            })
                                        });
                                    }
                                    
                                    c
                                })
                        })
                })
                .await?;
        },
        _ => {
            // For other voting methods, just show a message that it's not implemented yet
            component
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message
                                .content(format!("Voting for {} method is not yet implemented", poll.voting_method))
                                .ephemeral(true)
                        })
                })
                .await?;
        }
    }
    
    Ok(())
}

// Helper function to record a star vote
pub async fn handle_star_vote(
    database: &Database,
    ctx: &Context,
    component: &MessageComponentInteraction,
    poll_id: &str,
    option_id: &str,
    rating: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Recording star vote: poll_id={}, option_id={}, rating={}", poll_id, option_id, rating);
    
    // Record the vote in the database
    let vote = crate::models::Vote {
        user_id: component.user.id.to_string(),
        poll_id: poll_id.to_string(),
        option_id: option_id.to_string(),
        rating,
        timestamp: Utc::now(),
    };
    
    // Save the vote, replacing any existing vote for this poll+option+user
    database.save_vote(&vote).await?;
    
    info!("Sending vote acknowledgment as ephemeral message");
    
    // Use ephemeral message to acknowledge the vote
    component
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message
                        .content(format!("You rated \"{}\" with {} stars", option_id, rating))
                        .ephemeral(true)
                })
        })
        .await?;
    
    info!("Successfully recorded vote and sent acknowledgment");
    Ok(())
}
