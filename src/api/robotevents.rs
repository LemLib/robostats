use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    data: Vec<Team>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Team {
    id: i64,
    number: String,
    team_name: String,
    robot_name: String,
    organization: String,
    location: Location,
    registered: bool,
    program: Program,
    grade: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    city: String,
    region: String,
    postcode: String,
    country: String,
    coordinates: Coordinates,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Coordinates {
    lat: f64,
    lon: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Program {
    id: i64,
    name: String,
    code: String,
}


//This might not work also idc about the api authorization token being left in
async fn team_info() -> Result<(), Error> {
    let data: Data = Client::new()
        .get("https://www.robotevents.com/api/v2/teams?number%5B%5D=TEAM NUMBER HERE&program%5B%5D=1&myTeams=false")
        .header("Authorization", "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJhdWQiOiIzIiwianRpIjoiNDRiMmZmZTlkZGZlZTg1N2VjNGY3MWI2Y2I5OTkyZjNjYzIzYTI4NzdmMTY5NGExMzUzNWJlZTBjY2I2NGI5YTVjMWQ4MjlmMmFjZTcxNjEiLCJpYXQiOjE3MDE3MjMwMDUuNzc2MDk0LCJuYmYiOjE3MDE3MjMwMDUuNzc2MDk3MSwiZXhwIjoyNjQ4NDk0MjA1Ljc2NjEyNTIsInN1YiI6IjExMzQ4NiIsInNjb3BlcyI6W119.iUofrpTGHmKIUvOw2CcG5E9yDgDl6ZNT7UGDkS49q1dEhutfmQQiR8Eic4_iCxP74ZHqoRCmsu0L5LDzyrgJ2Jbv6p7XnSvjQlFsS32FGH_5s2bhkXAmnp_PU7ElqIlJ-zda5xV7OR3UWZz_GrE_6PGWjtTPWaeAwckBWYVCLQM7dVmlDKofEmv28fea3Y711UxD7Y1c3adZP9Ja47esw2sQ8Ae1OtUoPZ-wfkiMiApTzSovUJ27SJEuldD7TZMKFVEKkXz39PHeDVLk6mnFKn4Xc20Y2rHsAgoRSZgC5A9f8uDkUqVmM8L0kRkS7NuOX0A7bqO8H6f5CiaZhNON6VRi2FzIvddYQub45xfdVC6BhjaI_OBd7cWNe4jebWYIs9aOYni119B0FN4DAhqImA-I0TbGvDvxFLMmOzIty8wNFSGOmZzymIXKc1m4_T6TaOhVWBZv4sDo5ABIVUJuXP1juR53iyiU5QHRaZRYlRSWh6JImfCOZF4OjBbTsNtORaZB7Sarv3tjNNCMCpwD4GrjK6ZitxawKVArz5WDr6lEBXhiRpuViatn1xGJo-X2coAJzHBew7rUJUuk5bqRhurOJiE2sPm46hlghArbiHp96J3wHPgKG7TVF6NfgLIoyw2UjbmcakaRDF-6mNwctauNnLo98B7glJ4QMwvb_5U".to_owned())
        .send()
        .await?
        .json()
        .await?;
    println!("{:?}", data);
    OK(());
}
