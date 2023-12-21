use std::str::FromStr;

use robotevents::filters::TeamEventsFilter;
use robotevents::schema::Event;
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

use robotevents::{
    RobotEvents,
    filters::{TeamsFilter, SeasonsFilter, TeamAwardsFilter},
    schema::{PaginatedResponse, Team, Season, Award, IdInfo}
};
use crate::api::vrc_data_analysis::{
    VRCDataAnalysis,
    schema::TeamInfo
};

/// Represents a possible embed sent by the `/team`` command.
/// 
/// - The Overview embed displays general information about a team.
/// - The Awards embed displays information about a team's RobotEvents awards.
/// - The Stats page displays team statistics and rankings.
/// - The Events page displays a list of events that a team attended.
///  
/// > Different embed "pages" may require different, separately-fetched pieces of data.
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum EmbedPage {
    #[default]
    Overview,
    Awards,
    Stats,
    Events,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct ParseEmbedPageError;

impl FromStr for EmbedPage {
    type Err = ParseEmbedPageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "option_team_overview" => Ok(Self::Overview),
            "option_team_awards" => Ok(Self::Awards),
            "option_team_stats" => Ok(Self::Stats),
            "option_team_events" => Ok(Self::Events),
            _ => Err(ParseEmbedPageError),
        }
    }
}

