use serenity::builder::{
    CreateCommand,
    CreateEmbed,
    CreateCommandOption,
    CreateInteractionResponse,
    CreateInteractionResponseMessage
};
use serenity::all::CommandOptionType;
use serenity::model::application::{ResolvedOption, ResolvedValue};

pub fn run(options: &[ResolvedOption]) -> CreateInteractionResponse {
    let team_number = if let ResolvedValue::String(number) = options[0].value {
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