use serenity::all::{Color, CreateActionRow, CreateEmbed, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption, ReactionType};

use crate::api::robotevents::client::RobotEvents;
use crate::api::robotevents::schema::{Season, Team};
use crate::api::vrc_data_analysis::client::VRCDataAnalysis;

pub async fn create_interactions(team: Team, seasons: Vec<Season>) -> Vec<CreateActionRow> {
    let mut components = vec![
        CreateActionRow::SelectMenu(CreateSelectMenu::new(
            "team_page_select",
            CreateSelectMenuKind::String {
                options: vec![
                    CreateSelectMenuOption::new("Team Info", "team_page")
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
    components.push(CreateActionRow::SelectMenu(CreateSelectMenu::new(
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
    components
}

pub async fn create_general_embed(team_number: &str, program: &i64, robotevents: &RobotEvents,
                                  vrc_data_analysis: &VRCDataAnalysis) -> Result<(CreateEmbed, Vec<CreateActionRow>), String> {
    let trueskill = if *program != 1i64 {
        "Not supported for program".to_string()
    } else if let Ok(data_analysis) = vrc_data_analysis.team_info(team_number).await {
        data_analysis.trueskill_ranking.to_string()
    } else {
        "No ranking".to_string()
    };

    if let Ok(teams) = robotevents.find_teams(team_number, program).await {
        if let Some(team) = teams.iter().next() {
            let team = team.clone();
            let mut message_components: Vec<CreateActionRow> = if let Ok(seasons) = robotevents.team_active_seasons(&team).await {
                create_interactions(team.clone(), seasons).await
            } else {
                create_interactions(team.clone(), Vec::new()).await
            };

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

            return Ok((embed, message_components));
        } else {
            return Err("Failed to get information about team from RobotEvents".to_string());
        }
    } else {
        return Err("Failed to get information from RobotEvents".to_string());
    }
}

pub async fn create_awards_embed(team_number: &str, program: &i64, robotevents: &RobotEvents,
                                 vrc_data_analysis: &VRCDataAnalysis) -> Result<(CreateEmbed, Vec<CreateActionRow>), String> {
    if let Ok(teams) = robotevents.find_teams(team_number, program).await {
        if let Some(team) = teams.iter().next() {
            let team = team.clone();
            let mut message_components: Vec<CreateActionRow> = if let Ok(seasons) = robotevents.team_active_seasons(&team).await {
                create_interactions(team.clone(), seasons).await
            } else {
                create_interactions(team.clone(), Vec::new()).await
            };

            let mut embed = CreateEmbed::new()
                .title(format!(
                    "{} ({} {}) Awards",
                    team.number, team.program.code, team.grade
                ))
                .url(format!(
                    "https://www.robotevents.com/teams/{}/{}",
                    team.program.code, team.number
                ))
                .description(team.team_name)
                .color(match team.program.code.as_ref() {
                    "VRC" | "VEXU" | "TSA VRC" => Color::from_rgb(210, 38, 48),
                    "VIQRC" | "TSA VIQRC" => Color::from_rgb(0, 119, 200),
                    "VAIRC" => Color::from_rgb(00, 255, 00),
                    _ => Default::default(),
                });

            return Ok((embed, message_components));
        } else {
            return Err("Failed to get information about team from RobotEvents".to_string());
        }
    } else {
        return Err("Failed to get information from RobotEvents".to_string());
    }
}