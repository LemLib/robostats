use serenity::builder::CreateCommand;
use serenity::model::application::ResolvedOption;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};

pub fn run(_options: &[ResolvedOption]) -> CreateInteractionResponse {
    let message = CreateInteractionResponseMessage::new().content("Pong!");

    CreateInteractionResponse::Message(message)
}

pub fn register() -> CreateCommand {
    CreateCommand::new("ping").description("Ping the bot")
}