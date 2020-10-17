use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::time::Duration;

use thiserror::Error;

use crate::event_source::EventSourceError::UreqError;
use chrono::Timelike;

pub struct EventSource {
    req: ureq::Request,
    reader: Option<BufReader<Box<dyn Read>>>,
    delay: Duration,
    last_event_id: Option<String>,
    long_sleep: bool,
}

#[derive(Debug)]
pub struct Message {
    pub event: Option<String>,
    pub data: String,
    pub last_event_id: Option<String>,
}

#[derive(Error, Debug)]
pub enum EventSourceError {
    #[error("Error while making request")]
    UreqError(u16, String),
    #[error("Error while converting to utf-8")]
    UtfError(#[from] std::string::FromUtf8Error),
}

impl EventSource {
    pub fn new(url: &str, long_sleep: bool) -> EventSource {
        EventSource {
            req: ureq::get(url),
            reader: None,
            delay: Duration::from_millis(3000),
            last_event_id: None,
            long_sleep,
        }
    }

    // Ok(empty) and errors are end signals
    fn next_buf(&mut self) -> Result<Vec<u8>, EventSourceError> {
        let mut buf = Vec::new();

        loop {
            let mut reader = match self.reader.take() {
                None => {
                    let response = self.req.call();

                    if let Some(e) = response.synthetic_error() {
                        break Err(UreqError(e.status(), e.body_text()));
                    }

                    if response.status() == 204 {
                        // all done
                        break Ok(Vec::new());
                    }

                    BufReader::new(Box::new(response.into_reader()) as Box<dyn Read>)
                }
                Some(r) => r
            };

            // TODO read until either \cr or \lf, whichever is first/next to each other, per spec
            let res = reader.read_until(b'\n', &mut buf);

            if res.is_err() {
                buf.clear();
                log::warn!("Reconnecting...");
                let delay = if self.long_sleep {
                    self.calculate_delay().unwrap_or_else(|| {
                        log::warn!("Unable to calculate delay, using default");
                        self.delay
                    })
                } else {
                    self.delay
                };
                std::thread::sleep(delay);
                continue;
            }

            self.reader.replace(reader);

            break Ok(buf);
        }
    }

    fn next_line(&mut self) -> Result<String, EventSourceError> {
        self.next_buf().and_then(|buf| {
            String::from_utf8(buf)
                .map_err(EventSourceError::UtfError)
        })
    }

    fn next_message(&mut self) -> Result<Option<Message>, EventSourceError> {
        let mut event: Option<String> = None;
        let mut data = String::new();

        loop {
            let line = self.next_line()?;

            if line.is_empty() {
                // empty line but message isn't complete (no second newline)
                return Ok(None);
            }

            if line == "\n" {
                // dispatch time
                break;
            }

            let (head, tail) = match line.find(':') {
                Some(0) => continue, // comment!
                Some(i) => {
                    (&line[..i], &line[i..])
                }
                None => {
                    continue;
                }
            };

            // because index is inclusive, and space is optional, new line isn't included
            let content = tail.trim_start_matches(':')
                .trim_start_matches(' ')
                .trim_end();

            match head {
                "event" => { event.replace(content.to_string()); }
                "data" => {
                    data.push_str(content);
                    data.push('\n');
                }
                "id" => {
                    if !content.contains('\0') {
                        self.last_event_id = Some(content.to_string());
                    }
                }
                "retry" => {
                    if let Ok(ms) = content.parse::<u32>() {
                        self.delay = Duration::from_millis(ms as u64);
                    }
                }
                _ => continue, // ignore field
            };
        }

        Ok(Some(Message {
            event,
            data,
            last_event_id: self.last_event_id.clone(),
        }))
    }

    fn calculate_delay(&self) -> Option<std::time::Duration> {
        // For efficiency reasons, we don't want to check every 30 seconds for games,
        // we can use the fact that blaseball tends to start on the hour to calculate
        // a sleep interval that will sleep for most of that time
        // Inspired by exponential backoff but in reverse
        let now = chrono::Local::now();
        let next_hour = now.with_minute(0)
            .and_then(|t| t.with_second(0))
            .and_then(|t| t.with_nanosecond(0))
            .map(|t| t + (chrono::Duration::hours(1) - chrono::Duration::minutes(3)))?;

        let delta = (next_hour - now) / 2;

        if delta < chrono::Duration::zero() {
            return Some(self.delay);
        }

        let sleep_duration = std::cmp::max(delta.to_std().ok()?, self.delay);

        let then = now + chrono::Duration::from_std(sleep_duration).ok()?;
        log::info!("Sleeping until {}", then.format("%H:%M"));

        Some(sleep_duration)
    }
}

impl Iterator for EventSource {
    type Item = Message;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_message() {
            Ok(m) => m,
            Err(e) => {
                log::warn!("Error while reading event source {}", e);
                None
            }
        }
    }
}