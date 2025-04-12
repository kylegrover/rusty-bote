use crate::db::Database;
use serenity::model::application::interaction::{
    message_component::MessageComponentInteraction, InteractionResponseType,
};
// Import ActionRowComponent to match against its variants
use serenity::model::application::component::{ActionRowComponent, ComponentType, ButtonStyle};
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
                                            // Add a text label for the option
                                            row.create_button(|btn| {
                                                btn.custom_id(format!("label_{}", option.id))
                                                   .label(&option.text)
                                                   .style(ButtonStyle::Secondary)
                                                   .disabled(true)
                                            })
                                        });
                                        
                                        c.create_action_row(|row| {
                                            row.create_select_menu(|menu| {
                                                menu.custom_id(format!("vote_{}_{}", poll_id, option.id))
                                                    .placeholder("Select your rating")
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
        crate::models::VotingMethod::Plurality => {
            // For Plurality voting, show buttons for each option (pick one)
            component
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message
                                .ephemeral(true)
                                .content(format!("**{}**\nSelect ONE option:", poll.question))
                                .components(|c| {
                                    // Create buttons for each option, with up to 5 per row
                                    let mut options_iter = poll.options.iter().peekable();
                                    while options_iter.peek().is_some() {
                                        c.create_action_row(|row| {
                                            for _ in 0..5 {
                                                if let Some(option) = options_iter.next() {
                                                    row.create_button(|btn| {
                                                        btn.custom_id(format!("plurality_{}_{}", poll_id, option.id))
                                                           .label(&option.text)
                                                           .style(ButtonStyle::Primary)
                                                    });
                                                } else {
                                                    break;
                                                }
                                            }
                                            row
                                        });
                                    }
                                    c
                                })
                        })
                })
                .await?;
        },
        crate::models::VotingMethod::Approval => {
            // For Approval voting, show toggle buttons for each option
            component
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message
                                .ephemeral(true)
                                .content(format!("**{}**\nApprove as many options as you like:", poll.question))
                                .components(|c| {
                                    // Create a row with a toggle button for each option
                                    for option in &poll.options {
                                        c.create_action_row(|row| {
                                            row.create_button(|btn| {
                                                btn.custom_id(format!("approval_{}_{}_0", poll_id, option.id))
                                                   .label(format!("❌ {}", option.text))
                                                   .style(ButtonStyle::Danger)
                                            })
                                        });
                                    }
                                    
                                    // Add a submit button at the bottom
                                    c.create_action_row(|row| {
                                        row.create_button(|btn| {
                                            btn.custom_id(format!("approval_submit_{}", poll_id))
                                               .label("Submit Votes")
                                               .style(ButtonStyle::Success)
                                        })
                                    });
                                    
                                    c
                                })
                        })
                })
                .await?;
        },
        crate::models::VotingMethod::Ranked => {
            // For Ranked Choice voting, show a numbered list with up/down buttons
            let user_id = component.user.id.to_string();
            
            // Get any existing ranks for this user
            let existing_votes = database.get_user_poll_votes(&poll_id, &user_id).await?;
            
            // Create a mapping of option_id to rank
            let mut option_ranks: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
            for vote in existing_votes {
                option_ranks.insert(vote.option_id, vote.rating);
            }
            
            // Sort options by existing rank, then by unranked options
            let mut ranked_options = poll.options.clone();
            ranked_options.sort_by(|a, b| {
                let rank_a = option_ranks.get(&a.id).unwrap_or(&0);
                let rank_b = option_ranks.get(&b.id).unwrap_or(&0);
                
                // First sort by whether they have a rank (0 = unranked)
                let has_rank_a = *rank_a > 0;
                let has_rank_b = *rank_b > 0;
                
                if has_rank_a != has_rank_b {
                    return has_rank_a.cmp(&has_rank_b).reverse();  // Ranked options first
                }
                
                // Then sort by rank value (lower rank = higher preference)
                if has_rank_a && has_rank_b {
                    return rank_a.cmp(rank_b);
                }
                
                // Finally sort by option text for unranked options
                a.text.cmp(&b.text)
            });
            
            component
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message
                                .ephemeral(true)
                                .content(format!("**{}**\nRank the options in your order of preference:", poll.question))
                                .components(|c| {
                                    // Create a row with option name, current rank, and up/down buttons for each option
                                    let mut rank_display = 1;
                                    
                                    for option in &ranked_options {
                                        let current_rank = option_ranks.get(&option.id).unwrap_or(&0);
                                        let display_text = if *current_rank > 0 {
                                            format!("#{} - {}", current_rank, option.text)
                                        } else {
                                            format!("Unranked - {}", option.text)
                                        };
                                        
                                        c.create_action_row(|row| {
                                            // Option text with current rank
                                            row.create_button(|btn| {
                                                btn.custom_id(format!("rank_label_{}_{}", poll_id, option.id))
                                                   .label(&display_text)
                                                   .style(ButtonStyle::Secondary)
                                                   .disabled(true)
                                            })
                                            .create_button(|btn| {
                                                btn.custom_id(format!("rank_up_{}_{}", poll_id, option.id))
                                                   .emoji('⬆')
                                                   .style(ButtonStyle::Primary)
                                            })
                                            .create_button(|btn| {
                                                btn.custom_id(format!("rank_down_{}_{}", poll_id, option.id))
                                                   .emoji('⬇')
                                                   .style(ButtonStyle::Primary)
                                            })
                                            .create_button(|btn| {
                                                btn.custom_id(format!("rank_remove_{}_{}", poll_id, option.id))
                                                   .emoji('🗑')
                                                   .style(ButtonStyle::Danger)
                                            })
                                        });
                                        
                                        if *current_rank > 0 {
                                            rank_display += 1;
                                        }
                                    }
                                    
                                    // Add a submit button at the bottom
                                    c.create_action_row(|row| {
                                        row.create_button(|btn| {
                                            btn.custom_id(format!("rank_submit_{}", poll_id))
                                               .label("Submit Rankings")
                                               .style(ButtonStyle::Success)
                                        })
                                    });
                                    
                                    c
                                })
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
    
    // Acknowledge the vote with a deferred update response instead of a message
    component
        .create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::DeferredUpdateMessage)
        })
        .await?;
    
    info!("Successfully recorded vote");
    Ok(())
}

