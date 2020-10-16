use std::io::BufReader;
use std::io::BufRead;
use std::io::Error;
use std::io::Read;
use std::borrow::BorrowMut;
use std::string::FromUtf8Error;
use std::cmp::min;
use std::path::Display;

pub struct EventSource {
    req: ureq::Request,
    reader: Option<BufReader<Box<dyn Read>>>,
}

#[derive(Debug)]
pub struct Event {
    pub data: String
}

impl EventSource {
    pub fn new(url: &str) -> EventSource {
        EventSource {
            req: ureq::get(url),
            reader: None,
        }
    }

    // none iff done
    pub fn next_line(&mut self) -> Option<String> {
        let mut buf = Vec::new();

        loop {
            match self.reader.borrow_mut() {
                None => {
                    let response = self.req.call();
                    self.reader.replace(BufReader::new(Box::new(response.into_reader())));
                }
                Some(reader) => {
                    let res = reader.read_until(b'\n', &mut buf);

                    match res {
                        Ok(size) => if size == 0 {
                            return None;
                        } else {
                            if let Ok(s) = String::from_utf8(buf) {
                                return Some(s)
                            }

                            buf = Vec::new()
                        }
                        Err(_) => {
                            // need to reconnect after sleep
                            eprintln!("reconnecting ...");
                            self.reader.take();
                            std::thread::sleep(std::time::Duration::from_millis(3000));
                            continue;
                        }
                    }
                }
            }
        }
    }

    pub fn next_message(&mut self) -> Option<Event> {
        let mut data = String::new();

        // only handle data: lines for now
        loop {
            if let Some(s) = self.next_line() {
                if s == "\n" {
                    break;
                }

                if !s.starts_with("data: ") {
                    let len = min(5, s.len());
                    eprintln!("line starts with {}", &s[0..len]);
                    continue;
                }

                if !data.is_empty() {
                    data.push('\n');
                }

                data.push_str(s.trim_start_matches("data: "));
            } else {
                return None;
            }
        }

        Some(Event { data })
    }
}

impl Iterator for EventSource {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_message()
    }
}