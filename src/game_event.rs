use notify_rust::Notification;

use crate::emoji::pad;
use crate::Game;

pub trait GameEvent {
    fn accept(prev: &Option<Game>, cur: &Game);
}

pub struct PlayBall;

impl GameEvent for PlayBall {
    fn accept(prev: &Option<Game>, cur: &Game) {
        // return if not started yet, or both already started
        let is_first = match (prev, cur.game_start) {
            (None, true) => true,
            (Some(ref p), true) if !p.game_start => true,
            _ => false
        };

        if !is_first {
            return;
        }

        let away = format!(
            "{:2}{} ({:.2}%)",
            cur.away_team_emoji,
            cur.away_team_name,
            cur.away_odds * 100f32
        );

        let home = format!(
            "{:2}{} ({:.2}%)",
            cur.home_team_emoji,
            cur.home_team_name,
            cur.home_odds * 100f32
        );

        let away_pitcher = format!(
            "{:2}{}",
            cur.away_team_emoji,
            cur.away_pitcher_name
        );

        let home_pitcher = format!(
            "{:2}{}",
            cur.home_team_emoji,
            cur.home_pitcher_name
        );

        println!("> Play ball! {} @ {}", away, home);
        println!("> Pitching: {} and {}", away_pitcher, home_pitcher);
        println!("> Weather: {}", cur.weather)
    }
}

pub struct NotifyGameStart;

impl GameEvent for NotifyGameStart {
    fn accept(prev: &Option<Game>, cur: &Game) {
        let is_first = match (prev, cur.game_start) {
            (Some(ref p), true) if !p.game_start => true,
            _ => false
        };

        if !is_first {
            return;
        }

        let away = format!(
            "{:2}{} ({:.2}%)",
            cur.away_team_emoji,
            cur.away_team_name,
            cur.away_odds * 100f32
        );

        let home = format!(
            "{:2}{} ({:.2}%)",
            cur.home_team_emoji,
            cur.home_team_name,
            cur.home_odds * 100f32
        );

        Notification::new()
            .summary("Play ball!")
            .body(&format!("{} @ {}", away, home))
            .show()
            .unwrap();
    }
}

const TOP: char = '\u{2584}';
const BOTTOM: char = '\u{25BE}';
const OUT: char = '\u{25CF}';
const NOT_OUT: char = '\u{25CB}';
const BASE_EMPTY: char = '\u{25C7}';
const BASE_OCCUPIED: char = '\u{25C6}';
const BLASEBALL: char = '\u{26BE}';
const ROTATE: char = '\u{27F3}';

pub struct Status;

fn event(prev: &Option<Game>, cur: &Game) -> String {
    match prev {
        None => "  ".to_string(),
        Some(p) if p.away_score < cur.away_score => pad(BLASEBALL),
        Some(p) if p.home_score < cur.home_score => pad(BLASEBALL),
        Some(p) if p.top_of_inning != cur.top_of_inning => pad(ROTATE),
        Some(p) if p.half_inning_outs < cur.half_inning_outs => pad(OUT),
        _ => "  ".to_string(),
    }
}

impl GameEvent for Status {
    fn accept(prev: &Option<Game>, cur: &Game) {
        if cur.game_complete {
            return;
        }

        let inning = format!(
            "{}{:2}",
            pad(if cur.top_of_inning { TOP } else { BOTTOM }),
            cur.inning + 1
        );

        let away_score = format!(
            "{}{:2}",
            pad(cur.away_team_emoji),
            cur.away_score
        );

        let home_score = format!(
            "{}{:2}",
            pad(cur.away_team_emoji),
            cur.home_score
        );

        let num_bases = if cur.top_of_inning {
            cur.away_bases
        } else {
            cur.home_bases
        };

        let bases: String = (0..(num_bases - 1)).rev().map(|b| {
            if cur.bases_occupied.contains(&b) {
                pad(BASE_OCCUPIED)
            } else {
                pad(BASE_EMPTY)
            }
        }).collect::<Vec<String>>().join(""); // TODO format justify right

        let count = format!(
            "{}-{}",
            cur.at_bat_balls,
            cur.at_bat_strikes
        );

        let outs = format!(
            "{} {}",
            if cur.half_inning_outs > 0 { OUT } else { NOT_OUT },
            if cur.half_inning_outs > 1 { OUT } else { NOT_OUT }
        );

        let now = chrono::Local::now().format("%H:%M").to_string();

        let status = format!(
            "[{n}][{e}] {i}{a}{h} | {b:>w$} {c} {o} : {u}",
            n = now,
            e = event(prev, cur),
            i = inning,
            a = away_score,
            h = home_score,
            b = bases,
            w = std::cmp::max(cur.away_bases, cur.home_bases) as usize,
            c = count,
            o = outs,
            u = cur.last_update.trim_end()
        );

        println!("{}", status);
    }
}