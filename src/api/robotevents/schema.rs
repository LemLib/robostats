use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RobotEventsResponse<T> {
    pub meta: Meta,
    pub data: Vec<T>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Grade {
    College,

    #[serde(rename = "High School")]
    HighSchool,

    #[serde(rename = "Middle School")]
    MiddleSchool,

    #[serde(rename = "Elementary School")]
    ElementarySchool
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::College => "College",
            Self::HighSchool => "High School",
            Self::MiddleSchool => "Middle School",
            Self::ElementarySchool => "Elementary School"
        })
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Meta {
    current_page: i32,
    first_page_url: String,
    from: i32,
    last_page: i32,
    last_page_url: String,
    next_page_url: Option<String>,
    path: String,
    per_page: i32,
    to: i32,
    total: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Team {
    pub id: i32,
    pub number: String,
    pub team_name: String,
    pub robot_name: Option<String>,
    pub organization: String,
    pub location: Location,
    pub registered: bool,
    pub program: IdInfo,
    pub grade: Grade,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub venue: Option<String>,
    pub address_1: String,
    pub address_2: Option<String>,
    pub city: String,
    pub region: String,
    pub postcode: String,
    pub country: String,
    pub coordinates: Coordinates,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coordinates {
    pub lat: f32,
    pub lon: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdInfo {
    pub id: i32,
    pub name: String,
    pub code: String,
}