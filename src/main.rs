use std::env;

use serenity::all::Command;
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::Interaction;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

mod commands;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        // Register slash commands
        if let Err(error) = Command::create_global_command(&ctx.http, commands::ping::register()).await {
            println!("Could not register ping slash command: {error}");
        }
        if let Err(error) = Command::create_global_command(&ctx.http, commands::team::register()).await {
            println!("Could not register team command: {error}");
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let response: Option<CreateInteractionResponse> = match command.data.name.as_str() {
                "ping" => Some(commands::ping::run(&ctx, &command)),
                "team" => Some(commands::team::run(&ctx, &command)),
                // "awards" => Some(commands::awards::run(&ctx, &command)),
                _ => {
                    let message = CreateInteractionResponseMessage::new().content("not implemented :(");

                    Some(CreateInteractionResponse::Message(message))
                },
            };

            // Attempt to send response
            if let Some(response) = response {
                if let Err(error) = command.create_response(&ctx.http, response).await {
                    println!("Cannot respond to slash command: {error}");
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Build our client.
    let mut client = Client::builder(token, GatewayIntents::empty()).event_handler(Handler).await.expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(error) = client.start().await {
        println!("Client error: {error:?}");
    }
}