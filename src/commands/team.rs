use serenity::all::{
    CommandDataOptionValue, CommandOptionType, ComponentInteraction,
    ComponentInteractionDataKind, ReactionType,
};
use serenity::builder::{
    CreateActionRow, CreateCommand, CreateCommandOption, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption, EditInteractionResponse,
};
use serenity::client::Context;
use serenity::model::application::CommandInteraction;
use serenity::model::Color;

use crate::api::robotevents::client::RobotEvents;
use crate::api::vrc_data_analysis::client::VRCDataAnalysis;

use crate::api::robotevents::schema::{Season, Team};
use crate::api::vrc_data_analysis::schema::TeamInfo;

#[derive(Clone, Debug, PartialEq)]
pub enum EmbedPage {
    Overview(Team),
    Awards(Team),
    Stats(Team, TeamInfo),
    Events(Team),
}

impl EmbedPage {
    pub fn to_option_id(&self) -> &str {
        match self {
            Self::Overview(_) => "option_team_overview",
            Self::Awards(_) => "option_team_awards",
            Self::Stats(_, _) => "option_team_stats",
            Self::Events(_) => "option_team_events",
        }
    }

    pub fn embed(&self) -> CreateEmbed {
        match self {
            Self::Overview(team) => CreateEmbed::new()
                .title(format!(
                    "{} ({} {})",
                    team.number, team.program.code, team.grade
                ))
                .url(format!(
                    "https://www.robotevents.com/teams/{}/{}",
                    team.program.code, team.number
                ))
                .description(&team.team_name)
                .field("Organization", &team.organization, true)
                .field(
                    "Location",
                    format!(
                        "{}, {}, {}",
                        team.location.city, team.location.region, team.location.country
                    ),
                    true,
                )
                .field(
                    "Robot Name",
                    if let Some(robot_name) = &team.robot_name {
                        robot_name
                    } else {
                        "Unnamed"
                    },
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
                }),
            Self::Awards(_) => CreateEmbed::new().title("awards wow"),
            _ => CreateEmbed::new().title("fallback"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TeamCommandRequestError;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct TeamCommand {
    current_page: Option<EmbedPage>,
    current_season: Option<Season>,
    team_number: Option<String>,
    team: Option<Team>,
    seasons: Option<Vec<Season>>,
    active_seasons: Option<Vec<Season>>,
    program_id_filter: Option<i32>,
    data_analysis: Option<TeamInfo>,
    awards: Option<()>,
    events: Option<()>,
}

impl TeamCommand {
    pub fn command() -> CreateCommand {
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
                    //VRC is 1, VEXU is 4, and VEXIQ is 41
                    .add_int_choice("VRC", 1)
                    .add_int_choice("VEXU", 4)
                    .add_int_choice("VEXIQ", 41),
            )
    }

    pub fn components(
        &self,
        page_selection: &EmbedPage,
        season_selection: &Season,
    ) -> Vec<CreateActionRow> {
        let page_selection_id = page_selection.to_option_id();

        let mut components = vec![CreateActionRow::SelectMenu(CreateSelectMenu::new(
            "team_page_select",
            CreateSelectMenuKind::String {
                options: vec![
                    CreateSelectMenuOption::new("Team Overview", "option_team_overview")
                        .emoji(ReactionType::Unicode("🗿".to_string()))
                        .description("General information about the team")
                        .default_selection(page_selection_id == "option_team_overview"),
                    CreateSelectMenuOption::new("Awards", "option_team_awards")
                        .emoji(ReactionType::Unicode("🏆".to_string()))
                        .description("Awards from events throughout the season")
                        .default_selection(page_selection_id == "option_team_awards"),
                    CreateSelectMenuOption::new("Stats", "option_team_stats")
                        .emoji(ReactionType::Unicode("📊".to_string()))
                        .description("Team statistics & rankings")
                        .default_selection(page_selection_id == "option_team_stats"),
                    CreateSelectMenuOption::new("Events", "option_team_events")
                        .emoji(ReactionType::Unicode("🗓️".to_string()))
                        .description("Event attendance from this team")
                        .default_selection(page_selection_id == "option_team_events"),
                ],
            },
        ))];

        if let Some(active_seasons) = &self.active_seasons {
            if !active_seasons.is_empty() {
                components.push(CreateActionRow::SelectMenu(CreateSelectMenu::new(
                    "team_season_select",
                    CreateSelectMenuKind::String {
                        options: active_seasons
                            .iter()
                            .map(|season| {
                                CreateSelectMenuOption::new(
                                    &season.name,
                                    format!("option_season_{}", season.id),
                                )
                                .default_selection(season == season_selection)
                                .description(format!("{}-{}", season.years_start, season.years_end))
                            })
                            .collect(),
                    },
                )))
            }
        }

        components
    }

    pub async fn response(
        &mut self,
        _ctx: &Context,
        interaction: &CommandInteraction,
        robotevents: &RobotEvents,
    ) -> CreateInteractionResponseMessage {
        self.team_number =
            if let CommandDataOptionValue::String(number) = &interaction.data.options[0].value {
                Some(number.to_string())
            } else {
                return CreateInteractionResponseMessage::new().content("Invalid team number.");
            };

        self.program_id_filter = if interaction.data.options.len() < 2 {
            None
        } else if let CommandDataOptionValue::Integer(id) = &interaction.data.options[1].value {
            i32::try_from(*id).ok()
        } else {
            return CreateInteractionResponseMessage::new()
                .content("Invalid RobotEvents program value.");
        };

        if let Ok(team) = self.find_robotevents_team(&robotevents).await {
            self.active_seasons = if let Ok(seasons) = robotevents.team_active_seasons(&team).await {
                Some(seasons)
            } else {
                return CreateInteractionResponseMessage::new()
                    .content("Failed to get season information about team from RobotEvents.");
            };

            self.current_page = Some(EmbedPage::Overview(team));

            if let Some(page) = &self.current_page {
                CreateInteractionResponseMessage::new()
                    .embed(page.embed())
                    .components(self.components(page, &self.active_seasons.clone().unwrap()[0]))
            } else {
                return CreateInteractionResponseMessage::new().content("Invalid embed page.");
            }
        } else {
            CreateInteractionResponseMessage::new().content("Failed to connect to RobotEvents.")
        }
    }

    /// Returns the RobotEvents team data associated with this instance of [`Self`].
    /// If `self.team` happens to be `None`, this function will attempt to fetch the required information
    /// from the RobotEvents API, but otherwise return the cached result.
    pub async fn find_robotevents_team(
        &mut self,
        robotevents: &RobotEvents,
    ) -> Result<Team, TeamCommandRequestError> {
        let team_number = if let Some(team_number) = &self.team_number {
            team_number
        } else {
            return Err(TeamCommandRequestError);
        };

        if let Some(team) = self.team.clone() {
            Ok(team)
        } else {
            if let Ok(teams) = robotevents.find_teams(team_number, self.program_id_filter).await {
                if let Some(team) = teams.iter().next().to_owned() {
                    self.team = Some(team.clone());
                    Ok(team.clone())
                } else {
                    Err(TeamCommandRequestError)
                }
            } else {
                Err(TeamCommandRequestError)
            }
        }
    }

    pub async fn component_interaction_response(
        &mut self,
        ctx: &Context,
        command_interaction: &CommandInteraction,
        component_interaction: &ComponentInteraction,
        robotevents: &RobotEvents,
    ) -> CreateInteractionResponse {
        if let ComponentInteractionDataKind::StringSelect { values } = &component_interaction.data.kind {
            let team = if let Ok(team) = self.find_robotevents_team(robotevents).await {
                team
            } else {
                return CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("Failed fetch RobotEvents team data."),
                );
            };

            let message_edit = match values.first().unwrap().as_ref() {
                "option_team_overview" => {
                    let page = EmbedPage::Overview(team);
                    command_interaction
                        .edit_response(
                            &ctx,
                            EditInteractionResponse::new()
                                .embed(page.embed())
                                .components(self.components(
                                    &page,
                                    &self.active_seasons.clone().unwrap()[0],
                                )),
                        )
                        .await
                }
                "option_team_awards" => {
                    let page = EmbedPage::Awards(team);
                    command_interaction
                        .edit_response(
                            &ctx,
                            EditInteractionResponse::new()
                                .embed(page.embed())
                                .components(self.components(
                                    &page,
                                    &self.active_seasons.clone().unwrap()[0],
                                )),
                        )
                        .await
                }
                // "option_team_stats" => {
                // },
                // "option_team_events" => {
                // },
                _ => {
                    return CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("Unknown interaction component. This shouldn't happen."),
                    )
                }
            };

            if message_edit.is_ok() {
                CreateInteractionResponse::Acknowledge
            } else {
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content("Failed to edit embed."),
                )
            }
        } else {
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content("Failed to edit embed."),
            )
        }
    }
}
