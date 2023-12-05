use shuttle_secrets::SecretStore;

use serenity::all::Command;
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::Interaction;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

mod commands;

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        // Register slash commands
        Command::set_global_commands(&ctx.http, vec![
            commands::ping::register(),
            commands::team::register()
        ]);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let response: Option<CreateInteractionResponse> = match command.data.name.as_str() {
                "ping" => Some(commands::ping::response(&ctx, &command)),
                "team" => Some(commands::team::response(&ctx, &command)),
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

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secrets: SecretStore
) -> shuttle_serenity::ShuttleSerenity {
    // Configure the client with your Discord bot token in the environment.
    let token = secrets.get("DISCORD_TOKEN").expect("Couldn't find DISCORD_TOKEN in SecretStore. Do you have a Secrets.toml?");

    // Build client with token and default intents.
    let mut client = Client::builder(token, GatewayIntents::empty()).event_handler(Bot).await.expect("Error creating client");

    Ok(client.into());
}