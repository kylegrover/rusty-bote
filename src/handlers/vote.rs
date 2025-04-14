use crate::db::Database;
use serenity::model::application::interaction::{
    message_component::MessageComponentInteraction, InteractionResponseType,
};
use serenity::model::application::component::{ActionRowComponent, ButtonStyle};
use serenity::prelude::*;
use chrono::Utc;
use log::{info, warn};
use crate::models::Poll;

pub async fn handle_vote_button(
    database: &Database,
    ctx: &Context,
    component: &MessageComponentInteraction,
    poll: &Poll,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user_id = component.user.id.to_string();
    let existing_votes = database.get_user_poll_votes(&poll.id, &user_id).await?;
    let mut option_ratings = std::collections::HashMap::<String, i32>::new();
    for vote in &existing_votes {
        option_ratings.insert(vote.option_id.clone(), vote.rating);
    }

    match poll.voting_method {
        crate::models::VotingMethod::Star => {
            let options_to_show = if poll.options.len() > 5 {
                warn!("STAR poll {} has {} options, exceeding interactive limit of 5 due to Discord constraints.", poll.id, poll.options.len());
                &poll.options[..2]
            } else {
                &poll.options[..]
            };
            let limited_message = if poll.options.len() > 5 {
                "\n*Note: Interactive rating is limited to 5 options due to Discord constraints.*"
            } else {
                ""
            };

            component
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message
                                .ephemeral(true)
                                .content(format!(
                                    "**{}**\nRate each option from 0 to 5 stars:{}",
                                    poll.question, limited_message
                                ))
                                .components(|c| {
                                    for option in options_to_show {
                                        let rating = option_ratings.get(&option.id).copied().unwrap_or(0);
                                        c.create_action_row(|row| {
                                            for i in 1..=5 {
                                                let star = if i <= rating { '★' } else { '☆' };
                                                let button_style = if i <= rating { ButtonStyle::Primary } else { ButtonStyle::Secondary };

                                                row.create_button(|btn| {
                                                    btn.custom_id(format!("star_{}_{}_{}", poll.id, option.id, i))
                                                       .label(format!("{} ", star))
                                                       .style(button_style)
                                                       .disabled(i == rating)
                                                });
                                            }
                                            row
                                        });
                                    }
                                    c.create_action_row(|row| {
                                        row.create_button(|btn| {
                                            btn.custom_id(format!("done_voting_{}", poll.id))
                                               .label("Done Voting")
                                               .style(ButtonStyle::Success)
                                        })
                                    });
                                    c
                                })
                        })
                })
                .await?;
        },
        crate::models::VotingMethod::Plurality => {
            component
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message
                                .ephemeral(true)
                                .content(format!("**{}**\nSelect ONE option:", poll.question))
                                .components(|c| {
                                    let mut options_iter = poll.options.iter().peekable();
                                    while options_iter.peek().is_some() {
                                        c.create_action_row(|row| {
                                            for _ in 0..5 {
                                                if let Some(option) = options_iter.next() {
                                                    let selected = option_ratings.get(&option.id).copied().unwrap_or(0) > 0;
                                                    let style = if selected { ButtonStyle::Success } else { ButtonStyle::Primary };
                                                    let prefix = if selected { "✓ " } else { "" };
                                                    
                                                    row.create_button(|btn| {
                                                        btn.custom_id(format!("plurality_{}_{}", poll.id, option.id))
                                                           .label(format!("{}{}", prefix, option.text))
                                                           .style(style)
                                                    });
                                                } else {
                                                    break;
                                                }
                                            }
                                            row
                                        });
                                    }
                                    c.create_action_row(|row| {
                                        row.create_button(|btn| {
                                            btn.custom_id(format!("done_voting_{}", poll.id))
                                               .label("Done Voting")
                                               .style(ButtonStyle::Success)
                                        })
                                    });
                                    c
                                })
                        })
                })
                .await?;
        },
        crate::models::VotingMethod::Approval => {
            component
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message
                                .ephemeral(true)
                                .content(format!("**{}**\nApprove as many options as you like:", poll.question))
                                .components(|c| {
                                    let mut options_iter = poll.options.iter().peekable();
                                    while options_iter.peek().is_some() {
                                        c.create_action_row(|row| {
                                            for _ in 0..5 {
                                                if let Some(option) = options_iter.next() {
                                                    let value = option_ratings.get(&option.id).copied().unwrap_or(0);
                                                    let display_symbol = if value > 0 { "✅" } else { "❌" };
                                                    let button_style = if value > 0 { ButtonStyle::Success } else { ButtonStyle::Danger };
                                                    
                                                    row.create_button(|btn| {
                                                        btn.custom_id(format!("approval_{}_{}_{}",
                                                            poll.id, option.id, value))
                                                           .label(format!("{} {}", display_symbol, option.text))
                                                           .style(button_style)
                                                    });
                                                } else {
                                                    break;
                                                }
                                            }
                                            row
                                        });
                                    }
                                    c.create_action_row(|row| {
                                        row.create_button(|btn| {
                                            btn.custom_id(format!("done_voting_{}", poll.id))
                                               .label("Done Voting")
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
            let user_id = component.user.id.to_string();
            let existing_votes = database.get_user_poll_votes(&poll.id, &user_id).await?;
            let mut option_ranks: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
            for vote in existing_votes {
                option_ranks.insert(vote.option_id, vote.rating);
            }

            let mut ranked_options = poll.options.clone();
            ranked_options.sort_by(|a, b| {
                let rank_a = option_ranks.get(&a.id).unwrap_or(&0);
                let rank_b = option_ranks.get(&b.id).unwrap_or(&0);

                let has_rank_a = *rank_a > 0;
                let has_rank_b = *rank_b > 0;

                if has_rank_a != has_rank_b {
                    return has_rank_a.cmp(&has_rank_b).reverse();
                }

                if has_rank_a && has_rank_b {
                    return rank_a.cmp(rank_b);
                }

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
                                    let mut rank_display = 1;
                                    for option in &ranked_options {
                                        let current_rank = option_ranks.get(&option.id).unwrap_or(&0);
                                        let display_text = if *current_rank > 0 {
                                            format!("#{} - {}", current_rank, option.text)
                                        } else {
                                            format!("Unranked - {}", option.text)
                                        };
                                        c.create_action_row(|row| {
                                            row.create_button(|btn| {
                                                btn.custom_id(format!("rank_label_{}_{}", poll.id, option.id))
                                                   .label(&display_text)
                                                   .style(ButtonStyle::Secondary)
                                                   .disabled(true)
                                            })
                                            .create_button(|btn| {
                                                btn.custom_id(format!("rank_up_{}_{}", poll.id, option.id))
                                                   .emoji('⬆')
                                                   .style(ButtonStyle::Primary)
                                                   .disabled(*current_rank == 1)
                                            })
                                            .create_button(|btn| {
                                                btn.custom_id(format!("rank_down_{}_{}", poll.id, option.id))
                                                   .emoji('⬇')
                                                   .style(ButtonStyle::Primary)
                                                   .disabled(*current_rank == 0 || *current_rank == option_ranks.values().filter(|&&r| r > 0).count() as i32)
                                            })
                                            .create_button(|btn| {
                                                btn.custom_id(format!("rank_remove_{}_{}", poll.id, option.id))
                                                   .emoji('🗑')
                                                   .style(ButtonStyle::Danger)
                                                   .disabled(*current_rank == 0)
                                            })
                                        });
                                        if *current_rank > 0 {
                                            rank_display += 1;
                                        }
                                    }
                                    c.create_action_row(|row| {
                                        row.create_button(|btn| {
                                            btn.custom_id(format!("done_voting_{}", poll.id))
                                               .label("Done Voting")
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

pub async fn handle_star_vote(
    database: &Database,
    ctx: &Context,
    component: &MessageComponentInteraction,
    poll_id: &str,
    option_id: &str,
    rating: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Recording star vote: poll_id={}, option_id={}, rating={}", poll_id, option_id, rating);

    // Save the vote immediately
    let vote = crate::models::Vote {
        user_id: component.user.id.to_string(),
        poll_id: poll_id.to_string(),
        option_id: option_id.to_string(),
        rating,
        timestamp: Utc::now(),
    };

    database.save_vote(&vote).await?;

    let poll = database.get_poll(poll_id).await?;
    let existing_votes = database.get_user_poll_votes(&poll.id, &component.user.id.to_string()).await?;
    let mut option_ratings = std::collections::HashMap::<String, i32>::new();
    for vote in &existing_votes {
        option_ratings.insert(vote.option_id.clone(), vote.rating);
    }

    component
        .create_interaction_response(&ctx.http, |resp| {
            resp.kind(serenity::model::application::interaction::InteractionResponseType::UpdateMessage)
                .interaction_response_data(|msg| {
                    msg.ephemeral(true)
                       .content(format!("**{}**\nRate each option from 0 to 5 stars:", poll.question))
                       .components(|c| {
                           for option in &poll.options {
                               let rating = option_ratings.get(&option.id).copied().unwrap_or(0);
                               c.create_action_row(|row| {
                                   for i in 1..=5 {
                                       let star = if i <= rating { '★' } else { '☆' };
                                       let button_style = if i <= rating { ButtonStyle::Primary } else { ButtonStyle::Secondary };

                                       row.create_button(|btn| {
                                           btn.custom_id(format!("star_{}_{}_{}", poll.id, option.id, i))
                                              .label(format!("{} ", star))
                                              .style(button_style)
                                              .disabled(i == rating)
                                       });
                                   }
                                   row
                               });
                           }
                           c.create_action_row(|row| {
                               row.create_button(|btn| {
                                   btn.custom_id(format!("done_voting_{}", poll.id))
                                      .label("Done Voting")
                                      .style(ButtonStyle::Success)
                               })
                           });
                           c
                       })
                })
        })
        .await?;

    info!("Successfully recorded vote and updated UI");
    Ok(())
}

pub async fn handle_plurality_vote(
    database: &Database,
    ctx: &Context,
    component: &MessageComponentInteraction,
    poll_id: &str,
    option_id: &str,
    poll: &Poll,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Recording plurality vote: poll_id={}, option_id={}", poll_id, option_id);

    let user_id = component.user.id.to_string();

    // Save the votes immediately
    for option in &poll.options {
        let rating = if option.id == option_id { 1 } else { 0 };

        let vote = crate::models::Vote {
            user_id: user_id.clone(),
            poll_id: poll_id.to_string(),
            option_id: option.id.clone(),
            rating,
            timestamp: Utc::now(),
        };

        database.save_vote(&vote).await?;
    }

    // Get updated votes to display current state
    let existing_votes = database.get_user_poll_votes(poll_id, &user_id).await?;
    let mut option_ratings = std::collections::HashMap::<String, i32>::new();
    for vote in &existing_votes {
        option_ratings.insert(vote.option_id.clone(), vote.rating);
    }

    component
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|message| {
                    message
                        .content(format!("**{}**\nSelect ONE option:", poll.question))
                        .components(|c| {
                            let mut options_iter = poll.options.iter().peekable();
                            while options_iter.peek().is_some() {
                                c.create_action_row(|row| {
                                    for _ in 0..5 {
                                        if let Some(option) = options_iter.next() {
                                            let selected = option_ratings.get(&option.id).copied().unwrap_or(0) > 0;
                                            let style = if selected { ButtonStyle::Success } else { ButtonStyle::Primary };
                                            let prefix = if selected { "✓ " } else { "" };
                                            
                                            row.create_button(|btn| {
                                                btn.custom_id(format!("plurality_{}_{}", poll_id, option.id))
                                                   .label(format!("{}{}", prefix, option.text))
                                                   .style(style)
                                            });
                                        } else {
                                            break;
                                        }
                                    }
                                    row
                                });
                            }
                            c.create_action_row(|row| {
                                row.create_button(|btn| {
                                    btn.custom_id(format!("done_voting_{}", poll_id))
                                       .label("Done Voting")
                                       .style(ButtonStyle::Success)
                                })
                            });
                            c
                        })
                })
        })
        .await?;

    info!("Successfully recorded plurality vote and updated UI");
    Ok(())
}

pub async fn handle_approval_vote_toggle(
    database: &Database,
    ctx: &Context,
    component: &MessageComponentInteraction,
    poll_id: &str,
    option_id: &str,
    current_value: i32,
    poll: &Poll,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let new_value = 1 - current_value;
    let display_symbol = if new_value == 1 { "✅" } else { "❌" };
    let button_style = if new_value == 1 { ButtonStyle::Success } else { ButtonStyle::Danger };

    let option_text = poll.options.iter()
        .find(|o| o.id == option_id)
        .map(|o| o.text.clone())
        .unwrap_or_else(|| "Option".to_string());

    // Save the vote immediately
    let vote = crate::models::Vote {
        user_id: component.user.id.to_string(),
        poll_id: poll_id.to_string(),
        option_id: option_id.to_string(),
        rating: new_value,
        timestamp: Utc::now(),
    };
    
    database.save_vote(&vote).await?;

    component
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|message| {
                    message.components(|c| {
                        for row_data in &component.message.components {
                            c.create_action_row(|ar| {
                                for comp_data in &row_data.components {
                                    if let ActionRowComponent::Button(button_data) = comp_data {
                                        if button_data.custom_id.as_deref() == Some(&component.data.custom_id) {
                                            ar.create_button(|b| {
                                                b.custom_id(format!("approval_{}_{}_{}", poll_id, option_id, new_value))
                                                 .label(format!("{} {}", display_symbol, option_text))
                                                 .style(button_style)
                                            });
                                        } else {
                                            ar.create_button(|b| {
                                                b.custom_id(button_data.custom_id.clone().unwrap_or_default())
                                                 .label(button_data.label.clone().unwrap_or_default())
                                                 .style(button_data.style)
                                            });
                                        }
                                    }
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

pub async fn handle_done_voting(
    database: &Database,
    ctx: &Context,
    component: &MessageComponentInteraction,
    poll_id: &str,
    poll: &Poll,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("User completed voting for poll_id={}", poll_id);
    let user_id = component.user.id.to_string();
    
    // Get the user's current votes
    let user_votes = database.get_user_poll_votes(poll_id, &user_id).await?;
    
    // Generate vote summary based on voting method
    let mut vote_summary = format!("**{}**\n{} Voting\n\nYour vote has been recorded:\n", 
        poll.question, poll.voting_method);
    
    match poll.voting_method {
        crate::models::VotingMethod::Star => {
            let mut vote_map = std::collections::HashMap::new();
            for vote in &user_votes {
                vote_map.insert(vote.option_id.clone(), vote.rating);
            }
            
            for option in &poll.options {
                let rating = vote_map.get(&option.id).cloned().unwrap_or(0);
                let stars = "★".repeat(rating as usize);
                let empty_stars = "☆".repeat((5 - rating) as usize);
                vote_summary.push_str(&format!("{}: {}{}\n", option.text, stars, empty_stars));
            }
        },
        crate::models::VotingMethod::Plurality => {
            for option in &poll.options {
                let selected = user_votes.iter()
                    .any(|v| v.option_id == option.id && v.rating > 0);
                let symbol = if selected { "✓" } else { " " };
                vote_summary.push_str(&format!("{}: {}\n", option.text, symbol));
            }
        },
        crate::models::VotingMethod::Approval => {
            let mut vote_map = std::collections::HashMap::new();
            for vote in &user_votes {
                vote_map.insert(vote.option_id.clone(), vote.rating);
            }
            
            for option in &poll.options {
                // Only consider options with rating=1 as approved
                let approved = vote_map.get(&option.id).copied().unwrap_or(0) == 1;
                let symbol = if approved { "✅" } else { "❌" };
                vote_summary.push_str(&format!("{}: {}\n", option.text, symbol));
            }
        },
        crate::models::VotingMethod::Ranked => {
            let mut rankings = std::collections::HashMap::new();
            for vote in &user_votes {
                if vote.rating > 0 {
                    rankings.insert(vote.option_id.clone(), vote.rating);
                }
            }
            
            let mut ranked_options = poll.options.clone();
            ranked_options.sort_by(|a, b| {
                let rank_a = rankings.get(&a.id).unwrap_or(&0);
                let rank_b = rankings.get(&b.id).unwrap_or(&0);
                
                if rank_a == &0 && rank_b == &0 {
                    return a.text.cmp(&b.text);
                }
                
                if rank_a == &0 {
                    return std::cmp::Ordering::Greater;
                }
                
                if rank_b == &0 {
                    return std::cmp::Ordering::Less;
                }
                
                rank_a.cmp(rank_b)
            });
            
            for option in &ranked_options {
                let rank = rankings.get(&option.id).cloned().unwrap_or(0);
                if rank > 0 {
                    vote_summary.push_str(&format!("#{}: {}\n", rank, option.text));
                } else {
                    vote_summary.push_str(&format!("Unranked: {}\n", option.text));
                }
            }
        }
    }
    
    component
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|message| {
                    message
                        .content(&vote_summary)
                        .components(|c| {
                            c.create_action_row(|row| {
                                row.create_button(|btn| {
                                    btn.custom_id(format!("vote_change_{}", poll_id))
                                       .label("Change My Vote")
                                       .style(ButtonStyle::Secondary)
                                })
                            })
                        })
                })
        })
        .await?;

    info!("Successfully displayed vote confirmation");
    Ok(())
}

pub async fn handle_change_vote(
    database: &Database,
    ctx: &Context,
    component: &MessageComponentInteraction,
    poll_id: &str,
    poll: &Poll,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("User wants to change their vote for poll_id={}", poll_id);
    // Reuse handle_vote_button to show the voting interface again
    handle_vote_button(database, ctx, component, poll).await
}

pub async fn handle_rank_action(
    database: &Database,
    ctx: &Context,
    component: &MessageComponentInteraction,
    poll_id: &str,
    option_id: &str,
    action: &str,
    poll: &Poll,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user_id = component.user.id.to_string();

    let existing_votes = database.get_user_poll_votes(poll_id, &user_id).await?;
    let mut rankings = std::collections::HashMap::new();

    for vote in &existing_votes {
        if vote.rating > 0 {
            rankings.insert(vote.option_id.clone(), vote.rating);
        }
    }

    let current_rank = rankings.get(option_id).cloned().unwrap_or(0);

    match action {
        "up" => {
            if current_rank == 0 {
                let next_rank = rankings.values().max().unwrap_or(&0) + 1;
                rankings.insert(option_id.to_string(), next_rank);
            } else if current_rank > 1 {
                let higher_rank = current_rank - 1;
                
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
                let lower_rank = current_rank + 1;
                let max_rank = *rankings.values().max().unwrap_or(&0);
                
                if lower_rank <= max_rank {
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
                rankings.remove(option_id);
                
                let mut updated_rankings = std::collections::HashMap::new();
                for (opt_id, rank) in rankings.iter() {
                    if *rank > current_rank {
                        updated_rankings.insert(opt_id.clone(), *rank - 1);
                    } else {
                        updated_rankings.insert(opt_id.clone(), *rank);
                    }
                }
                rankings = updated_rankings;
            }
        },
        _ => {}
    }

    // Save all votes immediately
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

    let mut ranked_options = poll.options.clone();
    ranked_options.sort_by(|a, b| {
        let rank_a = rankings.get(&a.id).unwrap_or(&0);
        let rank_b = rankings.get(&b.id).unwrap_or(&0);
        
        let has_rank_a = *rank_a > 0;
        let has_rank_b = *rank_b > 0;
        
        if has_rank_a != has_rank_b {
            return has_rank_a.cmp(&has_rank_b).reverse();
        }
        if has_rank_a && has_rank_b {
            return rank_a.cmp(rank_b);
        }
        a.text.cmp(&b.text)
    });

    component
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|message| {
                    message
                        .content(format!("**{}**\nRank the options in your order of preference:", poll.question))
                        .components(|c| {
                            for option in &ranked_options {
                                let current_rank = rankings.get(&option.id).unwrap_or(&0);
                                let display_text = if *current_rank > 0 {
                                    format!("#{} - {}", current_rank, option.text)
                                } else {
                                    format!("Unranked - {}", option.text)
                                };
                                
                                c.create_action_row(|row| {
                                    row.create_button(|btn| {
                                        btn.custom_id(format!("rank_label_{}_{}", poll.id, option.id))
                                           .label(&display_text)
                                           .style(ButtonStyle::Secondary)
                                           .disabled(true)
                                    })
                                    .create_button(|btn| {
                                        btn.custom_id(format!("rank_up_{}_{}", poll.id, option.id))
                                           .emoji('⬆')
                                           .style(ButtonStyle::Primary)
                                           .disabled(*current_rank == 1)
                                    })
                                    .create_button(|btn| {
                                        btn.custom_id(format!("rank_down_{}_{}", poll.id, option.id))
                                           .emoji('⬇')
                                           .style(ButtonStyle::Primary)
                                           .disabled(*current_rank == 0 || *current_rank == rankings.values().filter(|&&r| r > 0).count() as i32)
                                    })
                                    .create_button(|btn| {
                                        btn.custom_id(format!("rank_remove_{}_{}", poll.id, option.id))
                                           .emoji('🗑')
                                           .style(ButtonStyle::Danger)
                                           .disabled(*current_rank == 0)
                                    })
                                });
                            }
                            
                            c.create_action_row(|row| {
                                row.create_button(|btn| {
                                    btn.custom_id(format!("done_voting_{}", poll.id))
                                       .label("Done Voting")
                                       .style(ButtonStyle::Success)
                                })
                            });
                            
                            c
                        })
                })
        })
        .await?;
    
    Ok(())
}

pub async fn handle_rank_submit(
    database: &Database,
    ctx: &Context,
    component: &MessageComponentInteraction,
    poll_id: &str,
    poll: Option<&Poll>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Handling rank submit for poll {}", poll_id);
    
    // If we have the poll object, use handle_done_voting for the enhanced experience
    if let Some(p) = poll {
        return handle_done_voting(database, ctx, component, poll_id, p).await;
    }
    
    // Legacy behavior - just show a simple confirmation message
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
    
    Ok(())
}

pub async fn handle_approval_submit(
    database: &Database,
    ctx: &Context,
    component: &MessageComponentInteraction,
    poll_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Legacy approval submit handler called for poll {}", poll_id);
    
    // Simple success message for backward compatibility
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
    
    Ok(())
}
