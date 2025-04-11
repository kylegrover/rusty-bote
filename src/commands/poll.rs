use crate::db::Database;
use crate::models::{Poll, PollOption, VotingMethod};
use chrono::{Duration, Utc};
use serenity::builder::{CreateActionRow, CreateButton, CreateEmbed};
use serenity::builder::CreateApplicationCommand;
use serenity::model::application::component::ButtonStyle;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::prelude::*;

pub fn create_poll_command(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("poll")
        .description("Create and manage polls")
        .create_option(|option| {
            option
                .name("create")
                .description("Create a new poll")
                .kind(serenity::model::application::command::CommandOptionType::SubCommand)
                .create_sub_option(|sub_option| {
                    sub_option
                        .name("question")
                        .description("The poll question")
                        .kind(serenity::model::application::command::CommandOptionType::String)
                        .required(true)
                })
                .create_sub_option(|sub_option| {
                    sub_option
                        .name("options")
                        .description("Comma-separated list of options")
                        .kind(serenity::model::application::command::CommandOptionType::String)
                        .required(true)
                })
                .create_sub_option(|sub_option| {
                    sub_option
                        .name("method")
                        .description("Voting method to use")
                        .kind(serenity::model::application::command::CommandOptionType::String)
                        .add_string_choice("STAR", "star")
                        .add_string_choice("Plurality", "plurality")
                        .add_string_choice("Ranked Choice", "ranked")
                        .add_string_choice("Approval", "approval")
                        .required(true)
                })
                .create_sub_option(|sub_option| {
                    sub_option
                        .name("duration")
                        .description("Duration in minutes (default: 1440 = 24 hours, 0 for manual close)")
                        .kind(serenity::model::application::command::CommandOptionType::Integer)
                        .required(false)
                })
                .create_sub_option(|sub_option| {
                    sub_option
                        .name("anonymous")
                        .description("Whether votes should be anonymous (default: true)")
                        .kind(serenity::model::application::command::CommandOptionType::Boolean)
                        .required(false)
                })
        })
        .create_option(|option| {
            option
                .name("end")
                .description("End an active poll")
                .kind(serenity::model::application::command::CommandOptionType::SubCommand)
                .create_sub_option(|sub_option| {
                    sub_option
                        .name("poll_id")
                        .description("ID of the poll to end")
                        .kind(serenity::model::application::command::CommandOptionType::String)
                        .required(true)
                })
        })
}

pub async fn handle_poll_command(
    database: &Database,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get the subcommand
    let subcommand_name = match command.data.options.first() {
        Some(option) => option.name.as_str(),
        None => {
            send_error_response(ctx, command, "No subcommand provided").await?;
            return Ok(());
        }
    };

    match subcommand_name {
        "create" => handle_create_poll(database, ctx, command).await?,
        "end" => handle_end_poll(database, ctx, command).await?,
        _ => {
            send_error_response(ctx, command, "Unknown subcommand").await?;
        }
    }

    Ok(())
}

async fn handle_create_poll(
    database: &Database,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get command options
    let options = match command.data.options.first() {
        Some(subcommand) => &subcommand.options,
        None => {
            send_error_response(ctx, command, "Missing options").await?;
            return Ok(());
        }
    };
    
    // Extract parameters
    let mut question = String::new();
    let mut options_str = String::new();
    let mut method_str = String::new();
    let mut duration: Option<i64> = None;
    let mut anonymous = true;
    
    for option in options {
        match option.name.as_str() {
            "question" => {
                question = option.value.as_ref().unwrap().as_str().unwrap().to_string();
            },
            "options" => {
                options_str = option.value.as_ref().unwrap().as_str().unwrap().to_string();
            },
            "method" => {
                method_str = option.value.as_ref().unwrap().as_str().unwrap().to_string();
            },
            "duration" => {
                if let Some(value) = option.value.as_ref() {
                    duration = Some(value.as_i64().unwrap_or(1440));
                }
            },
            "anonymous" => {
                if let Some(value) = option.value.as_ref() {
                    anonymous = value.as_bool().unwrap_or(true);
                }
            },
            _ => {}
        }
    }
    
    // Parse options
    let options_vec: Vec<String> = options_str.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    // Validate inputs
    if options_vec.len() < 2 {
        send_error_response(ctx, command, "You need at least 2 options for a poll").await?;
        return Ok(());
    }
    
    if options_vec.len() > 10 {
        send_error_response(ctx, command, "Maximum 10 options allowed").await?;
        return Ok(());
    }
    
    // Parse voting method
    let voting_method = match method_str.as_str() {
        "star" => VotingMethod::Star,
        "plurality" => VotingMethod::Plurality,
        "ranked" => VotingMethod::Ranked,
        "approval" => VotingMethod::Approval,
        _ => {
            send_error_response(ctx, command, "Invalid voting method").await?;
            return Ok(());
        }
    };
    
    // Create poll object
    let guild_id = command.guild_id.unwrap().to_string();
    let channel_id = command.channel_id.to_string();
    let creator_id = command.user.id.to_string();
    
    let poll = Poll::new(
        guild_id,
        channel_id,
        creator_id,
        question.clone(),
        options_vec,
        voting_method.clone(),
        duration,
    );
    
    // Save poll to database
    database.create_poll(&poll).await?;
    
    // Acknowledge the command with a response showing the poll
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message
                        .content("Poll created!")
                        .embed(|e| create_poll_embed(e, &poll))
                        .components(|c| {
                            c.create_action_row(|row| {
                                create_poll_components(row, &poll)
                            })
                        })
                })
        })
        .await?;

    Ok(())
}

fn create_poll_embed<'a>(embed: &'a mut CreateEmbed, poll: &Poll) -> &'a mut CreateEmbed {
    let method_name = match poll.voting_method {
        VotingMethod::Star => "STAR Voting",
        VotingMethod::Plurality => "Plurality Voting",
        VotingMethod::Ranked => "Ranked Choice Voting",
        VotingMethod::Approval => "Approval Voting",
    };
    
    let ends_at_str = match poll.ends_at {
        Some(time) => format!("<t:{}:R>", time.timestamp()),
        None => "When manually ended".to_string(),
    };
    
    let options_list = poll.options.iter()
        .map(|option| format!("• {}", option.text))
        .collect::<Vec<String>>()
        .join("\n");
    
    embed
        .title(&poll.question)
        .description(format!("**Options:**\n{}", options_list))
        .field("Voting Method", method_name, true)
        .field("Poll ID", &poll.id, true)
        .field("Ends", ends_at_str, true)
        .footer(|f| f.text("Click the buttons below to vote!"))
        .timestamp(poll.created_at.to_rfc3339())
}

fn create_poll_components<'a>(row: &'a mut CreateActionRow, poll: &Poll) -> &'a mut CreateActionRow {
    match poll.voting_method {
        VotingMethod::Star => {
            row.create_button(|button| {
                button
                    .custom_id("vote_button")
                    .style(ButtonStyle::Primary)
                    .label("Cast Your Vote")
            })
        },
        _ => {
            row.create_button(|button| {
                button
                    .custom_id("vote_button")
                    .style(ButtonStyle::Primary)
                    .label("Cast Your Vote (Coming Soon)")
            })
        }
    }
}

async fn handle_end_poll(
    database: &Database,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // In a full implementation, we would extract the poll ID and end the poll
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("Poll ending functionality coming soon!")
                })
        })
        .await?;

    Ok(())
}

async fn send_error_response(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    error_message: &str,
) -> Result<(), serenity::Error> {
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| 
                    message
                        .content(format!("Error: {}", error_message))
                        .ephemeral(true)
                )
        })
        .await
}
