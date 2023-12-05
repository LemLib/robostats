use serenity::builder::{
    CreateCommand,
    CreateEmbed,
    CreateCommandOption,
    CreateInteractionResponse,
    CreateInteractionResponseMessage,
    CreateActionRow,
    CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption
};
use serenity::all::{CommandOptionType, CommandDataOptionValue, ReactionType};
use serenity::client::Context;
use serenity::model::application::CommandInteraction;

pub fn response(_ctx: &Context, interaction: &CommandInteraction) -> CreateInteractionResponse {
    let team_number = if let CommandDataOptionValue::String(number) = &interaction.data.options[0].value {
        Some(number)
    } else {
        None
    };

    let page_menu = CreateSelectMenu::new("team_page_select", CreateSelectMenuKind::String {
        options: vec![
            CreateSelectMenuOption::new("Team Info", "team")
                .emoji(ReactionType::Unicode("🗿".to_string()))
                .description("General information about the team")
                .default_selection(true),
            CreateSelectMenuOption::new("Awards", "awards")
                .emoji(ReactionType::Unicode("🏆".to_string()))
                .description("Awards from events throughout the season"),
            CreateSelectMenuOption::new("Skills", "skills")
                .emoji(ReactionType::Unicode("📄".to_string()))
                .description("Skills scores"),
            CreateSelectMenuOption::new("Trueskill", "trueskill")
                .emoji(ReactionType::Unicode("📊".to_string()))
                .description("TrueSkill ranking from vrc-data-analysis"),
            CreateSelectMenuOption::new("Events", "events")
                .emoji(ReactionType::Unicode("🗓️".to_string()))
                .description("Event attendance from this team"),
        ],
    });

    //Season menu will be determined using data from robotevents
    let season_menu = CreateSelectMenu::new("team_season_select", CreateSelectMenuKind::String {
        options: vec![
            CreateSelectMenuOption::new("Season 1", "opt_1")
                .description("2022-2023"),
            CreateSelectMenuOption::new("Season 2", "opt_2")
                .description("2023-2024")
                .default_selection(true),
        ],
    });

    let message = CreateInteractionResponseMessage::new()
        .components(vec![
            CreateActionRow::SelectMenu(page_menu),
            CreateActionRow::SelectMenu(season_menu),
        ])
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
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "team", "Team Number").required(true)
        )
}
