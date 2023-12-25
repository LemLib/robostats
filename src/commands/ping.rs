use serenity::all::CreateEmbed;
use serenity::builder::CreateInteractionResponseMessage;
use serenity::builder::CreateCommand;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct PingCommand;

impl PingCommand {
    pub fn command() -> CreateCommand {
        CreateCommand::new("ping").description("Ping the bot")
    }

    pub fn response(&self) -> CreateInteractionResponseMessage {
        CreateInteractionResponseMessage::new()
            .add_embed(
                CreateEmbed::new()
                    .title("Pong!")
            )
    }
}