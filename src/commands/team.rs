use std::collections::HashMap;
use std::str::FromStr;

use robotevents::query::PaginatedQuery;
use serenity::all::{
    CommandDataOptionValue, CommandOptionType, ComponentInteraction,
    ComponentInteractionDataKind, ReactionType,
};
use serenity::builder::{
    CreateActionRow, CreateCommand, CreateCommandOption, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption, EditInteractionResponse, CreateEmbedFooter,
};
use serenity::client::Context;
use serenity::model::{application::CommandInteraction, Color};

use robotevents::{
    RobotEvents,
    query::{TeamsQuery, SeasonsQuery, TeamAwardsQuery, TeamEventsQuery},
    schema::{PaginatedResponse, Team, Event, Season, Award, IdInfo}
};
use crate::api::skills::{SkillsCache, SkillsRanking};
use crate::api::vrc_data_analysis::{
    VRCDataAnalysis,
    schema::TeamInfo
};

const MAX_PER_PAGE: i32 = 250;

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
            "overview" => Ok(Self::Overview),
            "stats" => Ok(Self::Stats),
            "awards" => Ok(Self::Awards),
            "events" => Ok(Self::Events),
            _ => Err(ParseEmbedPageError),
        }
    }
}

impl std::fmt::Display for EmbedPage {
     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::Overview => "overview",
            Self::Stats => "stats",
            Self::Awards => "awards",
            Self::Events => "events",
        })
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
    awards: Option<PaginatedResponse<Award>>,

    /// List of events the team has attended.
    events: Option<PaginatedResponse<Event>>,

    skills_ranking: Option<Option<SkillsRanking>>,
}

impl TeamCommand {
    /// Get a [`serenity::builder::CreateCommand`] instance associated with this command.
    /// 
    /// Contains metadata for the slash command that users will interact with through autocomplete.
    pub fn command(program_list: Option<PaginatedResponse<IdInfo>>) -> CreateCommand {
        let team_opt = CreateCommandOption::new(CommandOptionType::String, "number", "Team Number").required(true);
        let mut program_opt = CreateCommandOption::new(CommandOptionType::Integer, "program", "Program Name").required(false);
        if let Some(program_list) = program_list {
            for program in program_list.data.iter() {
                program_opt = program_opt.add_int_choice(&program.name, program.id);
            }
        }

        CreateCommand::new("team")
            .description("Displays information about a team")
            .set_options(vec![
                CreateCommandOption::new(CommandOptionType::SubCommand, "overview", "General information about a team")
                    .add_sub_option(team_opt.clone())
                    .add_sub_option(program_opt.clone()),
                CreateCommandOption::new(CommandOptionType::SubCommand, "stats", "Team statistics & rankings")
                    .add_sub_option(team_opt.clone())
                    .add_sub_option(program_opt.clone()),
                CreateCommandOption::new(CommandOptionType::SubCommand, "awards", "Awards a team has earned for a season")
                    .add_sub_option(team_opt.clone())
                    .add_sub_option(program_opt.clone()),
                CreateCommandOption::new(CommandOptionType::SubCommand, "events", "Event attendance from a team")
                    .add_sub_option(team_opt.clone())
                    .add_sub_option(program_opt.clone()),
            ])
    }

