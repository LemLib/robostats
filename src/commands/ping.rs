use serenity::builder::CreateCommand;
use serenity::model::application::ResolvedOption;

pub fn run(_options: &[ResolvedOption]) -> String {
    return "Pong!".to_string()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("ping").description("Ping the bot")
}