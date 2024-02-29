use std::{collections::HashMap, time::{Instant, Duration}, sync::Arc};

use robotevents::RobotEvents;
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EventV1 {
	pub season_name: String,
	pub sku: String,
	pub start_date: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TeamV1 {
	pub affiliations: Vec<String>,
	pub city: String,
	pub country: String,
	pub event_region: String,
	pub event_region_id: i32,
	pub grade_level: String,
	pub id: i32,
	pub link: String,
	pub organization: String,
	pub program: String,
	pub region: Option<String>,
	pub team: String,
	pub team_name: String,
	pub team_reg_id: i32,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SkillsScores {
	pub combined_stop_time: i32,
	pub driver: i32,
	pub driver_scored_at: String,
	pub driver_stop_time: i32,
	pub max_driver: i32,
	pub max_programming: i32,
	pub prog_scored_at: String,
	pub prog_stop_time: i32,
	pub programming: i32,
	pub score: i32,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct SkillsRanking {
	pub rank: i32,
	pub team: TeamV1,
	pub event: EventV1,
	pub scores: SkillsScores,
	pub eligible: bool,
}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct SkillsCacheKey(i32, String);

#[derive(Default, Debug, Clone)]
pub struct SkillsCache {
	cache: Arc<Mutex<HashMap<SkillsCacheKey, (Vec<SkillsRanking>, std::time::Instant)>>>,
}

impl SkillsCache {
	pub async fn get_team_ranking(&self, team: &robotevents::schema::Team, season_id: i32, robotevents: &RobotEvents) -> Result<Option<SkillsRanking>, reqwest::Error> {
		let mut cache = self.cache.lock().await;
		let key = SkillsCacheKey(season_id, team.grade.to_string());
		
		let rankings = match cache.get(&key) {
			Some((rankings, timestamp)) if timestamp.elapsed() < Duration::from_secs(43200) => rankings,
			_ => {
				let fetched_rankings: Vec<SkillsRanking> = robotevents
					.request_api_v1(format!("/seasons/{season_id}/skills?grade_level={}&post_season=0", team.grade.to_string()))
					.await?
					.json()
					.await?;

				cache.insert(key.clone(), (fetched_rankings, Instant::now()));
				&cache.get(&key).expect("Cache should be full after being immediately updated.").0
			},
		};

		Ok(match rankings.iter().find(|ranking| ranking.team.id == team.id) {
			Some(r) => Some(r.clone()),
			None => None,
		})
	}
}