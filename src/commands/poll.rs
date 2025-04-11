use crate::db::Database;
use serenity::builder::CreateApplicationCommand;
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
    _database: &Database,
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
        "create" => handle_create_poll(ctx, command).await?,
        "end" => handle_end_poll(ctx, command).await?,
        _ => {
            send_error_response(ctx, command, "Unknown subcommand").await?;
        }
    }

    Ok(())
}

async fn handle_create_poll(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // In a full implementation, we would extract all options and create a poll in the database
    // For now, we'll just acknowledge the command with a simple response
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("Poll creation functionality coming soon!")
                })
        })
        .await?;

    Ok(())
}

async fn handle_end_poll(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // In a full implementation, we would extract the poll ID and end the poll
    // For now, we'll just acknowledge the command with a simple response
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
                .interaction_response_data(|message| message.content(error_message).ephemeral(true))
        })
        .await
}
