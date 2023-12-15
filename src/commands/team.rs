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

use crate::BotRequestError;
use crate::api::robotevents::client::RobotEvents;
use crate::api::vrc_data_analysis::client::VRCDataAnalysis;

use crate::api::robotevents::schema::{Season, IdInfo, Team};
use crate::api::vrc_data_analysis::schema::TeamInfo;

/// Represents a possible embed sent by the `/team`` command.
/// 
/// - The Overview embed displays general information about a team.
/// - The Awards embed displays information about a team's RobotEvents awards.
/// - The Stats page displays team statistics and rankings.
/// - The Events page displays a list of events that a team attended.
///  
/// > Different embed "pages" may require different, separately-fetched pieces of data.
#[derive(Clone, Debug, PartialEq)]
pub enum EmbedPage {
    Overview(Team),
    Awards(Team),
    Stats(Team, TeamInfo),
    Events(Team),
}

impl EmbedPage {
    /// Converts a variant of [`Self`] to a string matching the select option IDs used by the
    /// bot's messages.
    pub fn to_option_id(&self) -> &str {
        match self {
            Self::Overview(_) => "option_team_overview",
            Self::Awards(_) => "option_team_awards",
            Self::Stats(_, _) => "option_team_stats",
            Self::Events(_) => "option_team_events",
        }
    }

    /// Generates an embed based on a page variant and provided data.
    /// 
    /// Returned as an instance of [`serenity::builder::CreateEmbed`].
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
                // TODO: Should we hide this field entirely if no robot name is available?
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
                // TODO: More collors for different RobotEvents programs.
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

/// Handler for the "/team" command.
/// The team command serves the purpose of displaying information and statistics about a singular
/// RobotEvents team.
/// 
/// Data stored in this struct is heavily wrapped in Option<T> due to being lazily fetched via HTT
/// and cached.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct TeamCommand {
    /// Current user-selected [`EmbedPage`].
    current_page: Option<EmbedPage>,

    /// Current user-selected season.
    current_season: Option<Season>,
    
    /// Team number string requested by the user on the initial command interaction.
    team_number: Option<String>,

    /// Team fetched from robotevents.
    /// > This will be `None` if the request fails or hasn't been made yet.
    team: Option<Team>,

    /// List of active seasons the team has competed in.
    /// > This will be `None` if the request fails or hasn't been made yet.
    active_seasons: Option<Vec<Season>>,

    /// The "program" option selected by the user when making the initial command.
    program_id_filter: Option<i32>,

    /// Data analysis info fetched about A VRC team.
    /// This is `None` if the data hasn't been fetched yet or the team is in a program other than VRC.
    data_analysis: Option<TeamInfo>,

    /// List of awards the team has recieved.
    awards: Option<()>,

    /// List of events the team has attended.
    events: Option<()>,
}

