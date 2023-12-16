use crate::api::robotevents::schema::*;
use reqwest::header::USER_AGENT;
use std::time::Duration;

#[derive(Default, Debug, Clone)]
pub struct RobotEvents {
    pub bearer_token: String,
    pub req_client: reqwest::Client,
}

pub const V2_API_BASE: &str = "https://www.robotevents.com/api/v2";
pub const V1_API_BASE: &str = "https://www.robotevents.com/api/v1";

impl RobotEvents {
    pub fn new(bearer_token: impl AsRef<str>) -> Self {
        Self {
            bearer_token: bearer_token.as_ref().to_owned(),
            req_client: reqwest::Client::new()
        }
    }

    /// Make a request to a [RobotEvents API v2](https://www.robotevents.com/api/v2) endpoint.
    /// 
    /// Requires a bearer authentication token to be provided for requests to work. This can
    /// be obtained from RobotEvents by creating an account and requesting one.
    async fn request(&self, endpoint: impl AsRef<str>) -> Result<reqwest::Response, reqwest::Error> {
        Ok(self
            .req_client
            .get(format!("{V2_API_BASE}{}", endpoint.as_ref()))
            .header("accept-language", "en")
            .header(USER_AGENT, "RoboStats Discord Bot")
            .bearer_auth(&self.bearer_token)
            .timeout(Duration::from_secs(10))
            .send()
            .await?)
    }

    /// Make a request to a RobotEvents API v1 endpoint.
    async fn request_api_v1(&self, endpoint: impl AsRef<str>) -> Result<reqwest::Response, reqwest::Error> {
        Ok(self
            .req_client
            .get(format!("{V1_API_BASE}{}", endpoint.as_ref()))
            .header("accept-language", "en")
            .header(USER_AGENT, "RoboStats Discord Bot")
            .timeout(Duration::from_secs(10))
            .send()
            .await?)
    }

    pub async fn find_teams(&self, team_number: impl AsRef<str>, program_filter: Option<i32>) -> Result<Vec<Team>, reqwest::Error> {
        let url = if let Some(filter) = program_filter {
            format!("/teams?number%5B%5D={}&program%5B%5D={}", team_number.as_ref(), filter)
        } else {
            format!("/teams?number%5B%5D={}", team_number.as_ref())
        };

        let response = self.request(url).await?;

        Ok(response.json::<PaginatedResponse<Team>>().await?.data)
    }

    pub async fn all_seasons(&self) -> Result<Vec<Season>, reqwest::Error> {
        let response = self.request("/seasons").await?;

        Ok(response.json::<PaginatedResponse<Season>>().await?.data)
    }

    pub async fn all_programs(&self) -> Result<Vec<IdInfo>, reqwest::Error> {
        let response = self.request("/programs").await?;

        Ok(response.json::<PaginatedResponse<IdInfo>>().await?.data)
    }

    pub async fn team_active_seasons(&self, team: &Team) -> Result<Vec<Season>, reqwest::Error> {
        let response = self.request(format!("/seasons?team%5B%5D={}", team.id)).await?;

        Ok(response.json::<PaginatedResponse<Season>>().await?.data)
    }

    pub async fn team_awards(&self, team: &Team, season_filter: Option<i32>) -> Result<Vec<Award>, reqwest::Error> {
        let url = if let Some(filter) = season_filter {
            format!("/teams/{}/awards?season%5B%5D={}", team.id, filter)
        } else {
            format!("/teams/{}/awards", team.id)
        };

        let response = self.request(url).await?;

        Ok(response.json::<PaginatedResponse<Award>>().await?.data)
    }

    pub async fn team_skills_runs(&self, team: &Team, season_filter: Option<i32>) -> Result<Vec<Skill>, reqwest::Error> {
        let url = if let Some(filter) = season_filter {
            format!("/teams/{}/skills?season%5B%5D={}", team.id, filter)
        } else {
            format!("/teams/{}/skills", team.id)
        };

        let response = self.request(url).await?;

        Ok(response.json::<PaginatedResponse<Skill>>().await?.data)
    }
}
