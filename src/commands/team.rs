use serenity::builder::{
    CreateCommand,
    CreateEmbed,
    CreateCommandOption,
    CreateInteractionResponse,
    CreateInteractionResponseMessage
};
use serenity::all::{CommandOptionType, CommandDataOptionValue};
use serenity::client::Context;
use serenity::model::application::CommandInteraction;

pub fn run(_ctx: &Context, interaction: &CommandInteraction) -> CreateInteractionResponse {
    let team_number = if let CommandDataOptionValue::String(number) = &interaction.data.options[0].value {
        Some(number)
    } else {
        None
    };

    let message = CreateInteractionResponseMessage::new()
        .add_embed(
            CreateEmbed::new()
                .title(format!("Team {}", team_number.unwrap()))
                .description("testing")
        );

    CreateInteractionResponse::Message(message)
}

pub fn register() -> CreateCommand {
    CreateCommand::new("team")
        .description("Embed Test")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "team", "Team Number").required(true)
        )
}