impl TeamCommand {
    /// Get a [`serenity::builder::CreateCommand`] instance associated with this command.
    /// 
    /// Contains metadata for the slash command that users will interact with through autocomplete.
    pub fn command(program_list: Option<Vec<IdInfo>>) -> CreateCommand {
        let mut command = CreateCommand::new("team")
            .description("Displays information about a team")
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "team", "Team Number")
                    .required(true),
            );
    
        // Generate a list of RobotEvents programs for the user to filter teams from.
        // This list was fetched on bot startup, and could possibly fail. In the event
        // that it did fail, we'll simply not present this choice to the user.
        if let Some(program_list) = program_list {
            let mut option = CreateCommandOption::new(CommandOptionType::Integer, "program", "Program Name").required(false);
            
            for program in program_list.iter() {
                option = option.add_int_choice(&program.code, program.id);
            }
            
            command = command.add_option(option);
        }

        command
    }

    /// Get a list of message components associated with this command. Includes select menus,
    /// radio options, buttons, etc...
    /// 
    /// This isn't hardcoded into [`Self::response`] due to Discord unfortunately wiping the
    /// user's selection choice when a message is edited. As a result, we need to reconstruct
    /// the select menu with a new default selection each time the bot edits the embed.
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

        // In the event that the team has no active seasons, we won't render the season menu at all.
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
                                // NOTE: We could compare seasons by PartialEq I guess, although it might be
                                //       better to stick to IDs for now.
                                .default_selection(season.id == season_selection.id)
                                .description(format!("{}-{}", season.years_start, season.years_end))
                            })
                            .collect(),
                    },
                )))
            }
        }

        components
    }

    /// Generate an initial response message to a command interaction.
    /// 
    /// When the bot recieves a [`serenity::model::application::CommandInteraction`] as a result of a user
    /// sending a slash command, this message will be constructed and sent in response.
    /// 
    /// The `/team` command has two primary user-provided arguments:
    /// - A team number, which is eventually searched for on RobotEvents.
    /// - An optional team program filter, which specifies which program the bot should search for the
    ///   team in (e.g. VRC, VIQC, VAIC...).
    /// 
    /// > By default, this response will start on the team overview [`EmbedPage`].
    pub async fn response(
        &mut self,
        _ctx: &Context,
        interaction: &CommandInteraction,
        robotevents: &RobotEvents,
    ) -> CreateInteractionResponseMessage {
        // Set the initially requested team number from command arguments.
        self.team_number =
            if let CommandDataOptionValue::String(number) = &interaction.data.options[0].value {
                Some(number.to_string())
            } else {
                return CreateInteractionResponseMessage::new().content("Invalid team number.");
            };

        // Set program filter if used.
        self.program_id_filter = if interaction.data.options.len() < 2 {
            None
        } else if let CommandDataOptionValue::Integer(id) = &interaction.data.options[1].value {
            i32::try_from(*id).ok() // This conversion from i64 to i32 shouldn't ever realistically fail...
        } else {
            return CreateInteractionResponseMessage::new()
                .content("Invalid RobotEvents program value.");
        };

        // Fetch RobotEvents team data over HTTP.
        if let Ok(team) = self.find_robotevents_team(&robotevents).await {

            // Find a list of seasons that the fetched team was active in using a separate endpoint.
            self.active_seasons = if let Ok(seasons) = robotevents.team_active_seasons(&team).await {
                Some(seasons)
            } else {
                return CreateInteractionResponseMessage::new()
                    .content("Failed to get season information about team from RobotEvents.");
            };

            // We start initially on the overview embed until the user selects another page.
            self.current_page = Some(EmbedPage::Overview(team));

            // This check is technically redundant, since we just set `self.current_page` to Some(T), but
            // it keeps us from have to unnecessarily clone using other methods, so... ¯\_(ツ)_/¯
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
    /// 
    /// If `self.team` happens to be `None`, this function will attempt to fetch the required information
    /// from the RobotEvents API, but otherwise return the cached result.
    pub async fn find_robotevents_team(
        &mut self,
        robotevents: &RobotEvents,
    ) -> Result<Team, TeamCommandRequestError> {
        // Ensure tha a team number has been provided by a user.
        let team_number = if let Some(team_number) = &self.team_number {
            team_number
        } else {
            // This can only be `None` if the command has not been responded to at all, or the bot somehow
            // updates an invalid response which shouldn't realistically happen.
            return Err(TeamCommandRequestError);
        };

        // If we've already fetched the team data once (such as in the case of page changes with message edits),
        // use the data already stored in [`Self`].
        if let Some(team) = self.team.clone() {
            Ok(team)
        } else {
            // Fetch team using RobotEvents HTTP client
            if let Ok(teams) = robotevents.find_teams(team_number, self.program_id_filter).await {
                if let Some(team) = teams.iter().next() {
                    self.team = Some(team.clone()); // Cache value for later use. 
                    Ok(team.clone())
                } else {
                    Err(TeamCommandRequestError)
                }
            } else {
                Err(TeamCommandRequestError)
            }
        }
    }

    /// Responds to an interaction with the `/team` command's message components (e.g. select menus).
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
