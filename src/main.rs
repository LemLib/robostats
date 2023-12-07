#![feature(let_chains)]

use api::robotevents::client::RobotEvents;
use shuttle_secrets::SecretStore;

use serenity::all::Command;
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::Interaction;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use crate::api::vrc_data_analysis::client::VRCDataAnalysis;

mod commands;
mod api;

struct Bot {
    robotevents: RobotEvents,
    vrc_data_analysis: VRCDataAnalysis,
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        // Register slash commands
        Command::set_global_commands(&ctx.http, vec![
            commands::ping::register(),
            commands::teaminfo::team::register(),
            commands::wiki::register()
        ]).await.expect("Failed to register slash commands.");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let response = match command.data.name.as_str() {
                "ping" => Some(commands::ping::response(&ctx, &command)),
                "team" => Some(commands::teaminfo::team::response(&ctx, &command, &self.robotevents, &self.vrc_data_analysis).await),
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
        } else if let Interaction::Component(component) = interaction {
            match component.data.custom_id.as_str() {
                "team_page_response" => {}
                "team_page_response" => {}
                "team_page_response" => {}
                "team_page_response" => {}
                _ => {}
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
            robotevents: RobotEvents::new(robotevents_token),
            vrc_data_analysis: VRCDataAnalysis::new(),
        })
        .await
        .expect("Error creating client");

    Ok(client.into())
}