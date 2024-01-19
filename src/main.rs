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
    skills::SkillsCache,
};
use robotevents::{
    RobotEvents,
    schema::{PaginatedResponse, IdInfo, Season},
    query::{SeasonsQuery, PaginatedQuery},
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
        if self.program_list.is_err() {
            println!("Failed to fetch program list from RobotEvents. Command functionality may be limited as a result.");
        }
        if self.season_list.is_err() {
            println!("Failed to fetch season list from RobotEvents. Command functionality may be limited as a result.");
        }

        // Needs to be done in separate calls because of discord request character limit for command registration.
        Command::create_global_command(&ctx.http, WikiCommand::command()).await.expect("Failed to register wiki command.");
        Command::create_global_command(&ctx.http, TeamCommand::command(self.program_list.clone().ok())).await.expect("Failed to register team command.");
        Command::create_global_command(&ctx.http, PingCommand::command()).await.expect("Failed to register ping command.");
        Command::create_global_command(&ctx.http, PredictCommand::command()).await.expect("Failed to register predict command.");
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
            season_list: robotevents.seasons(SeasonsQuery::default().per_page(250)).await.map_err(|_| BotRequestError),
            robotevents,
            vrc_data_analysis,
            skills_cache: SkillsCache::default(),
        })
        .await
        .expect("Error creating client");

    Ok(client.into())
}
