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
use serenity::all::{CommandOptionType, CommandDataOptionValue};
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
            CreateSelectMenuOption::new("Page 1", "opt_1"),
            CreateSelectMenuOption::new("Page 2", "opt_2"),
        ],
    });
    let season_menu = CreateSelectMenu::new("team_season_select", CreateSelectMenuKind::String {
        options: vec![
            CreateSelectMenuOption::new("Season 1", "opt_1"),
            CreateSelectMenuOption::new("Season 2", "opt_2"),
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