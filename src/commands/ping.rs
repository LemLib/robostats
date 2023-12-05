use serenity::builder::CreateCommand;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::client::Context;
use serenity::model::application::CommandInteraction;

pub fn run(_ctx: &Context, _interaction: &CommandInteraction) -> CreateInteractionResponse {
    let message = CreateInteractionResponseMessage::new().content("Pong!");

    CreateInteractionResponse::Message(message)
}

pub fn register() -> CreateCommand {
    CreateCommand::new("ping").description("Ping the bot")
}