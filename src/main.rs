use std::io::Write;

use anyhow::anyhow;
use serde_json::Value;

mod emoji;
mod event_source;

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
struct Game {
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
    weather: u32,
}

fn fetch_teams() -> anyhow::Result<Vec<Team>> {
    let body = ureq::get(ALL_TEAMS_ENDPOINT).call().into_reader();
    let v: Vec<Team> = serde_json::from_reader(body)?;
    return Ok(v);
}

fn main() -> anyhow::Result<()> {
    let args: Args = argh::from_env();
    pretty_env_logger::init();

    let teams = fetch_teams()?;
    let target = teams.iter().find(|&e| {
        e.nickname.to_ascii_lowercase() == args.team_name.to_ascii_lowercase()
    }).ok_or(anyhow!("no team found"))?;

    println!("target: {:?}", target);

    let event_source = event_source::EventSource::new(STREAM_ENDPOINT);

    for message in event_source {
        let val = serde_json::from_str::<Value>(&message.data);

        if let Ok(v) = val {
            let games = v["value"]["games"]["schedule"].clone();
            let games: Vec<Game> = match serde_json::from_value(games) {
                Ok(g) => g,
                Err(e) => {
                    eprintln!("could not deserialize value.games.schedule");
                    eprintln!("{:?}", e);
                    continue;
                }
            };

            let target_game = match games.iter().find(|&g| g.away_team == target.id || g.home_team == target.id) {
                Some(g) => g,
                None => continue
            };

            println!("Found game {}: {} @ {}", target_game.id, target_game.away_team_emoji, target_game.home_team_emoji);
        } else {
            eprintln!("received non json data")
        }

        std::io::stdout().flush().unwrap();
        std::io::stderr().flush().unwrap();
    }

    println!("all done");
    Ok(())
}
