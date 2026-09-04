mod html;
mod text;

pub use html::HtmlRenderer;
pub use text::TextRenderer;

use std::sync::LazyLock;

use regex::Regex;

use crate::db::BacklogRow;

/// Quassel backlog message type bits (see Quassel's Message::Type).
mod msg_type {
    pub const PLAIN: i32 = 0x0001;
    pub const NOTICE: i32 = 0x0002;
    pub const ACTION: i32 = 0x0004;
    pub const NICK: i32 = 0x0008;
    pub const MODE: i32 = 0x0010;
    pub const JOIN: i32 = 0x0020;
    pub const PART: i32 = 0x0040;
    pub const QUIT: i32 = 0x0080;
    pub const KICK: i32 = 0x0100;
    pub const KILL: i32 = 0x0200;
    pub const SERVER: i32 = 0x0400;
    pub const INFO: i32 = 0x0800;
    pub const ERROR: i32 = 0x1000;
    pub const DAYCHANGE: i32 = 0x2000;
}

/// A classified backlog row, ready to be rendered by a `Renderer`.
pub enum Event<'a> {
    Message {
        sender: &'a str,
        text: &'a str,
    },
    Notice {
        sender: &'a str,
        text: &'a str,
    },
    Action {
        sender: &'a str,
        text: &'a str,
    },
    NickChange {
        old: &'a str,
        new: &'a str,
    },
    ModeChange {
        sender: &'a str,
        mode: &'a str,
    },
    Join {
        nick: &'a str,
        host: &'a str,
        channel: &'a str,
    },
    Part {
        nick: &'a str,
        host: &'a str,
        channel: &'a str,
        reason: Option<&'a str>,
    },
    Quit {
        nick: &'a str,
        host: &'a str,
        reason: &'a str,
    },
    Kick {
        target: &'a str,
        by: &'a str,
        reason: Option<&'a str>,
    },
    Kill {
        target: &'a str,
        reason: &'a str,
    },
    TopicChange {
        user: &'a str,
        channel: &'a str,
        topic: &'a str,
    },
    TopicJoin1 {
        channel: &'a str,
        topic: &'a str,
    },
    TopicJoin2 {
        user: &'a str,
        logtime: &'a str,
    },
    Server {
        text: &'a str,
    },
    Info {
        text: &'a str,
    },
    Error {
        text: &'a str,
    },
    DayChange {
        text: &'a str,
    },
    Other,
}

static TOPIC_CHANGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(\S+) has changed topic for (\S+) to: "(.*)"$"#).unwrap());
static TOPIC_JOIN1_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"Topic for (\S+) is "(.*)"$"#).unwrap());
static TOPIC_JOIN2_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Topic set by (\S+) on (\S+ \S+ \d+ \S+)").unwrap());

/// Splits an IRC "nick!ident@host" sender into (nick, host). If there is no
/// '!', the whole string is treated as the nick (this happens for server
/// senders and for some self-authored events).
fn split_sender(sender: &str) -> (&str, &str) {
    match sender.split_once('!') {
        Some((nick, host)) => (nick, host),
        None => (sender, ""),
    }
}

fn classify_server(text: &str) -> Event<'_> {
    if let Some(caps) = TOPIC_CHANGE_RE.captures(text) {
        let (_, [who, channel, topic]) = caps.extract();
        return Event::TopicChange {
            user: who,
            channel,
            topic,
        };
    }
    if let Some(caps) = TOPIC_JOIN1_RE.captures(text) {
        let (_, [channel, topic]) = caps.extract();
        return Event::TopicJoin1 { channel, topic };
    }
    if let Some(caps) = TOPIC_JOIN2_RE.captures(text) {
        let (_, [who, when]) = caps.extract();
        return Event::TopicJoin2 {
            user: who,
            logtime: when,
        };
    }
    Event::Server { text }
}

/// Classifies a raw backlog row into a renderable `Event`, given the name of
/// the buffer it came from (used as a fallback channel name for join/part
/// events, whose message field is occasionally empty).
pub fn classify<'a>(row: &'a BacklogRow, buffer_name: &'a str) -> Event<'a> {
    let (nick, host) = split_sender(&row.sender);
    let msg = row.message.as_str();
    match row.msg_type {
        msg_type::PLAIN => Event::Message {
            sender: nick,
            text: msg,
        },
        msg_type::NOTICE => Event::Notice {
            sender: nick,
            text: msg,
        },
        msg_type::ACTION => Event::Action {
            sender: nick,
            text: msg,
        },
        msg_type::NICK => Event::NickChange {
            old: nick,
            new: msg,
        },
        msg_type::MODE => {
            let mode = match msg.split_once(' ') {
                Some((_target, rest)) => rest,
                None => msg,
            };
            Event::ModeChange { sender: nick, mode }
        }
        msg_type::JOIN => {
            let channel = if msg.is_empty() { buffer_name } else { msg };
            Event::Join {
                nick,
                host,
                channel,
            }
        }
        msg_type::PART => {
            let channel = if msg.is_empty() { buffer_name } else { msg };
            Event::Part {
                nick,
                host,
                channel,
                reason: None,
            }
        }
        msg_type::QUIT => Event::Quit {
            nick,
            host,
            reason: msg,
        },
        msg_type::KICK => {
            let (target, reason) = match msg.split_once(' ') {
                Some((target, reason)) if !reason.is_empty() => (target, Some(reason)),
                Some((target, _)) => (target, None),
                None => (msg, None),
            };
            Event::Kick {
                target,
                by: nick,
                reason,
            }
        }
        msg_type::KILL => {
            let (target, reason) = match msg.split_once(' ') {
                Some((target, reason)) => (target, reason),
                None => (msg, ""),
            };
            Event::Kill { target, reason }
        }
        msg_type::SERVER => classify_server(msg),
        msg_type::INFO => Event::Info { text: msg },
        msg_type::ERROR => Event::Error { text: msg },
        msg_type::DAYCHANGE => Event::DayChange { text: msg },
        _ => Event::Other,
    }
}

/// Renders classified backlog events into a specific output format.
pub trait Renderer {
    /// File extension (without the leading dot) used for this format.
    fn extension(&self) -> &'static str;
    /// Text written once at the start of a channel's output file.
    fn header(&self, network: &str, buffer: &str) -> String;
    /// Text written once at the end of a channel's output file.
    fn footer(&self) -> String;
    /// Renders a single event, including its trailing newline.
    fn render(&self, time: i64, event: &Event) -> String;
}
