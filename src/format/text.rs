use chrono::{Local, TimeZone};

use super::{Event, Renderer};

pub struct TextRenderer;

fn timestamp(time: i64) -> String {
    match Local.timestamp_opt(time, 0).single() {
        Some(dt) => dt.format("[%Y-%m-%d %H:%M:%S]").to_string(),
        None => "[????-??-?? ??:??:??]".to_string(),
    }
}

impl Renderer for TextRenderer {
    fn extension(&self) -> &'static str {
        "log"
    }

    fn header(&self, network: &str, buffer: &str) -> String {
        format!("--- Log of {buffer} on {network} ---\n")
    }

    fn footer(&self) -> String {
        String::new()
    }

    fn render(&self, time: i64, event: &Event) -> String {
        let ts = timestamp(time);
        match event {
            Event::Message { sender, text } => format!("{ts} <{sender}> {text}\n"),
            Event::Notice { sender, text } => format!("{ts} -{sender}- {text}\n"),
            Event::Action { sender, text } => format!("{ts} * {sender} {text}\n"),
            Event::NickChange { old, new } => {
                format!("{ts} *** {old} is now known as {new}\n")
            }
            Event::ModeChange { sender, mode } => {
                format!("{ts} *** {sender} sets mode: {mode}\n")
            }
            Event::Join { nick, host, channel } => {
                format!("{ts} *** {nick} ({host}) has joined {channel}\n")
            }
            Event::Part {
                nick,
                host,
                channel,
                reason,
            } => match reason {
                Some(reason) => {
                    format!("{ts} *** {nick} ({host}) has left {channel} ({reason})\n")
                }
                None => format!("{ts} *** {nick} ({host}) has left {channel}\n"),
            },
            Event::Quit { nick, host, reason } => {
                format!("{ts} *** {nick} ({host}) has quit ({reason})\n")
            }
            Event::Kick { target, by, reason } => match reason {
                Some(reason) => {
                    format!("{ts} *** {target} was kicked by {by} ({reason})\n")
                }
                None => format!("{ts} *** {target} was kicked by {by}\n"),
            },
            Event::Kill { target, reason } => {
                format!("{ts} *** {target} was killed ({reason})\n")
            }
            Event::TopicChange { user, channel, topic } => {
                format!("{ts} *** {user} changes topic for {channel} to '{topic}'\n")
            }
            Event::TopicJoin1 { channel, topic } => {
                format!("{ts} *** Topic for {channel} is '{topic}'\n")
            }
            Event::TopicJoin2 { user, logtime } => {
                format!("{ts} *** Topic set by {user} on {logtime}\n")
            }
            Event::Server { text } => format!("{ts} *** {text}\n"),
            Event::Info { text } => format!("{ts} *** {text}\n"),
            Event::Error { text } => format!("{ts} *** Error: {text}\n"),
            Event::DayChange { text } => {
                if text.is_empty() {
                    format!("{ts} --- Day changed ---\n")
                } else {
                    format!("{ts} --- Day changed ({text}) ---\n")
                }
            }
            Event::Other => String::new(),
        }
    }
}
