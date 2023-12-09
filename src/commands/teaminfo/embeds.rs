use serenity::all::{Color, CreateActionRow, CreateEmbed, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption, ReactionType};

use crate::api::robotevents::client::RobotEvents;
use crate::api::robotevents::schema::{Season, Team};
use crate::api::vrc_data_analysis::client::VRCDataAnalysis;

pub async fn create_interactions(seasons: Vec<Season>) -> Vec<CreateActionRow> {
    let mut components = vec![
        CreateActionRow::SelectMenu(CreateSelectMenu::new(
            "team_page_select",
            CreateSelectMenuKind::String {
                options: vec![
                    CreateSelectMenuOption::new("Team Info", "team_page").emoji(ReactionType::Unicode("🗿".to_string())).description("General information about the team"),
                    CreateSelectMenuOption::new("Awards", "awards_page").emoji(ReactionType::Unicode("🏆".to_string())).description("Awards from events throughout the season"),
                    CreateSelectMenuOption::new("Stats", "stats_page").emoji(ReactionType::Unicode("📊".to_string())).description("Team statistics & rankings"),
                    CreateSelectMenuOption::new("Events", "events_page").emoji(ReactionType::Unicode("🗓️".to_string())).description("Event attendance from this team"),
                ],
            },
        ))
    ];
    components.push(CreateActionRow::SelectMenu(CreateSelectMenu::new(
        "team_season_select",
        CreateSelectMenuKind::String {
            options: seasons.iter().enumerate().map(|(i, season)| {
                CreateSelectMenuOption::new(
                    &season.name,
                    format!("option_season_{}", season.id),
                ).default_selection(i == 0).description(format!("{}-{}", season.years_start, season.years_end))
            }).collect(),
        },
    )));
    components
}

pub async fn create_general_embed(team: Team, robotevents: &RobotEvents,
                                  vrc_data_analysis: &VRCDataAnalysis, components: bool) -> (Option<CreateEmbed>, Option<Vec<CreateActionRow>>, Option<String>) {
    let mut message_components: Option<Vec<CreateActionRow>> = if let Ok(seasons) = robotevents.team_active_seasons(&team).await && components {
        Some(create_interactions(seasons).await)
    } else if components {
        Some(create_interactions(Vec::new()).await)
    } else {
        None::<Vec<CreateActionRow>>
    };
    let trueskill = if team.program.id != 1 {
        "Not supported for program".to_string()
    } else {
        match vrc_data_analysis.team_info(team.number.as_str()).await {
            Ok(team) => team.trueskill_ranking.to_string(),
            Err(_) => "No trueskill".to_string()
        }
    };
    let mut embed = CreateEmbed::new().title(format!(
        "{} ({} {})",
        team.number, team.program.code, team.grade
    )).url(format!(
        "https://www.robotevents.com/teams/{}/{}",
        team.program.code, team.number
    )).description(team.team_name).field("Organization", team.organization, true).field(
        "Location",
        format!(
            "{}, {}, {}",
            team.location.city, team.location.region, team.location.country
        ),
        true,
    ).field("TrueSkill Ranking", trueskill, true).field(
        "Registered",
        if team.registered { "Yes" } else { "No" },
        true,
    ).color(match team.program.code.as_ref() {
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

    (Some(embed), message_components, None)
}

pub async fn create_awards_embed(team: Team, robotevents: &RobotEvents,
                                 components: bool) -> (Option<CreateEmbed>, Option<Vec<CreateActionRow>>, Option<String>) {
    let mut message_components: Option<Vec<CreateActionRow>> = if let Ok(seasons) = robotevents.team_active_seasons(&team).await && components {
        Some(create_interactions(seasons).await)
    } else if components {
        Some(create_interactions(Vec::new()).await)
    } else {
        None::<Vec<CreateActionRow>>
    };

    let mut embed = CreateEmbed::new().title(format!(
        "{} ({} {}) Awards",
        team.number, team.program.code, team.grade
    )).url(format!(
        "https://www.robotevents.com/teams/{}/{}",
        team.program.code, team.number
    )).color(match team.program.code.as_ref() {
        "VRC" | "VEXU" | "TSA VRC" => Color::from_rgb(210, 38, 48),
        "VIQRC" | "TSA VIQRC" => Color::from_rgb(0, 119, 200),
        "VAIRC" => Color::from_rgb(00, 255, 00),
        _ => Default::default(),
    });

    (Some(embed), message_components, None)
}

pub async fn create_stats_embed(team: Team, robotevents: &RobotEvents,
                                components: bool) -> (Option<CreateEmbed>, Option<Vec<CreateActionRow>>, Option<String>) {
    let team = team.clone();
    let mut message_components: Option<Vec<CreateActionRow>> = if let Ok(seasons) = robotevents.team_active_seasons(&team).await && components {
        Some(create_interactions(seasons).await)
    } else if components {
        Some(create_interactions(Vec::new()).await)
    } else {
        None::<Vec<CreateActionRow>>
    };

    let mut embed = CreateEmbed::new().title(format!(
        "{} ({} {}) Stats",
        team.number, team.program.code, team.grade
    )).url(format!(
        "https://www.robotevents.com/teams/{}/{}",
        team.program.code, team.number
    )).color(match team.program.code.as_ref() {
        "VRC" | "VEXU" | "TSA VRC" => Color::from_rgb(210, 38, 48),
        "VIQRC" | "TSA VIQRC" => Color::from_rgb(0, 119, 200),
        "VAIRC" => Color::from_rgb(00, 255, 00),
        _ => Default::default(),
    });

    (Some(embed), message_components, None)
}

pub async fn create_events_embed(team: Team, robotevents: &RobotEvents, components: bool) -> (Option<CreateEmbed>, Option<Vec<CreateActionRow>>, Option<String>) {
    let team = team.clone();
    let mut message_components: Option<Vec<CreateActionRow>> = if let Ok(seasons) = robotevents.team_active_seasons(&team).await && components {
        Some(create_interactions(seasons).await)
    } else if components {
        Some(create_interactions(Vec::new()).await)
    } else {
        None::<Vec<CreateActionRow>>
    };

    let mut embed = CreateEmbed::new().title(format!(
        "{} ({} {}) Events",
        team.number, team.program.code, team.grade
    )).url(format!(
        "https://www.robotevents.com/teams/{}/{}",
        team.program.code, team.number
    )).color(match team.program.code.as_ref() {
        "VRC" | "VEXU" | "TSA VRC" => Color::from_rgb(210, 38, 48),
        "VIQRC" | "TSA VIQRC" => Color::from_rgb(0, 119, 200),
        "VAIRC" => Color::from_rgb(00, 255, 00),
        _ => Default::default(),
    });

    (Some(embed), message_components, None)
}