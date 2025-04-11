use crate::db::Database;
use serenity::builder::CreateActionRow;
use serenity::model::application::component::ButtonStyle;
use serenity::model::application::interaction::{
    message_component::MessageComponentInteraction, InteractionResponseType,
};
use serenity::prelude::*;
use chrono::Utc;

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
                                    // Each option gets its own action row with star rating buttons
                                    for option in &poll.options {
                                        c.create_action_row(|row| {
                                            // First add the option name as a disabled button
                                            row.create_button(|btn| {
                                                btn.custom_id(format!("option_name_{}", option.id))
                                                    .label(&option.text)
                                                    .style(ButtonStyle::Secondary)
                                                    .disabled(true)
                                            });
                                            
                                            // Add 0-5 star buttons
                                            for stars in 0..=5 {
                                                row.create_button(|btn| {
                                                    let custom_id = format!("vote_{}_{}_{}", poll_id, option.id, stars);
                                                    let label = if stars == 0 {
                                                        "0".to_string()
                                                    } else {
                                                        "⭐".repeat(stars as usize)
                                                    };
                                                    
                                                    btn.custom_id(custom_id)
                                                        .label(label)
                                                        .style(if stars == 0 {
                                                            ButtonStyle::Secondary
                                                        } else {
                                                            ButtonStyle::Primary
                                                        })
                                                });
                                            }
                                            row
                                        });
                                    }
                                    
                                    // Add a submit button
                                    c.create_action_row(|row| {
                                        row.create_button(|btn| {
                                            btn.custom_id(format!("submit_vote_{}", poll_id))
                                                .label("Submit Vote")
                                                .style(ButtonStyle::Success)
                                        })
                                    })
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
    
    // Acknowledge the vote
    component
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|message| {
                    message.content(format!("You rated \"{}\" with {} stars", option_id, rating))
                })
        })
        .await?;
    
    Ok(())
}
