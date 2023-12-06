use serde::{Deserialize, Serialize, Debug};
#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    ap_per_match: f64,
    awp_per_match: f64,
    ccwm: f64,
    dpr: i64,
    mu: f64,
    opr: f64,
    score_auto_max: i64,
    score_driver_max: i64,
    score_total_max: i64,
    sigma: f64,
    team_name: String,
    team_number: String,
    total_losses: i64,
    total_ties: i64,
    total_wins: i64,
    trueskill: i64,
    trueskill_ranking: i64,
    wp_per_match: f64,
}