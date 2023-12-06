use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TeamInfo {
    ap_per_match: f64,
    awp_per_match: f64,
    ccwm: f64,
    dpr: f64,
    mu: f64,
    opr: f64,
    score_auto_max: f64,
    score_driver_max: f64,
    score_total_max: f64,
    sigma: f64,
    team_name: String,
    team_number: String,
    total_losses: i64,
    total_ties: i64,
    total_wins: i64,
    trueskill: f64,
    trueskill_ranking: i64,
    wp_per_match: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Prediction {
    blue1: String,
    blue2: String,
    prediction_msg: String,
    red1: String,
    red2: String,
    red_win_probability: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CCWM {
    b1_awp_per_match: f64,
    b2_awp_per_match: f64,
    blue_strength: f64,

    message: String,
    r1_awp_per_match: f64,
    r2_awp_per_match: f64,
    red_strength: f64,
}

async fn team_info(team_number: String) -> Result<TeamInfo, Error> {
    let data: TeamInfo = Client::new()
        .get(format!(
            "https://vrc-data-analysis.com/v1/team/{}",
            team_number
        ))
        .send()
        .await?
        .json()
        .await?;
    Ok(data)
}

async fn predict(
    red1: String,
    red2: String,
    blue1: String,
    blue2: String,
) -> Result<Prediction, Error> {
    let data: Prediction = Client::new()
        .get(format!(
            "https://vrc-data-analysis.com/v1/predict/{}/{}/{}/{}",
            red1, red2, blue1, blue2
        ))
        .send()
        .await?
        .json()
        .await?;
    Ok(data)
}

async fn ccwm(red1: String, red2: String, blue1: String, blue2: String) -> Result<CCWM, Error> {
    let data: CCWM = Client::new()
        .get(format!(
            "https://vrc-data-analysis.com/v1/ccwmstrength/{}/{}/{}/{}",
            red1, red2, blue1, blue2
        ))
        .send()
        .await?
        .json()
        .await?;
    Ok(data)
}
