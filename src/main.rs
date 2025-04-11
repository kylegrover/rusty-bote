mod commands;
mod db;
mod handlers;
mod models;
mod utils;
mod voting;

use log::{error, info};
use serenity::async_trait;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;
use std::env;

struct RustyBote {
    database: db::Database,
}

#[async_trait]
impl EventHandler for RustyBote {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        // Register commands - for testing, register to a specific guild
        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = commands::register_commands(&ctx, guild_id).await;
        match commands {
            Ok(_) => info!("Successfully registered commands"),
            Err(e) => error!("Failed to register commands: {}", e),
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                info!("Received command interaction: {}", command.data.name);

                let response = commands::handle_command(&self.database, &ctx, &command).await;

                if let Err(e) = response {
                    error!("Failed to handle command: {}", e);
                    if let Err(e) = command
                        .create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| {
                                    message.content("An error occurred while processing the command.")
                                })
                        })
                        .await
                    {
                        error!("Failed to send error response: {}", e);
                    }
                }
            }
            Interaction::MessageComponent(component) => {
                info!("Received component interaction: {}", component.data.custom_id);
                
                if let Err(e) = handlers::handle_component(&self.database, &ctx, &component).await {
                    error!("Failed to handle component: {}", e);
                }
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load .env file if available
    dotenvy::dotenv().ok();
    env_logger::init();

    // Initialize database
    let database = db::Database::new().await?;
    
    // Get Discord token from environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a Discord token in the environment");

    // Create the client
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MESSAGE_REACTIONS;
    let mut client = Client::builder(&token, intents)
        .event_handler(RustyBote { database })
        .await?;

    // Start the client
    info!("Starting bot...");
    client.start().await?;
    Ok(())
}
