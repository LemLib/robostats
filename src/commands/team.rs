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

use crate::api::robotevents::client::RobotEvents;

pub async fn response(_ctx: &Context, interaction: &CommandInteraction, robotevents: &RobotEvents) -> CreateInteractionResponse {
    let team_number = if let CommandDataOptionValue::String(number) = &interaction.data.options[0].value {
        number
    } else {
        return CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content("Invalid team number."))
    };

    if let Ok(teams) = robotevents.find_teams(team_number).await {
        if let Some(team) = teams.iter().next() {
            let team = team.clone();

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
                        .title(format!("Team {}", team.number))
                        .description(team.team_name)
                        .field("Organization", team.organization, true)
                        .field("Grade", team.grade, true)    
                        .field("Registered", format!("{}", team.registered), true)
                );

            CreateInteractionResponse::Message(message)   
        } else {
            return CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content("Failed to get information about team from RobotEvents."))
        }
    } else {
        return CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content("Failed to connect to RobotEvents."))
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("team")
        .description("Embed Test")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "team", "Team Number").required(true)
        )
}
