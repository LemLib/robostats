use crate::api::robotevents::schema::*;
use reqwest::header::USER_AGENT;
use std::time::Duration;

pub struct RobotEvents {
    pub bearer_token: String,
    pub req_client: reqwest::Client,
}

pub const API_BASE: &str = "https://www.robotevents.com/api/v2";

#[derive(Default, Debug, Clone, PartialEq)]
pub struct RobotEventsError;

impl RobotEvents {
    pub fn new(bearer_token: impl AsRef<str>) -> Self {
        Self {
            bearer_token: bearer_token.as_ref().to_owned(),
            req_client: reqwest::Client::new()
        }
    }

    async fn request(&self, endpoint: impl AsRef<str>) -> Result<reqwest::Response, reqwest::Error> {
        Ok(self
            .req_client
            .get(format!("{API_BASE}{}", endpoint.as_ref()))
            .header("accept-language", "en")
            .header(USER_AGENT, "RoboStats Discord Bot")
            .bearer_auth(&self.bearer_token)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .unwrap())
    }

    pub async fn find_teams(&self, team_number: &str, program: &i64) -> Result<Vec<Team>, reqwest::Error> {
        let response = self.request(format!("/teams?number%5B%5D={team_number}&program%5B%5D={program}")).await?;

        Ok(response.json::<RobotEventsResponse<Team>>().await?.data)
    }

    pub async fn all_seasons(&self, team: &Team) -> Result<Vec<Season>, reqwest::Error> {
        let response = self.request("/seasons").await?;

        Ok(response.json::<RobotEventsResponse<Season>>().await?.data)
    }

    pub async fn team_active_seasons(&self, team: &Team) -> Result<Vec<Season>, reqwest::Error> {
        let response = self.request(format!("/seasons?team%5B%5D={}", team.id)).await?;

        Ok(response.json::<RobotEventsResponse<Season>>().await?.data)
    }
}
