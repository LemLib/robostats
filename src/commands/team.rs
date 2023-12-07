use serenity::all::{CommandDataOptionValue, CommandOptionType, ReactionType};
use serenity::builder::{
    CreateActionRow, CreateCommand, CreateCommandOption, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption,
};
use serenity::client::Context;
use serenity::model::application::CommandInteraction;
use serenity::model::Color;

use crate::api::robotevents::client::RobotEvents;
use crate::api::vrc_data_analysis::client::VRCDataAnalysis;

pub async fn response(
    _ctx: &Context,
    interaction: &CommandInteraction,
    robotevents: &RobotEvents,
    vrc_data_analysis: &VRCDataAnalysis,
) -> CreateInteractionResponse {
    let team_number =
        if let CommandDataOptionValue::String(number) = &interaction.data.options[0].value {
            number
        } else {
            return CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content("Invalid team number."),
            );
        };

    let program: &i64 =
        if &interaction.data.options.len() < &2 {
            &1i64
        } else if let CommandDataOptionValue::Integer(number) = &interaction.data.options[1].value {
            number
        } else {
            return CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content("Invalid program value."),
            );
        };

    let trueskill = if *program != 1i64 {
        "Trueskill not supported".to_string()
    } else if let Ok(data_analysis) = vrc_data_analysis.team_info(team_number).await {
        data_analysis.trueskill_ranking.to_string()
    } else {
        "No ranking".to_string()
    };

    if let Ok(teams) = robotevents.find_teams(team_number, program).await {
        if let Some(team) = teams.iter().next() {
            let team = team.clone();

            let mut message_components: Vec<CreateActionRow> = vec![
                CreateActionRow::SelectMenu(CreateSelectMenu::new(
                    "team_page_select",
                    CreateSelectMenuKind::String {
                        options: vec![
                            CreateSelectMenuOption::new("Team Info", "team_oage")
                                .emoji(ReactionType::Unicode("🗿".to_string()))
                                .description("General information about the team")
                                .default_selection(true),
                            CreateSelectMenuOption::new("Awards", "awards_page")
                                .emoji(ReactionType::Unicode("🏆".to_string()))
                                .description("Awards from events throughout the season"),
                            CreateSelectMenuOption::new("Stats", "stats_page")
                                .emoji(ReactionType::Unicode("📊".to_string()))
                                .description("Team statistics & rankings"),
                            CreateSelectMenuOption::new("Events", "events_page")
                                .emoji(ReactionType::Unicode("🗓️".to_string()))
                                .description("Event attendance from this team"),
                        ],
                    },
                ))
            ];

            if let Ok(seasons) = robotevents.team_active_seasons(&team).await {
                message_components.push(CreateActionRow::SelectMenu(CreateSelectMenu::new(
                    "team_season_select",
                    CreateSelectMenuKind::String {
                        options: seasons
                            .iter()
                            .enumerate()
                            .map(|(i, season)| {
                                CreateSelectMenuOption::new(
                                    &season.name,
                                    format!("option_season_{}", season.id),
                                )
                                    .default_selection(i == 0)
                                    .description(format!("{}-{}", season.years_start, season.years_end))
                            })
                            .collect(),
                    },
                )));
            }

            let mut embed = CreateEmbed::new()
                .title(format!(
                    "{} ({} {})",
                    team.number, team.program.code, team.grade
                ))
                .url(format!(
                    "https://www.robotevents.com/teams/{}/{}",
                    team.program.code, team.number
                ))
                .description(team.team_name)
                .field("Organization", team.organization, true)
                .field(
                    "Location",
                    format!(
                        "{}, {}, {}",
                        team.location.city, team.location.region, team.location.country
                    ),
                    true,
                )
                .field("TrueSkill Ranking", trueskill, true)
                .field(
                    "Registered",
                    if team.registered { "Yes" } else { "No" },
                    true,
                )
                .color(match team.program.code.as_ref() {
                    "VRC" | "VEXU" | "TSA VRC" => Color::from_rgb(210, 38, 48),
                    "VIQRC" | "TSA VIQRC" => Color::from_rgb(0, 119, 200),
                    "VAIRC" => Color::from_rgb(00, 255, 00),
                    _ => Default::default(),
                });

            if let Some(robot_name) = team.robot_name {
                if !robot_name.is_empty() {
                    embed = embed.field("Robot Name", robot_name, true)
                }
            }

            let message = CreateInteractionResponseMessage::new().components(message_components).embed(embed);

            CreateInteractionResponse::Message(message)
        } else {
            return CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Failed to get information about team from RobotEvents."),
            );
        }
    } else {
        return CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new().content("Failed to connect to RobotEvents."),
        );
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("team")
        .description("Displays information about a team")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "team", "Team Number")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::Integer, "program", "Program Name")
                .required(false)
                //these integer values are the program ids
                .add_int_choice("VRC", 1)
                .add_int_choice("VEXU", 4)
                .add_int_choice("VIQRC", 41)
                .add_int_choice("TSA VRC", 46)
                .add_int_choice("TSA VIQRC", 47)
                .add_int_choice("VAIRC", 57)
        )
}
