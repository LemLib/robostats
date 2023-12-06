use crate::api::robotevents::schema::*;
use reqwest::header::USER_AGENT;
use std::time::Duration;

pub struct VrcDataAnalysis {
    pub bearer_token: String,
    pub req_client: reqwest::Client,
}

pub const API_BASE: &str = "https://vrc-data-analysis.com/v1";

#[derive(Default, Debug, Clone, PartialEq)]
pub struct RobotEventsError;

impl VrcDataAnalysis {
    async fn request(&self, endpoint: impl AsRef<str>) -> Result<reqwest::Response, reqwest::Error> {
        Ok(self
            .req_client
            .get(format!("{API_BASE}{}", endpoint.as_ref()))
            .header("accept-language", "en")
            .header(USER_AGENT, "RoboStats Discord Bot")
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .unwrap())
    }

    pub async fn team_data(&self, team_number: &str, program: i32) -> Result<Vec<Team>, reqwest::Error> {
        let response = self.request(format!("/team/{team_number}")).await?;
        Ok(response.json::<RobotEventsResponse<Team>>().await?.data)
    }

}
