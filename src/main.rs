#![feature(let_chains)]

use std::any::Any;
use api::robotevents::client::RobotEvents;
use shuttle_secrets::SecretStore;

use serenity::all::{Command, ComponentInteractionDataKind};
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
        } else if let Interaction::Component(mut component) = interaction {
            let data = match &component.data.kind {
                ComponentInteractionDataKind::StringSelect {
                    values,
                } => &values[0],
                _ => return
            };
            match data.as_str() {
                "team_page" | "awards_page" | "stats_page" | "events_page" => {

                    let embed = component.message.embeds.first().expect("").clone();

                    let b = embed.title.expect("").clone();
                    let mut split = b.split(" ");

                    let team_num = split.next().expect("");
                    let program = match split.next().expect("") {
                        "(VRC" => 1i64,
                        "(VEXU" => 4i64,
                        "(VIQRC" => 41i64,
                        "(TSA" => {
                            match embed.colour.expect("").tuple() {
                                (210,38,48) => 46i64,
                                (0,119,200) => 47i64,
                                _ => 0i64
                            }
                        },
                        "(VAIRC" => 57i64,
                        _ => 0i64
                    };

                    let _ = component.message.edit(ctx.clone(), commands::teaminfo::team::edit(&ctx.clone(), data, team_num, &program, &self.robotevents, &self.vrc_data_analysis).await).await;
                    component.create_response(ctx, CreateInteractionResponse::Acknowledge).await.expect("");
                }
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