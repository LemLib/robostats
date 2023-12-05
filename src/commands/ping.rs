use serenity::all::CreateEmbed;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::builder::CreateCommand;
use serenity::client::Context;
use serenity::model::application::CommandInteraction;

pub fn response(_ctx: &Context, _interaction: &CommandInteraction) -> CreateInteractionResponse {
    let message = CreateInteractionResponseMessage::new()
        .add_embed(
            CreateEmbed::new()
                .title("Pong!")
        );

    CreateInteractionResponse::Message(message)
}

pub fn register() -> CreateCommand {
    CreateCommand::new("ping").description("Ping the bot")
}