// Handle plurality vote (select exactly one option)
pub async fn handle_plurality_vote(
    database: &Database,
    ctx: &Context,
    component: &MessageComponentInteraction,
    poll_id: &str,
    option_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Recording plurality vote: poll_id={}, option_id={}", poll_id, option_id);
    
    // Get all options for this poll
    let poll = database.get_poll(poll_id).await?;
    let user_id = component.user.id.to_string();
    
    // Create transactions to update all votes
    for option in &poll.options {
        // Set the selected option to 1, all others to 0
        let rating = if option.id == option_id { 1 } else { 0 };
        
        let vote = crate::models::Vote {
            user_id: user_id.clone(),
            poll_id: poll_id.to_string(),
            option_id: option.id.clone(),
            rating,
            timestamp: Utc::now(),
        };
        
        // Save each vote
        database.save_vote(&vote).await?;
    }
    
    // Acknowledge the vote
    component
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message
                        .content("Your vote has been recorded.")
                        .ephemeral(true)
                })
        })
        .await?;
    
    info!("Successfully recorded plurality vote");
    Ok(())
}

// Handle approval vote toggle
pub async fn handle_approval_vote_toggle(
    database: &Database,
    ctx: &Context,
    component: &MessageComponentInteraction,
    poll_id: &str,
    option_id: &str,
    current_value: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let new_value = 1 - current_value;
    let display_symbol = if new_value == 1 { "✅" } else { "❌" };
    let button_style = if new_value == 1 { ButtonStyle::Success } else { ButtonStyle::Danger };
    
    let poll = database.get_poll(poll_id).await?;
    let option_text = poll.options.iter()
        .find(|o| o.id == option_id)
        .map(|o| o.text.clone())
        .unwrap_or_else(|| "Option".to_string());

    component
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|message| {
                    message.components(|c| {
                        for row in &component.message.components {
                            c.create_action_row(|ar| {
                                for comp in &row.components {
                                    // Match on the ActionRowComponent enum
                                    if let ActionRowComponent::Button(button_data) = comp {
                                        let (custom_id, label, style) = if button_data.custom_id.as_deref() == Some(&component.data.custom_id) {
                                            // This is the clicked button - update it
                                            (
                                                format!("approval_{}_{}_{}", poll_id, option_id, new_value),
                                                format!("{} {}", display_symbol, option_text),
                                                button_style
                                            )
                                        } else {
                                            // Copy existing button properties
                                            (
                                                button_data.custom_id.clone().unwrap_or_default(),
                                                button_data.label.clone().unwrap_or_default(),
                                                button_data.style // ButtonStyle is directly available
                                            )
                                        };
                                        
                                        ar.create_button(|b| {
                                            b.custom_id(custom_id)
                                             .label(label)
                                             .style(style)
                                        });
                                    }
                                    // Handle other component types if necessary, otherwise ignore
                                }
                                ar
                            });
                        }
                        c
                    })
                })
        })
        .await?;

    Ok(())
}

