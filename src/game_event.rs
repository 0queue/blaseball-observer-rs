use crate::Game;
use notify_rust::Notification;

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

        println!("Play ball! {} @ {}", away, home);
        println!("Pitching: {} and {}", away_pitcher, home_pitcher);
        println!("Weather: {}", cur.weather)
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