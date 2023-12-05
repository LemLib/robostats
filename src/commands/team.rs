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
                .field("Organization", "Team Org", true)    
                .field("Grade", "Grade Level", true)    
                .field("Active", "Active Status", true)
        );

    CreateInteractionResponse::Message(message)
}

pub fn register() -> CreateCommand {
    CreateCommand::new("team")
        .description("Embed Test")
        .set_options(vec![
            CreateCommandOption::new(CommandOptionType::String, "Team Number", "Team Number")
        ])
}
