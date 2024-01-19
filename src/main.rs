use api::robotevents::client::RobotEvents;
use shuttle_secrets::SecretStore;

use serenity::all::Command;
use serenity::all::Message;
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, CreateEmbed, CreateEmbedFooter};
use serenity::model::application::Interaction;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

mod commands;
mod api;

struct Bot {
    robotevents: RobotEvents,
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        // Register slash commands
        Command::set_global_commands(&ctx.http, vec![
            commands::ping::register(),
            commands::team::register(),
            commands::wiki::register()
        ]).await.expect("Failed to register slash commands.");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let response: Option<CreateInteractionResponse> = match command.data.name.as_str() {
                "ping" => Some(commands::ping::response(&ctx, &command)),
                "team" => Some(commands::team::response(&ctx, &command, &self.robotevents).await),
                "wiki" => Some(commands::wiki::response(&ctx, &command)),
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

    // On message
    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore messages from bots
        if msg.author.bot {
            return;
        }

        // Ignore messages that don't start with the prefix
        if msg.content.starts_with(ctx.cache.current_user().mention().to_string().as_str()) {
            let message = CreateMessage::new()
                .add_embed(
                    CreateEmbed::new()
                        .title("Hello there! 👋")
                        .description("RoboStats uses slash commands to operate. Try `/team`!")
                        .footer(
                            CreateEmbedFooter::new("Made with ❤️ by the VRC community.")
                        )
                );
            // Send message
            if let Err(error) = msg.channel_id.send_message(&ctx.http, message).await {
                println!("Cannot send message: {error}");
            }
        }
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secrets: SecretStore
) -> shuttle_serenity::ShuttleSerenity {
    let discord_token = secrets.get("DISCORD_TOKEN").expect("Couldn't find DISCORD_TOKEN in SecretStore. Do you have a Secrets.toml?");
    let robotevents_token = secrets.get("ROBOTEVENTS_TOKEN").expect("Couldn't find ROBOTEVENTS_TOKEN in SecretStore. Do you have a Secrets.toml?");

    // Build client with token and default intents.
    let client = Client::builder(discord_token, GatewayIntents::empty())
        .event_handler(Bot {
            robotevents: RobotEvents::new(robotevents_token)
        })
        .await
        .expect("Error creating client");

    Ok(client.into())
}