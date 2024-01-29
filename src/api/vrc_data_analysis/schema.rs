use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct TeamInfo {
    pub ap_per_match: f64,
    pub awp_per_match: f64,
    pub ccwm: f64,
    pub dpr: f64,
    pub mu: f64,
    pub opr: f64,
    pub score_auto_max: Option<f64>,
    pub score_driver_max: Option<f64>,
    pub score_total_max: Option<f64>,
    pub sigma: f64,
    pub team_name: String,
    pub team_number: String,
    pub total_losses: i64,
    pub total_ties: i64,
    pub total_wins: i64,
    pub trueskill: f64,
    pub trueskill_ranking: i64,
    pub wp_per_match: f64,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct Prediction {
    pub blue1: String,
    pub blue2: String,
    pub prediction_msg: String,
    pub red1: String,
    pub red2: String,
    pub red_win_probability: f64,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct CCWM {
    pub b1_awp_per_match: f64,
    pub b2_awp_per_match: f64,
    pub blue_strength: f64,

    pub message: String,
    pub r1_awp_per_match: f64,
    pub r2_awp_per_match: f64,
    pub red_strength: f64,
}