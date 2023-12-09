use serenity::all::{CommandDataOptionValue, CommandOptionType, ComponentInteraction, CreateActionRow, CreateEmbed, EditInteractionResponse, EditMessage};
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
    let team_number =
        if let CommandDataOptionValue::String(number) = &interaction.data.options[0].value {
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
    let page = match arg_page.clone() {
        None => "general",
        Some(s) => s.value.as_str().unwrap_or("invalid")
    };
    let result = match page {
        "general" => create_general_embed(team_number, &program, robotevents, vrc_data_analysis, true).await,
        "awards" => create_awards_embed(team_number, &program, robotevents, vrc_data_analysis, true).await,
        "stats" => create_stats_embed(team_number, &program, robotevents, vrc_data_analysis, true).await,
        "events" => create_events_embed(team_number, &program, robotevents, vrc_data_analysis, true).await,
        _ => Err::<(CreateEmbed, Option<Vec<CreateActionRow>>), String>("Invalid page (How did we get here?)".to_string())
    };


    let message = match result {
        Ok((a, b)) => {
            let mut response = CreateInteractionResponseMessage::new().embed(a);
            if b.is_some() { response = response.components(b.unwrap()); }
            response
        }
        Err(s) => CreateInteractionResponseMessage::new().content(s)
    };
    return CreateInteractionResponse::Message(message);
}

pub async fn edit(
    _ctx: &Context,
    page : &str,
    team_number : &str,
    program : &i64,
    robotevents: &RobotEvents,
    vrc_data_analysis: &VRCDataAnalysis,
) -> EditMessage {
    let result = match page {
        "team_page" => create_general_embed(team_number, program, robotevents, vrc_data_analysis, false).await,
        "awards_page" => create_awards_embed(team_number, program, robotevents, vrc_data_analysis, false).await,
        "stats_page" => create_stats_embed(team_number, program, robotevents, vrc_data_analysis, false).await,
        "events_page" => create_events_embed(team_number, program, robotevents, vrc_data_analysis, false).await,
        _ => Err::<(CreateEmbed, Option<Vec<CreateActionRow>>), String>("Invalid page (How did we get here?)".to_string())
    };

    let message = match result {
        Ok((a, _)) => {
            let response = EditMessage::new().embed(a);
            response
        }
        Err(s) => EditMessage::new().content(s)
    };

    return message;
}

pub fn register() -> CreateCommand {
    CreateCommand::new("team")
        .description("Displays information about a team")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "team", "Team Number")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::Integer, "program", "Program Name")
                .required(false)
                //these integer values are the program ids
                .add_int_choice("VRC", 1)
                .add_int_choice("VEXU", 4)
                .add_int_choice("VIQRC", 41)
                .add_int_choice("TSA VRC", 46)
                .add_int_choice("TSA VIQRC", 47)
                .add_int_choice("VAIRC", 57)
                .max_int_value(57)
                .min_int_value(1)
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "page", "The page to open to")
                .required(false)
                .add_string_choice("General Info", "general")
                .add_string_choice("Awards", "awards")
                .add_string_choice("Stats", "stats")
                .add_string_choice("Event Record", "events")
        )
}
