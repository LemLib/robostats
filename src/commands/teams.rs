use serenity::builder::{CreateCommand, CreateEmbed};
use serenity::model::application::ResolvedOption;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};

pub fn run(_options: &[ResolvedOption]) -> CreateInteractionResponse {
    let message = CreateInteractionResponseMessage::new()
        .add_embed(
            CreateEmbed::new()
                .title("Team")
                .description("testing")
        );

    CreateInteractionResponse::Message(message)
}

pub fn register() -> CreateCommand {
    CreateCommand::new("team").description("Embed Test")
}