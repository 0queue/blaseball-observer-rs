use anyhow::anyhow;
use serde_json::Value;
use crate::game_event::PlayBall;
use crate::game_event::GameEvent;
use crate::game_event::NotifyGameStart;

mod emoji;
mod weather;
mod event_source;
mod game_event;

const ALL_TEAMS_ENDPOINT: &str = "https://www.blaseball.com/database/allTeams";
const STREAM_ENDPOINT: &str = "https://www.blaseball.com/events/streamData";

/// Watch a game of blaseball
#[derive(argh::FromArgs, Debug)]
struct Args {
    /// team nickname to watch, or empty to love da crabs
    #[argh(positional, default = "\"crabs\".to_string()")]
    team_name: String
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Team {
    id: String,
    // actually a uuid but whatever
    full_name: String,
    location: String,
    nickname: String,
    shorthand: String,
    #[serde(deserialize_with = "emoji::deserialize")]
    emoji: char,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    id: String,
    bases_occupied: Vec<i32>,
    last_update: String,
    away_bases: u32,
    away_pitcher_name: String,
    away_team: String,
    away_team_name: String,
    away_team_nickname: String,
    away_team_color: String,
    #[serde(deserialize_with = "emoji::deserialize")]
    away_team_emoji: char,
    away_odds: f32,
    away_strikes: i32,
    away_score: i32,
    home_bases: u32,
    home_pitcher_name: String,
    home_team: String,
    home_team_name: String,
    home_team_nickname: String,
    home_team_color: String,
    #[serde(deserialize_with = "emoji::deserialize")]
    home_team_emoji: char,
    home_odds: f32,
    home_strikes: i32,
    home_score: i32,
    game_complete: bool,
    game_start: bool,
    half_inning_outs: i32,
    inning: i32,
    top_of_inning: bool,
    at_bat_balls: u32,
    at_bat_strikes: u32,
    #[serde(deserialize_with = "weather::deserialize")]
    weather: String,
}

fn fetch_teams() -> anyhow::Result<Vec<Team>> {
    let body = ureq::get(ALL_TEAMS_ENDPOINT).call().into_reader();
    let v: Vec<Team> = serde_json::from_reader(body)?;
    return Ok(v);
}

fn message_to_games(msg: &event_source::Message) -> Result<Vec<Game>, serde_json::Error> {
    serde_json::from_str::<Value>(&msg.data).and_then(|v| {
        let games = v["value"]["games"]["schedule"].clone();
        serde_json::from_value::<Vec<Game>>(games)
    })
}

fn main() -> anyhow::Result<()> {
    let args: Args = argh::from_env();
    pretty_env_logger::init();

    let teams = fetch_teams()?;
    let target = teams.iter().find(|&e| {
        e.nickname.to_ascii_lowercase() == args.team_name.to_ascii_lowercase()
    }).ok_or(anyhow!("no team found"))?;

    println!("target: {:?}", target);

    let schedule_events = event_source::EventSource::new(STREAM_ENDPOINT)
        .flat_map(|m| {
            let json = message_to_games(&m);

            if json.is_err() {
                log::warn!("error while decoding {:?}", &json)
            }

            json
        })
        .flat_map(|gs| {
            let game = gs.into_iter().find(|g| {
                g.away_team == target.id || g.home_team == target.id
            });

            if game.is_none() {
                log::warn!("No game found for the {}", target.full_name);
            }

            game
        });

    let mut prev: Option<Game> = None;
    for cur in schedule_events {
        println!("Found game {}: {} @ {}", cur.id, cur.away_team_emoji, cur.home_team_emoji);

        PlayBall::accept(&prev, &cur);
        NotifyGameStart::accept(&prev, &cur);

        prev.replace(cur);
    }

    println!("all done");
    Ok(())
}
