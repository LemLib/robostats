use std::time::Duration;

use reqwest::header::USER_AGENT;

use crate::api::robotevents::schema::*;

pub struct RobotEvents {
    pub bearer_token: String,
    pub req_client: reqwest::Client,
}

pub const API_BASE: &str = "https://www.robotevents.com/api/v2";

impl RobotEvents {
    pub fn new(bearer_token: impl AsRef<str>) -> Self {
        Self {
            bearer_token: bearer_token.as_ref().to_owned(),
            req_client: reqwest::Client::new(),
        }
    }

    async fn request(&self, endpoint: impl AsRef<str>) -> Result<reqwest::Response, reqwest::Error> {
        self.req_client.get(format!("{API_BASE}{}", endpoint.as_ref())).header("accept-language", "en").header(USER_AGENT, "RoboStats Discord Bot").bearer_auth(&self.bearer_token).timeout(Duration::from_secs(10)).send().await
    }

    pub async fn find_team_program(&self, team_number: &str, program: &i64, any_program: bool) -> Result<Vec<Team>, reqwest::Error> {
        let response = self.request(format!("/teams?number%5B%5D={team_number}&program%5B%5D={program}")).await;
        match response {
            Ok(_) => {
                let json = response.unwrap().json::<RobotEventsResponse<Team>>().await;
                match json {
                    Ok(t) => Ok(t.data),
                    Err(e) => if any_program {
                        self.find_teams_any_program(team_number).await

                    } else {
                        Err(e)
                    }
                }
            }
            Err(e) => if any_program {
                self.find_teams_any_program(team_number).await
            } else {
                Err(e)
            },
        }
    }
    pub async fn find_teams_any_program(&self, team_number: &str) -> Result<Vec<Team>, reqwest::Error> {
        let response = self.request(format!("/teams?number%5B%5D={team_number}")).await;
        match response {
            Ok(_) => {
                let json = response.unwrap().json::<RobotEventsResponse<Team>>().await;
                match json {
                    Ok(t) => Ok(t.data),
                    Err(e) => Err(e),
                }
            },
            Err(e) => Err(e),
        }
    }

    pub async fn all_seasons(&self) -> Result<Vec<Season>, reqwest::Error> {
        let response = self.request("/seasons").await;
        match response {
            Ok(_) => {
                let json = response.unwrap().json::<RobotEventsResponse<Season>>().await;
                match json {
                    Ok(t) => Ok(t.data),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e)
        }
    }

    pub async fn team_active_seasons(&self, team: &Team) -> Result<Vec<Season>, reqwest::Error> {
        let response = self.request(format!("/seasons?team%5B%5D={}", team.id)).await;
        match response {
            Ok(_) => {
                let json = response.unwrap().json::<RobotEventsResponse<Season>>().await;
                match json {
                    Ok(t) => Ok(t.data),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e)
        }
    }
}
