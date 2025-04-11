mod poll;

use crate::db::Database;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::id::GuildId;
use serenity::prelude::*;

pub async fn register_commands(ctx: &Context, guild_id: GuildId) -> Result<(), serenity::Error> {
    // Register guild commands for faster testing
    // In production, you might want to use Command::create_global_application_command
    guild_id.set_application_commands(&ctx.http, |commands| {
        commands.create_application_command(|command| poll::create_poll_command(command))
    }).await?;

    Ok(())
}

pub async fn handle_command(
    database: &Database,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match command.data.name.as_str() {
        "poll" => poll::handle_poll_command(database, ctx, command).await?,
        _ => {
            command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(serenity::model::application::interaction::InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content("Unknown command")
                        })
                })
                .await?;
        }
    }

    Ok(())
}
