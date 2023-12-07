use serenity::all::{CommandDataOptionValue, CommandOptionType};
use serenity::builder::{
    CreateCommand, CreateCommandOption, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use serenity::client::Context;
use serenity::model::application::CommandInteraction;

use crate::api::robotevents::client::RobotEvents;
use crate::api::vrc_data_analysis::client::VRCDataAnalysis;
use crate::commands::teaminfo::embeds::{create_awards_embed, create_general_embed};

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
    let program: i64 =
        if arg_program.is_none() {
            1i64
        } else if let CommandDataOptionValue::Integer(number) = arg_program.cloned().unwrap().value {
            number
        } else {
            return CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content("Invalid program value."));
        };
    let page: &str =
        if arg_page.is_none() {
            "general"
        } else if let CommandDataOptionValue::String(str) = arg_page.cloned().unwrap().value {
            str.as_str()
        } else {
            return CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content("Invalid page."));
        };

    let result = match page {
        "general" => create_general_embed(team_number, &program, robotevents, vrc_data_analysis, true).await,
        "awards" => create_awards_embed(team_number, &program, robotevents, vrc_data_analysis, true).await,
        _ => Err("Invalid page (How did we get here?)")
    };


    let message = match result {
        Ok((a, b)) => {
            let response = CreateInteractionResponseMessage::new().embed(a);
            if b.is_some() { response.components(b.unwrap()); }
            response
        }
        Err(s) => CreateInteractionResponseMessage::new().content(s)
    };
    return CreateInteractionResponse::Message(message);
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
