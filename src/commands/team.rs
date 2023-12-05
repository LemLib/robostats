use serenity::builder::{CreateCommand, CreateEmbed, CreateCommandOption};
use serenity::model::application::{ResolvedOption, ResolvedValue};
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};

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
        .set_options(vec![
            CreateCommandOption::new(serenity::all::CommandOptionType::String, "Team Number", "Team Number")
        ])
}