impl EmbedPage {
    /// Converts a variant of [`Self`] to a string matching the select option IDs used by the
    /// bot's messages.
    pub fn to_option_id(&self) -> &str {
        match self {
            Self::Overview => "option_team_overview",
            Self::Awards => "option_team_awards",
            Self::Stats => "option_team_stats",
            Self::Events => "option_team_events",
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
    current_page: EmbedPage,

    /// Current user-selected season ID.
    current_season: Option<i32>,
    
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
    awards: Option<Vec<Award>>,

    /// List of events the team has attended.
    events: Option<Vec<Event>>,
}

impl TeamCommand {
    /// Get a [`serenity::builder::CreateCommand`] instance associated with this command.
    /// 
    /// Contains metadata for the slash command that users will interact with through autocomplete.
    pub fn command(program_list: Option<PaginatedResponse<IdInfo>>) -> CreateCommand {
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
            
            for program in program_list.data.iter() {
                option = option.add_int_choice(&program.code.clone().unwrap_or("Unknown".to_string()), program.id);
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
        page_selection: EmbedPage,
        season_selection_id: i32,
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
            // Also: Season selection does nothing on the overview page, since RobotEvents only returns the
            // latest info about a team, so there's no point in showing it there either.
            let is_overview_page = if let EmbedPage::Overview = page_selection { true } else { false };

            if !active_seasons.is_empty() && !is_overview_page {
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
                                .default_selection(season.id == season_selection_id)
                                .description(format!("{}-{}", season.years_start, season.years_end))
                            })
                            .collect(),
                    },
                )))
            }
        }

        components
    }

    /// Generates an embed based on a page variant and provided data.
    /// 
    /// Returned as an instance of [`serenity::builder::CreateEmbed`].
    pub async fn embed(&mut self, page: EmbedPage, robotevents: &RobotEvents) -> CreateEmbed {
        let team = if let Ok(team) = self.find_robotevents_team(&robotevents).await {
            team
        } else {
            return CreateEmbed::new()
                .title("Failed to fetch RobotEvents team data.");
        };

        let mut embed = CreateEmbed::new()
            // TODO: More collors for different RobotEvents programs.
            .color(match team.program.code.clone().unwrap_or("VRC".to_string()).as_ref() {
                "VRC" | "VEXU" => Color::from_rgb(210, 38, 48),
                "VIQRC" => Color::from_rgb(0, 119, 200),
                "VAIRC" => Color::from_rgb(91, 91, 91),
                _ => Default::default(),
            });

        match page {
            EmbedPage::Overview => {
                embed = embed
                    .title(format!(
                        "{} ({}, {})",
                        team.number, team.program.code.clone().unwrap_or("VRC".to_string()), team.grade
                    ))
                    .url(format!(
                        "https://www.robotevents.com/teams/{}/{}",
                        team.program.code.clone().unwrap_or("VRC".to_string()), team.number
                    ))
                    .description(&team.team_name)
                    .field("Organization", &team.organization, false)
                    .field(
                        "Location",
                        format!(
                            "{}, {}, {}",
                            team.location.city, team.location.region, team.location.country
                        ),
                        false,
                    )
                    // TODO: Should we hide this field entirely if no robot name is available?
                    .field(
                        "Robot Name",
                        if let Some(robot_name) = &team.robot_name {
                            robot_name
                        } else {
                            "Unnamed"
                        },
                        false,
                    )
                    .field(
                        "Registered",
                        if team.registered { "Yes" } else { "No" },
                        false,
                    );
            },
            EmbedPage::Awards => {
                let awards = if let Ok(awards) = self.find_team_awards(robotevents).await {
                    awards
                } else {
                    return CreateEmbed::new()
                        .title("Failed to fetch RobotEvents awards data.");
                };

                embed = embed
                    .title(format!(
                        "{} ({}, {}) Awards",
                        team.number, team.program.code.clone().unwrap_or("VRC".to_string()), team.grade
                    ))
                    .description(format!("Total Awards: {}", awards.len()));

                for award in awards {
                    embed = embed.field(award.event.name, award.title, true);
                }
            },
            EmbedPage::Events => {
                let events = if let Ok(events) = self.find_team_events(robotevents).await {
                    events
                } else {
                    return CreateEmbed::new()
                        .title("Failed to fetch RobotEvents events data.");
                };

                embed = embed
                    .title(format!(
                        "{} ({}, {}) Events",
                        team.number, team.program.code.clone().unwrap_or("VRC".to_string()), team.grade
                    ))
                    .description(format!("Total Events: {}", events.len()));

                for event in events {
                    embed = embed.field(event.name, format!("[View More](https://robotevents.com/{})", event.sku), true);
                }
            },
            _ => {
                embed = embed
                    .title("Unknown Page")
                    .description("How did you even get here? 🤔");
            }
        }

        embed
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
            self.active_seasons = if let Ok(seasons) = robotevents.seasons(SeasonsFilter::new().team(team.id)).await {
                self.current_season = Some(seasons.data[0].id);
                Some(seasons.data)
            } else {
                return CreateInteractionResponseMessage::new()
                    .content("Failed to get season information about team from RobotEvents.");
            };

            // This check is technically redundant, since we just set `self.current_page` to Some(T), but
            // it keeps us from have to unnecessarily clone using other methods, so... ¯\_(ツ)_/¯
            CreateInteractionResponseMessage::new()
                .embed(self.embed(self.current_page, &robotevents).await)
                .components(self.components(self.current_page, self.current_season.unwrap()))
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
        // Ensure that a team number has been provided by a user.
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
            let mut filter = TeamsFilter::new().number(team_number.to_string());
            if let Some(program_id_filter) = self.program_id_filter {
                filter = filter.program(program_id_filter);
            }

            if let Ok(teams) = robotevents.teams(filter).await {
                if let Some(team) = teams.data.iter().next() {
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
    
    pub async fn find_team_awards(
        &mut self,
        robotevents: &RobotEvents,
    ) -> Result<Vec<Award>, TeamCommandRequestError> {
        if let Some(team) = &self.team {
            if let Some(awards) = self.awards.clone() {
                Ok(awards)
            } else {
                // Fetch awards using RobotEvents HTTP client
                if let Ok(fetched_awards) = team.awards(robotevents, TeamAwardsFilter::new().season(self.current_season.unwrap())).await {
                    self.awards = Some(fetched_awards.clone().data);
                    Ok(fetched_awards.data)
                } else {
                    Err(TeamCommandRequestError)
                }
            }
        } else {
            Err(TeamCommandRequestError)
        }
    }

    pub async fn find_team_events(
        &mut self,
        robotevents: &RobotEvents,
    ) -> Result<Vec<Event>, TeamCommandRequestError> {
        if let Some(team) = &self.team {
            if let Some(events) = self.events.clone() {
                Ok(events)
            } else {
                // Fetch events using RobotEvents HTTP client
                if let Ok(fetched_events) = team.events(robotevents, TeamEventsFilter::new().season(self.current_season.unwrap())).await {
                    self.events = Some(fetched_events.clone().data);
                    Ok(fetched_events.data)
                } else {
                    Err(TeamCommandRequestError)
                }
            }
        } else {
            Err(TeamCommandRequestError)
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
            let changed_value: &str = values.first().unwrap().as_ref();

            let message_edit = if changed_value.starts_with("option_team_") { // User changed page
                self.current_page = changed_value.parse::<EmbedPage>().unwrap();

                command_interaction
                    .edit_response(
                        &ctx,
                        EditInteractionResponse::new()
                            .embed(self.embed(self.current_page, &robotevents).await)
                            .components(self.components(
                                self.current_page,
                                self.current_season.unwrap(),
                            )),
                    )
                    .await
            } else if changed_value.starts_with("option_season_") { // User changed season
                let season_id = if let Ok(parsed_id) = changed_value.trim_start_matches("option_season_").parse::<i32>() {
                    parsed_id
                } else {
                    return CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(format!("Failed to parse season ID for {}.", changed_value)),
                    )
                };

                self.current_season = Some(season_id);
                self.awards = None; // Reset awards cache since the season has changed.
                self.events = None; // Reset awards cache since the season has changed.

                command_interaction
                    .edit_response(
                        &ctx,
                        EditInteractionResponse::new()
                            .embed(self.embed(self.current_page, &robotevents).await)
                            .components(self.components(
                                self.current_page,
                                season_id,
                            )),
                    )
                    .await
            } else {
                return CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("Unhandled component interaction. This shouldn't happen."),
                )
            };

            if message_edit.is_ok() {
                CreateInteractionResponse::Acknowledge
            } else {
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content("Failed to edit embed."),
                )
            }
        } else {
            return CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Unhandled component interaction. This shouldn't happen."),
            )
        }
    }
}
