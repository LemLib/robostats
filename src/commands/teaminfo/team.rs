use serenity::all::{CommandDataOptionValue, CommandOptionType, EditMessage};
use serenity::all::CreateInteractionResponse::Message;
use serenity::builder::{
    CreateCommand, CreateCommandOption, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use serenity::client::Context;
use serenity::model::application::CommandInteraction;

use crate::api::robotevents::client::RobotEvents;
use crate::api::vrc_data_analysis::client::VRCDataAnalysis;
use crate::commands::teaminfo::embeds::{create_awards_embed, create_events_embed, create_general_embed, create_stats_embed};

pub async fn response(
    _ctx: &Context,
    interaction: &CommandInteraction,
    robotevents: &RobotEvents,
    vrc_data_analysis: &VRCDataAnalysis,
) -> CreateInteractionResponse {
    let team_number = if let CommandDataOptionValue::String(number) = &interaction.data.options[0].value {
        number
    } else {
        return CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content("Invalid team number."));
    };
    let arg_program = interaction.data.options.iter().find(|a| a.name == "program");
    let arg_page = interaction.data.options.iter().find(|b| b.name == "page");
    let program: i64 = match arg_program.clone() {
        None => 1,
        Some(n) => n.value.as_i64().unwrap_or(1)
    };
    let result = if let Ok(teams) = robotevents.find_teams(team_number, &program).await {
        if let Some(team) = teams.iter().next() {
            let page = match arg_page.clone() {
                None => "general",
                Some(s) => s.value.as_str().unwrap_or("invalid")
            };
            match page {
                "general" => create_general_embed(team.clone(), robotevents, vrc_data_analysis, true).await,
                "awards" => create_awards_embed(team.clone(), robotevents, true).await,
                "stats" => create_stats_embed(team.clone(), robotevents, true).await,
                "events" => create_events_embed(team.clone(), robotevents, true).await,
                _ => (None, None, Some("How did we get here? (Invalid page)".to_string()))
            }
        } else {
            (None, None, Some("Failed to get information about team from RobotEvents".to_string()))
        }
    } else {
        (None, None, Some("Failed to get information from RobotEvents".to_string()))
    };
    let mut message = CreateInteractionResponseMessage::new();
    if result.0.is_some() { message = message.embed(result.0.unwrap()); }
    if result.1.is_some() { message = message.components(result.1.unwrap()); }
    if result.2.is_some() { message = message.content(result.2.unwrap()); }
    return Message(message);
}

pub async fn edit(
    _ctx: &Context,
    page: &str,
    team_number: &str,
    program: &i64,
    robotevents: &RobotEvents,
    vrc_data_analysis: &VRCDataAnalysis,
) -> EditMessage {
    let result = if let Ok(teams) = robotevents.find_teams(team_number, &program).await {
        if let Some(team) = teams.iter().next() {
            match page {
                "team_page" => create_general_embed(team.clone(), robotevents, vrc_data_analysis, true).await,
                "awards_page" => create_awards_embed(team.clone(), robotevents, true).await,
                "stats_page" => create_stats_embed(team.clone(), robotevents, true).await,
                "events_page" => create_events_embed(team.clone(), robotevents, true).await,
                _ => (None, None, Some("How did we get here? (Invalid page)".to_string()))
            }
        } else {
            (None, None, Some("Failed to get information about team from RobotEvents".to_string()))
        }
    } else {
        (None, None, Some("Failed to get information from RobotEvents".to_string()))
    };
    let mut message = EditMessage::new();
    if result.0.is_some() { message = message.embed(result.0.unwrap()); }
    if result.1.is_some() { message = message.components(result.1.unwrap()); }
    if result.2.is_some() { message = message.content(result.2.unwrap()); }
    return message;
}

pub fn register() -> CreateCommand {
    CreateCommand::new("team").description("Displays information about a team").add_option(
        CreateCommandOption::new(CommandOptionType::String, "team", "Team Number").required(true),
    ).add_option(
        CreateCommandOption::new(CommandOptionType::Integer, "program", "Program Name").required(false)
            .add_int_choice("VRC", 1).add_int_choice("VEXU", 4).add_int_choice("VIQRC", 41).add_int_choice("TSA VRC", 46).add_int_choice("TSA VIQRC", 47).add_int_choice("VAIRC", 57).max_int_value(57).min_int_value(1)
    ).add_option(
        CreateCommandOption::new(CommandOptionType::String, "page", "The page to open to").required(false).add_string_choice("General Info", "general").add_string_choice("Awards", "awards").add_string_choice("Stats", "stats").add_string_choice("Event Record", "events")
    )
}
