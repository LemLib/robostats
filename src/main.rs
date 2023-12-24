use std::time::Duration;


use shuttle_secrets::SecretStore;
use serenity::{
    prelude::*,
    async_trait,
    all::Command,
    futures::StreamExt,
    builder::{CreateInteractionResponse, CreateInteractionResponseMessage},
    model::{
        application::Interaction,
        gateway::Ready,
    }
};

use commands::{
    PingCommand,
    PredictCommand,
    TeamCommand,
    WikiCommand,
};
use api::{
    vrc_data_analysis::VRCDataAnalysis,
    skills_cache::SkillsCache,
};
use robotevents::{
    RobotEvents,
    schema::{PaginatedResponse, IdInfo, Season},
    filters::SeasonsFilter,
};

mod api;
mod commands;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct BotRequestError;

struct Bot {
    robotevents: RobotEvents,
    vrc_data_analysis: VRCDataAnalysis,
    skills_cache: SkillsCache,
    season_list: Result<PaginatedResponse<Season>, BotRequestError>,
    program_list: Result<PaginatedResponse<IdInfo>, BotRequestError>
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        // Log warnings in the event that initial list fetching failed on startup.
        if let Err(_) = self.program_list {
            println!("Failed to fetch program list from RobotEvents. Command functionality may be limited as a result.");
        }
        if let Err(_) = self.season_list {
            println!("Failed to fetch season list from RobotEvents. Command functionality may be limited as a result.");
        }

        // Needs to be done in separate calls because of discord request character limit for command registration.
        Command::create_global_command(&ctx.http, WikiCommand::command()).await.expect("Failed to register wiki command.");
        Command::create_global_command(&ctx.http, TeamCommand::command(self.program_list.clone().ok())).await.expect("Failed to register team command.");
        Command::create_global_command(&ctx.http, PingCommand::command()).await.expect("Failed to register ping command.");
        Command::create_global_command(&ctx.http, PredictCommand::command()).await.expect("Failed to register predict command.");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                // Some commands store persistent data across component interactions, and thus require an
                // instance to be created for them ahead of time.
                let mut team_command = TeamCommand::default();
                let predict_command = PredictCommand::default();
                let ping_command = PingCommand::default();
                let wiki_command = WikiCommand::default();

                // Generate a response messaage for a given command type.
                let response_message = match command.data.name.as_str() {
                    "ping" => {
                        ping_command.response()
                    },
                    "predict" => {
                        predict_command.response(&ctx, &command, &self.vrc_data_analysis).await
                    }
                    "team" => {
                        team_command.response(&ctx, &command, &self.robotevents, &self.vrc_data_analysis, &self.skills_cache).await
                    },
                    "wiki" => {
                        wiki_command.response(&ctx, &command)
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
                    // Each command gets a 3 minute event listener for interactions with components.
                    let mut interaction_stream =
                        response.await_component_interaction(&ctx.shard).timeout(Duration::from_secs(60 * 3)).stream();

                    while let Some(component_interaction) = interaction_stream.next().await {
                        match command.data.name.as_str() {
                            "team" => {
                                component_interaction.create_response(
                                    &ctx,
                                    team_command.component_interaction_response(&ctx, &command, &component_interaction, &self.robotevents, &self.vrc_data_analysis, &self.skills_cache).await
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

    // HTTP clients for RobotEvents and vrc-data-analysis
    let robotevents = RobotEvents::new(robotevents_token);
    let vrc_data_analysis = VRCDataAnalysis::new();

    // Build client with token and default intents.
    let client = Client::builder(discord_token, GatewayIntents::empty())
        .event_handler(Bot {
            // Fetch a list of all seasons and programs from RobotEvents
            // We store these as Result<T, E> internally so HTTP fails don't prevent the bot from starting.
            program_list: robotevents.programs().await.map_err(|_| BotRequestError),
            season_list: robotevents.seasons(SeasonsFilter::default()).await.map_err(|_| BotRequestError),
            robotevents,
            vrc_data_analysis,
            skills_cache: SkillsCache::default(),
        })
        .await
        .expect("Error creating client");

    Ok(client.into())
}
