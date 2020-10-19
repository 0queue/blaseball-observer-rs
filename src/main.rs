use anyhow::anyhow;
use anyhow::Context;
use serde_json::Value;
use crate::game_event::PlayBall;
use crate::game_event::Status;
use crate::game_event::GameOver;
use crate::game_event::GameEvent;
use crate::game_event::NotifyGameStart;
use std::io::Write;

mod emoji;
mod weather;
mod event_source;
mod game_event;

const ALL_TEAMS_ENDPOINT: &str = "https://www.blaseball.com/database/allTeams";
const STREAM_ENDPOINT: &str = "https://www.blaseball.com/events/streamData";

/// Watch a game of blaseball
///
/// RIV Baltimore Crabs
#[derive(argh::FromArgs, Debug)]
struct Args {
    /// team nickname to watch
    #[argh(positional)]
    team_name: String,

    /// whether to use knowledge of blaseball to sleep longer between reconnects
    #[argh(switch)]
    long_sleep: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    id: String,
    // actually a uuid but whatever
    full_name: String,
    location: String,
    nickname: String,
    shorthand: String,
    #[serde(deserialize_with = "emoji::deserialize")]
    emoji: char,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    id: String,
    bases_occupied: Vec<u32>,
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
    serde_json::from_reader(body).with_context(|| anyhow!("could not decode from teams endpoint {}"))
}

fn message_to_games(msg: &event_source::Message) -> Result<Vec<Game>, serde_json::Error> {
    serde_json::from_str::<Value>(&msg.data).and_then(|v| {
        let games = v["value"]["games"]["schedule"].clone();
        serde_json::from_value::<Vec<Game>>(games)
    })
}

// TODO LOTS of duplicate log messages, and no idea why
fn main() -> anyhow::Result<()> {
    let args: Args = argh::from_env();
    pretty_env_logger::init();

    let teams = fetch_teams()?;
    let rooting_for = teams.iter().find(|&e| {
        e.nickname.to_ascii_lowercase() == args.team_name.to_ascii_lowercase()
    }).ok_or_else(|| anyhow!("no team found"))?;

    println!("Rooting for the {} {}", emoji::pad(rooting_for.emoji), rooting_for.full_name);

    let schedule_events = event_source::EventSource::new(STREAM_ENDPOINT, args.long_sleep)
        .flat_map(|m| {
            let json = message_to_games(&m);

            if json.is_err() {
                log::warn!("error while decoding {:?}", &json)
            }

            json
        })
        .flat_map(|gs| {
            let len = gs.len();
            let game = gs.into_iter().find(|g| {
                g.away_team == rooting_for.id || g.home_team == rooting_for.id
            });

            if game.is_none() {
                log::warn!("No game found for the {}, of {} games", rooting_for.full_name, len);
            }

            game
        });

    let mut prev: Option<Game> = None;
    for cur in schedule_events {
        PlayBall::accept(&prev, &cur, &rooting_for);
        NotifyGameStart::accept(&prev, &cur, &rooting_for);
        Status::accept(&prev, &cur, &rooting_for);
        GameOver::accept(&prev, &cur, &rooting_for);

        prev.replace(cur);

        std::io::stdout().flush().unwrap();
        std::io::stderr().flush().unwrap();
    }

    println!("all done");
    Ok(())
}