    /// Generate the message components associated with this command, including the page and season
    /// select menus.
    /// 
    /// This isn't hardcoded into [`Self::response`] due to Discord unfortunately wiping the
    /// user's selection choice when a message is edited. As a result, we need to reconstruct
    /// the select menu with a new default selection each time the bot edits the embed.
    pub fn components(
        &self,
        page_selection: EmbedPage,
        season_selection_id: i32,
    ) -> Vec<CreateActionRow> {
        let mut page_selection_id = page_selection.to_string();
        page_selection_id.insert_str(0, "option_team_");

        let mut components = vec![CreateActionRow::SelectMenu(CreateSelectMenu::new(
            "team_page_select",
            CreateSelectMenuKind::String {
                options: vec![
                    CreateSelectMenuOption::new("Team Overview", "option_team_overview")
                        .emoji(ReactionType::Unicode("🗿".to_string()))
                        .description("General information about the team")
                        .default_selection(page_selection_id == "option_team_overview"),
                    CreateSelectMenuOption::new("Stats", "option_team_stats")
                        .emoji(ReactionType::Unicode("📊".to_string()))
                        .description("Team statistics & rankings")
                        .default_selection(page_selection_id == "option_team_stats"),
                    CreateSelectMenuOption::new("Awards", "option_team_awards")
                        .emoji(ReactionType::Unicode("🏆".to_string()))
                        .description("Awards from events throughout the season")
                        .default_selection(page_selection_id == "option_team_awards"),
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

    /// Constructs an embed based on a page variant and provided data. This will be added to response
    /// messages based on the page that the user has selected along with the components provided by
    /// [`Self::components`].
    /// 
    /// Returned as an instance of [`serenity::builder::CreateEmbed`].
    pub async fn embed(&mut self, page: EmbedPage, robotevents: &RobotEvents, vrc_data_analysis: &VRCDataAnalysis, skills_cache: &SkillsCache) -> CreateEmbed {
        let team = match self.find_robotevents_team(&robotevents).await {
            Ok(team) => team,
            Err(err) => {
                return CreateEmbed::new()
                    .title("Failed to fetch RobotEvents team data.")
                    .description(format!("```rs\n{err:?}```"));
            },
        };

        let mut embed = CreateEmbed::new()
            .url(format!(
                "https://www.robotevents.com/teams/{}/{}",
                team.program.code.clone().unwrap_or("VRC".to_string()), team.number
            ))
            // TODO: More colors for different RobotEvents programs.
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
                    .description(&team.team_name)
                    .field("Organization", &team.organization, false)
                    .field(
                        "Location",
                        format!(
                            "{}, {}, {}",
                            team.location.city, team.location.region.unwrap_or("Unknown".to_string()), team.location.country
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
            EmbedPage::Stats => {
                let skills_ranking = if let Some(skills_ranking) = &self.skills_ranking {
                    Ok(skills_ranking.clone())
                } else {
                    if let Some(team) = &self.team {
                        skills_cache.get_team_ranking(team, self.current_season.unwrap(), robotevents).await
                    } else {
                        return CreateEmbed::new()
                            .title("Invalid team data.");
                    }
                };

                let data_analysis = if let Some(data_analysis) = &self.data_analysis {
                    Some(Ok(data_analysis.clone()))
                } else {
                    if let Some(team) = &self.team {
                        if team.program.code == Some("VRC".to_string()) {
                            Some(vrc_data_analysis.team_info(&team.number).await)
                        } else {
                            None
                        }
                    } else {
                        return CreateEmbed::new()
                            .title("Invalid team data.");
                    }
                };

                embed = embed
                    .title(format!(
                        "{} ({}, {}) Statistics",
                        team.number, team.program.code.clone().unwrap_or("VRC".to_string()), team.grade
                    ));

                match skills_ranking {
                    Ok(ranking) => {
                        if let Some(ranking) = ranking {
                            embed = embed.field(
                                "Skills",
                                format!(
                                    "World Ranking: **#{}**\nDriver: **{}**\nProgramming: **{}**\nCombined: **{}**",
                                    ranking.rank,
                                    ranking.scores.max_driver,
                                    ranking.scores.max_programming,
                                    ranking.scores.score
                                ),
                                false
                            );
                        } else {
                            embed = embed.field("Skills", "No skills runs found for this team.", false);
                        }
                    },
                    Err(err) => {
                        embed = embed.field("Failed to get skills ranking data from RobotEvents.", format!("```rs\n{err:?}```"), false);
                    },
                }

                if let Some(analysis_result) = data_analysis {
                    match analysis_result {
                        Ok(analysis) => {
                            embed = embed.field(
                                    "TrueSkill",
                                    format!("TrueSkill: **{}**\nWorld Ranking **#{}**", analysis.trueskill, analysis.trueskill_ranking),
                                    false
                                )
                                .field(
                                    "Match Statistics",
                                    format!("OPR: **{}**\nDPR: **{}**\nCCWM: **{}**", analysis.opr, analysis.dpr, analysis.ccwm),
                                    false
                                )
                                .field(
                                    "Match Performance",
                                    format!(
                                        "Wins: **{}**\nLosses: **{}**\nTies: **{}**\nAverage WPs: **{}**\nAWP Percentage: **{:.2}%**",
                                        analysis.total_wins,
                                        analysis.total_losses,
                                        analysis.total_ties,
                                        analysis.wp_per_match,
                                        (analysis.awp_per_match * 100.0)
                                    ),
                                    false
                                )
                                .footer(
                                    CreateEmbedFooter::new("TrueSkill & Match Data provided by vrc-data-analysis.com")
                                        .icon_url("https://cdn.discordapp.com/attachments/1181718273017004043/1185320302272585758/favicon-3.png")
                                );
                        },
                        Err(err) => {
                            embed = embed.field("Failed to get team statistics from vrc-data-analysis.", format!("```rs\n{err:?}```"), false);
                        }
                    }
                } else {
                    embed = embed.description("*Note: TrueSkill and match analysis data is only available for VRC teams at the moment.*");
                }
            },
            EmbedPage::Awards => {
                let awards = if let Some(awards) = &self.awards {
                    awards.clone()
                } else {
                    if let Some(team) = &self.team {
                        match team.awards(robotevents, TeamAwardsQuery::new().season(self.current_season.unwrap()).per_page(MAX_PER_PAGE)).await {
                            Ok(awards) => {
                                self.awards = Some(awards.clone());
                                awards
                            },
                            Err(err) => {
                                return CreateEmbed::new()
                                    .title("Failed to fetch RobotEvents awards data.")
                                    .description(format!("```rs\n{err:?}```"));
                            },
                        }
                    } else {
                        return CreateEmbed::new()
                            .title("Invalid team data.");
                    }
                };

                let mut categorized_awards: HashMap<i32, Vec<Award>> = HashMap::new();
                for award in awards.data {
                    categorized_awards.entry(award.event.id).or_default().push(award);
                }

                embed = embed
                    .title(format!(
                        "{} ({}, {}) Awards",
                        team.number, team.program.code.clone().unwrap_or("VRC".to_string()), team.grade
                    ));

                    if let Some(to) = awards.meta.to {
                        embed = embed.footer(
                            CreateEmbedFooter::new(format!(
                                "Page {} of {} ({}/{} Results)",
                                awards.meta.current_page,
                                awards.meta.last_page,
                                to,
                                awards.meta.total
                            ))
                        );
                    } else {
                        embed = embed.description("No awards found.");
                    }

                for awards in categorized_awards.values() {
                    embed = embed.field(
                        &awards[0].event.name,
                        awards
                            .iter()
                            .map(|a| "- ".to_string() + &a.title)
                            .collect::<Vec<_>>()
                            .join("\n"),
                        true
                    );
                }
            },
            EmbedPage::Events => {
                let events = if let Some(events) = &self.events {
                    events.clone()
                } else {
                    if let Some(team) = &self.team {
                        match team.events(robotevents, TeamEventsQuery::new().season(self.current_season.unwrap()).per_page(MAX_PER_PAGE)).await {
                            Ok(events) => {
                                self.events = Some(events.clone());
                                events
                            },
                            Err(err) => {
                                return CreateEmbed::new()
                                    .title("Failed to fetch RobotEvents events data.")
                                    .description(format!("```rs\n{err:?}```"));
                            },
                        }
                        
                    } else {
                        return CreateEmbed::new()
                            .title("Invalid team data.");
                    }
                };

                embed = embed
                    .title(format!(
                        "{} ({}, {}) Events",
                        team.number, team.program.code.clone().unwrap_or("VRC".to_string()), team.grade
                    ));

                if let Some(to) = events.meta.to {
                    embed = embed.footer(
                        CreateEmbedFooter::new(format!(
                            "Page {} of {} ({}/{} Results)",
                            events.meta.current_page,
                            events.meta.last_page,
                            to,
                            events.meta.total
                        ))
                    );
                } else {
                    embed = embed.description("No events found.");
                }

                for event in events.data {
                    embed = embed.field(
                        event.name,
                        format!(
                            "[View More](https://robotevents.com/robot-competitions/{}/{})",
                            event.program.code.unwrap_or("VRC".to_string()),
                            event.sku
                        ),
                        true
                    );
                }
            },
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
        vrc_data_analysis: &VRCDataAnalysis,
        skills_cache: &SkillsCache,
    ) -> CreateInteractionResponseMessage {
        let options = if let CommandDataOptionValue::SubCommand(cmd) = &interaction.data.options[0].value {
            cmd
        }  else {
            return CreateInteractionResponseMessage::new().content("Invalid subcommand option.");
        };

        self.current_page = if let Ok(parsed_page) = interaction.data.options[0].name.parse::<EmbedPage>() {
            parsed_page
        } else {
            return CreateInteractionResponseMessage::new().content("Failed to parse subcommand type.");
        };

        // Set the initially requested team number from command arguments.
        self.team_number =
            if let CommandDataOptionValue::String(number) = &options[0].value {
                Some(number.to_string())
            } else {
                return CreateInteractionResponseMessage::new().content("Invalid team number.");
            };

        // Set program filter if used.
        self.program_id_filter = if options.len() < 2 {
            None
        } else if let CommandDataOptionValue::Integer(id) = &options[1].value {
            i32::try_from(*id).ok() // This conversion from i64 to i32 shouldn't ever realistically fail...
        } else {
            return CreateInteractionResponseMessage::new()
                .content("Invalid RobotEvents program value.");
        };

        // Fetch RobotEvents team data over HTTP.
        if let Ok(team) = self.find_robotevents_team(&robotevents).await {

            // Find a list of seasons that the fetched team was active in using a separate endpoint.
            self.active_seasons = match robotevents.seasons(SeasonsQuery::new().team(team.id).per_page(MAX_PER_PAGE)).await {
                Ok(seasons) => {
                    self.current_season = Some(seasons.data[0].id);
                    Some(seasons.data)
                },
                Err(err) => {
                    return CreateInteractionResponseMessage::new()
                        .add_embed(
                            CreateEmbed::new().title("Failed to get season information about team from RobotEvents.")
                            .description(format!("```rs\n{err:?}```")),
                        );
                }
            };

            // This check is technically redundant, since we just set `self.current_page` to Some(T), but
            // it keeps us from have to unnecessarily clone using other methods, so... ¯\_(ツ)_/¯
            CreateInteractionResponseMessage::new()
                .embed(self.embed(self.current_page, robotevents, vrc_data_analysis, skills_cache).await)
                .components(self.components(self.current_page, self.current_season.unwrap()))
        } else {
            CreateInteractionResponseMessage::new().content("Failed to find a RobotEvents team with this number.")
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
            return Ok(team);
        }

        // Fetch team using RobotEvents HTTP client
        let mut query = TeamsQuery::new().number(team_number.to_string());
        if let Some(program_id_filter) = self.program_id_filter {
            query = query.program(program_id_filter);
        }

        if let Ok(teams) = robotevents.teams(query).await {
            if let Some(team) = teams.data.iter().next() {
                self.team = Some(team.clone()); // Cache value for later use. 
                return Ok(team.clone());
            }
        }

        Err(TeamCommandRequestError)
    }
    
    /// Responds to an interaction with the `/team` command's message components (e.g. select menus).
    pub async fn component_interaction_response(
        &mut self,
        ctx: &Context,
        command_interaction: &CommandInteraction,
        component_interaction: &ComponentInteraction,
        robotevents: &RobotEvents,
        vrc_data_analysis: &VRCDataAnalysis,
        skills_cache: &SkillsCache,
    ) -> CreateInteractionResponse {
        if let ComponentInteractionDataKind::StringSelect { values } = &component_interaction.data.kind {
            let changed_value: &str = values.first().unwrap().as_ref();

            let message_edit = if changed_value.starts_with("option_team_") { // User changed page
                self.current_page = changed_value.trim_start_matches("option_team_").parse::<EmbedPage>().unwrap();

                command_interaction
                    .edit_response(
                        &ctx,
                        EditInteractionResponse::new()
                            .embed(self.embed(self.current_page, robotevents, vrc_data_analysis, skills_cache).await)
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

                // Reset season-specific information.
                self.awards = None;
                self.events = None;
                self.skills_ranking = None;

                command_interaction
                    .edit_response(
                        &ctx,
                        EditInteractionResponse::new()
                            .embed(self.embed(self.current_page, robotevents, vrc_data_analysis, skills_cache).await)
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
