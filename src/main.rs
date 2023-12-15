use std::time::Duration;

use api::robotevents::client::RobotEvents;
use api::vrc_data_analysis;
use commands::wiki;
use serenity::futures::StreamExt;
use shuttle_secrets::SecretStore;

use crate::api::vrc_data_analysis::client::VRCDataAnalysis;
use crate::commands::{PingCommand, TeamCommand};

use serenity::{
    prelude::*,
    all::Command,
    async_trait,
    builder::{CreateInteractionResponse, CreateInteractionResponseMessage},
    model::{
        application::Interaction,
        gateway::Ready,
    }
};

mod api;
mod commands;

struct Bot {
    robotevents: RobotEvents,
    vrc_data_analysis: VRCDataAnalysis,
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        // Register slash commands
        Command::set_global_commands(
            &ctx.http,
            vec![
                PingCommand::command(),
                TeamCommand::command(),
                commands::wiki::register(),
            ],
        )
        .await
        .expect("Failed to register slash commands.");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                // Some commands store persistent data across component interactions, and thus require an
                // instance to be created for them ahead of time.
                let mut team_command = TeamCommand::default();
                let ping_command = PingCommand::default();

                // Generate a response messaage for a given command type.
                let response_message = match command.data.name.as_str() {
                    "ping" => {
                        ping_command.response()
                    },
                    "team" => {
                        team_command.response(&ctx, &command, &self.robotevents).await
                    },
                    "wiki" => {
                        wiki::response(&ctx, &command)
                    },
                    _ => {
                        CreateInteractionResponseMessage::new().content("not implemented :(")
                    }
                };
                
                // Send initial response message to user's command.
                if let Err(error) = command.create_response(&ctx.http, CreateInteractionResponse::Message(response_message)).await {
                    println!("Failed to respond to {} command: {error}", command.data.name.as_str());
                }
                
                // Wait for component interactions and handle them according to the respective command.
                if let Ok(response) = command.get_response(&ctx.http).await {
                    let mut interaction_stream =
                        response.await_component_interaction(&ctx.shard).timeout(Duration::from_secs(60 * 3)).stream();

                    while let Some(component_interaction) = interaction_stream.next().await {
                        match command.data.name.as_str() {
                            "team" => {
                                component_interaction.create_response(
                                    &ctx,
                                    team_command.component_interaction_response(&ctx, &command, &component_interaction, &self.robotevents).await
                                ).await.unwrap_or(());
                            },
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    let discord_token = secrets
        .get("DISCORD_TOKEN")
        .expect("Couldn't find DISCORD_TOKEN in SecretStore. Do you have a Secrets.toml?");
    let robotevents_token = secrets
        .get("ROBOTEVENTS_TOKEN")
        .expect("Couldn't find ROBOTEVENTS_TOKEN in SecretStore. Do you have a Secrets.toml?");

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
