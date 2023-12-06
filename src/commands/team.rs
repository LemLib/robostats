use serenity::all::{CommandDataOptionValue, CommandOptionType, ReactionType, ActionRowComponent};
use serenity::builder::{
    CreateActionRow, CreateCommand, CreateCommandOption, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption,
};
use serenity::client::Context;
use serenity::model::application::CommandInteraction;
use serenity::model::Color;

use crate::api::robotevents::client::RobotEvents;

pub async fn response(
    _ctx: &Context,
    interaction: &CommandInteraction,
    robotevents: &RobotEvents,
) -> CreateInteractionResponse {
    let team_number =
        if let CommandDataOptionValue::String(number) = &interaction.data.options[0].value {
            number
        } else {
            return CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content("Invalid team number."),
            );
        };

    if let Ok(teams) = robotevents.find_teams(team_number).await {
        if let Some(team) = teams.iter().next() {
            let team = team.clone();

            let mut message_components: Vec<CreateActionRow> = vec![
                CreateActionRow::SelectMenu(CreateSelectMenu::new(
                    "team_page_select",
                    CreateSelectMenuKind::String {
                        options: vec![
                            CreateSelectMenuOption::new("Team Info", "team")
                                .emoji(ReactionType::Unicode("🗿".to_string()))
                                .description("General information about the team")
                                .default_selection(true),
                            CreateSelectMenuOption::new("Awards", "awards")
                                .emoji(ReactionType::Unicode("🏆".to_string()))
                                .description("Awards from events throughout the season"),
                            CreateSelectMenuOption::new("Stats", "stats")
                                .emoji(ReactionType::Unicode("📊".to_string()))
                                .description("Team statistics & rankings"),
                            CreateSelectMenuOption::new("Events", "events")
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

            let embed = CreateEmbed::new()
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
                .field(
                    "Registered",
                    if team.registered { "Yes" } else { "No" },
                    true,
                )
                .color(match team.program.code.as_ref() {
                    "VRC" | "VEXU" => Color::from_rgb(210, 38, 48),
                    "VIQRC" => Color::from_rgb(0, 119, 200),
                    "VAIRC" => Color::from_rgb(91, 91, 91),
                    _ => Default::default(),
                });

            let embed = if let Some(robot_name) = team.robot_name {
                if !robot_name.is_empty() {
                    embed.field("Robot Name", robot_name, true)
                } else {
                    embed
                }
            } else {
                embed
            };

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
}