// Handle final approval vote submission
pub async fn handle_approval_submit(
    database: &Database,
    ctx: &Context,
    component: &MessageComponentInteraction,
    poll_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Submitting approval votes: poll_id={}", poll_id);
    
    let user_id = component.user.id.to_string();
    let mut votes = Vec::new();
    
    for row in &component.message.components {
        for comp in &row.components {
            // Match on the ActionRowComponent enum
            if let ActionRowComponent::Button(button_data) = comp {
                if let Some(custom_id) = &button_data.custom_id {
                    if custom_id.starts_with("approval_") && !custom_id.starts_with("approval_submit_") {
                        let parts: Vec<&str> = custom_id.split('_').collect();
                        if parts.len() >= 4 {
                            let option_id = parts[2];
                            let value: i32 = parts[3].parse().unwrap_or(0);
                            
                            votes.push(crate::models::Vote {
                                user_id: user_id.clone(),
                                poll_id: poll_id.to_string(),
                                option_id: option_id.to_string(),
                                rating: value,
                                timestamp: Utc::now(),
                            });
                        }
                    }
                }
            }
            // Handle other component types if necessary, otherwise ignore
        }
    }
    
    for vote in votes {
        database.save_vote(&vote).await?;
    }
    
    component
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message
                        .content("Your votes have been recorded.")
                        .ephemeral(true)
                })
        })
        .await?;
    
    info!("Successfully recorded approval votes");
    Ok(())
}

// Helper functions for ranked choice voting
pub async fn handle_rank_action(
    database: &Database,
    ctx: &Context,
    component: &MessageComponentInteraction,
    poll_id: &str,
    option_id: &str,
    action: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user_id = component.user.id.to_string();
    
    // Get current rankings for this user
    let existing_votes = database.get_user_poll_votes(poll_id, &user_id).await?;
    let mut rankings = std::collections::HashMap::new();
    
    // Build a map of option_id -> rank and rank -> option_id
    for vote in &existing_votes {
        if vote.rating > 0 {
            rankings.insert(vote.option_id.clone(), vote.rating);
        }
    }
    
    // Get the current rank of this option
    let current_rank = rankings.get(option_id).cloned().unwrap_or(0);
    
    // Apply the requested action
    match action {
        "up" => {
            if current_rank == 0 {
                // Unranked option - give it the next available rank
                let next_rank = rankings.values().max().unwrap_or(&0) + 1;
                rankings.insert(option_id.to_string(), next_rank);
            } else if current_rank > 1 {
                // Move up in ranking (lower number = higher preference)
                let higher_rank = current_rank - 1;
                
                // Find and swap with the option that has the higher rank
                if let Some((swap_option, _)) = rankings.iter()
                    .find(|(_, rank)| **rank == higher_rank)
                    .map(|(opt_id, _)| (opt_id.clone(), higher_rank)) {
                    rankings.insert(swap_option, current_rank);
                }
                
                rankings.insert(option_id.to_string(), higher_rank);
            }
        },
        "down" => {
            if current_rank > 0 {
                // If not the lowest rank, move down
                let lower_rank = current_rank + 1;
                let max_rank = rankings.values().max().unwrap_or(&0);
                
                if lower_rank <= *max_rank {
                    // Find and swap with the option that has the lower rank
                    if let Some((swap_option, _)) = rankings.iter()
                        .find(|(_, rank)| **rank == lower_rank)
                        .map(|(opt_id, _)| (opt_id.clone(), lower_rank)) {
                        rankings.insert(swap_option, current_rank);
                    }
                    
                    rankings.insert(option_id.to_string(), lower_rank);
                }
            }
        },
        "remove" => {
            if current_rank > 0 {
                // Remove this option from rankings
                rankings.remove(option_id);
                
                // Shift all rankings above this one down by 1
                for (_, rank) in rankings.iter_mut() {
                    if *rank > current_rank {
                        *rank -= 1;
                    }
                }
            }
        },
        _ => {}
    }
    
    // Save the updated rankings
    let poll = database.get_poll(poll_id).await?;
    
    // Clear existing votes and save new ones
    for option in &poll.options {
        let rank = rankings.get(&option.id).cloned().unwrap_or(0);
        
        let vote = crate::models::Vote {
            user_id: user_id.clone(),
            poll_id: poll_id.to_string(),
            option_id: option.id.clone(),
            rating: rank,
            timestamp: Utc::now(),
        };
        
        database.save_vote(&vote).await?;
    }
    
    // Re-render the voting UI with updated rankings
    handle_vote_button(database, ctx, component).await?;
    
    Ok(())
}

// Handle ranked choice final submission
pub async fn handle_rank_submit(
    database: &Database,
    ctx: &Context,
    component: &MessageComponentInteraction,
    poll_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Acknowledge the submission
    component
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message
                        .content("Your rankings have been recorded.")
                        .ephemeral(true)
                })
        })
        .await?;
    
    info!("Successfully recorded ranked choice votes");
    Ok(())
}
