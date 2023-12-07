use serenity::all::{CommandDataOptionValue, CommandOptionType, CreateMessage, ReactionType};
use serenity::builder::{
    CreateActionRow, CreateCommand, CreateCommandOption, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption,
};
use serenity::client::Context;
use serenity::model::application::CommandInteraction;
use serenity::model::Color;

use crate::api::robotevents::client::RobotEvents;
use crate::api::vrc_data_analysis::client::VRCDataAnalysis;
use crate::commands::teaminfo::embeds::create_general_embed;

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
            return CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content("Invalid team number."),
            );
        };

    let program: &i64 =
        if &interaction.data.options.len() < &2 {
            &1i64
        } else if let CommandDataOptionValue::Integer(number) = &interaction.data.options[1].value {
            number
        } else {
            return CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content("Invalid program value."),
            );
        };
            let result = create_general_embed(team_number, program, robotevents, vrc_data_analysis).await;


            let message = match result {
                Ok((a, b)) => CreateInteractionResponseMessage::new().components(b).embed(a),
                Err(s) => CreateInteractionResponseMessage::new().content(s)
            };
            CreateInteractionResponse::Message(message)
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
        )
}
