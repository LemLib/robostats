use reqwest::header::USER_AGENT;
use std::time::Duration;

use crate::api::vrc_data_analysis::schema::*;

#[derive(Default, Debug, Clone)]
pub struct VRCDataAnalysis {
    pub req_client: reqwest::Client,
}

pub const API_BASE: &str = "https://vrc-data-analysis.com/v1";

impl VRCDataAnalysis {
    pub fn new() -> Self {
        Self {
            req_client: reqwest::Client::new()
        }
    }

    async fn request(
        &self,
        endpoint: impl AsRef<str>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        Ok(self
            .req_client
            .get(format!("{API_BASE}{}", endpoint.as_ref()))
            .header("accept-language", "en")
            .header(USER_AGENT, "RoboStats Discord Bot")
            .timeout(Duration::from_secs(10))
            .send()
            .await?)
    }

    pub async fn team_info(&self, team_number: &str) -> Result<TeamInfo, reqwest::Error> {
        let response = self.request(format!("/team/{team_number}")).await?;

        Ok(response.json().await?)
    }

    pub async fn predict_match(
        &self,
        red_alliance: (&str, &str),
        blue_alliance: (&str, &str),
    ) -> Result<Prediction, reqwest::Error> {
        let response = self
            .request(format!(
                "/predict/{}/{}/{}/{}",
                red_alliance.0, red_alliance.1, blue_alliance.0, blue_alliance.1
            ))
            .await?;

        Ok(response.json().await?)
    }

    #[allow(unused)]
    pub async fn ccwm(
        &self,
        red_alliance: (&str, &str),
        blue_alliance: (&str, &str),
    ) -> Result<CCWM, reqwest::Error> {
        let response = self
            .request(format!(
                "/ccwmstrength/{}/{}/{}/{}",
                red_alliance.0, red_alliance.1, blue_alliance.0, blue_alliance.1
            ))
            .await?;

        Ok(response.json().await?)
    }
}