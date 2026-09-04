use chrono::{Local, TimeZone};

use super::{Event, Renderer};

pub struct HtmlRenderer;

fn timestamp(time: i64) -> String {
    match Local.timestamp_opt(time, 0).single() {
        Some(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        None => "????-??-?? ??:??:??".to_string(),
    }
}

/// Escapes text for safe inclusion in HTML, dropping raw control characters
/// (such as mIRC color codes) that would otherwise render as garbage.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c if (c as u32) < 0x20 && c != '\t' => {}
            c => out.push(c),
        }
    }
    out
}

fn line(class: &str, time: i64, body: String) -> String {
    format!(
        "<div class=\"line {class}\"><span class=\"time\">{}</span> {body}</div>\n",
        escape(&timestamp(time))
    )
}

impl Renderer for HtmlRenderer {
    fn extension(&self) -> &'static str {
        "html"
    }

    fn header(&self, network: &str, buffer: &str) -> String {
        let title = format!("{} on {}", escape(buffer), escape(network));
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>{title}</title>
<style>
  :root {{ color-scheme: light dark; }}
  body {{
    background: canvas;
    color: canvastext;
    font-family: ui-monospace, Menlo, Consolas, monospace;
    font-size: 13px;
    margin: 0;
    padding: 1em;
  }}
  h1 {{ font-size: 1.1em; font-weight: 600; margin: 0 0 0.75em; }}
  .line {{ white-space: pre-wrap; word-wrap: break-word; padding: 1px 0; }}
  .time {{ color: #888; margin-right: 0.5em; }}
  .sender {{ font-weight: 600; }}
  .message .sender {{ color: #2a6fc9; }}
  .notice .sender, .notice {{ color: #a6631a; }}
  .action {{ color: #7a3ec9; font-style: italic; }}
  .join, .part, .quit, .kick, .kill, .nick, .mode {{ color: #4a9c4a; }}
  .topic {{ color: #4a9c4a; font-weight: 600; }}
  .server, .info, .daychange {{ color: #888; }}
  .error {{ color: #c93a3a; }}
</style>
</head>
<body>
<h1>{title}</h1>
"#
        )
    }

    fn footer(&self) -> String {
        "</body>\n</html>\n".to_string()
    }

    fn render(&self, time: i64, event: &Event) -> String {
        match event {
            Event::Message { sender, text } => line(
                "message",
                time,
                format!(
                    "&lt;<span class=\"sender\">{}</span>&gt; {}",
                    escape(sender),
                    escape(text)
                ),
            ),
            Event::Notice { sender, text } => line(
                "notice",
                time,
                format!(
                    "-<span class=\"sender\">{}</span>- {}",
                    escape(sender),
                    escape(text)
                ),
            ),
            Event::Action { sender, text } => line(
                "action",
                time,
                format!("* {} {}", escape(sender), escape(text)),
            ),
            Event::NickChange { old, new } => line(
                "nick",
                time,
                format!("*** {} is now known as {}", escape(old), escape(new)),
            ),
            Event::ModeChange { sender, mode } => line(
                "mode",
                time,
                format!("*** {} sets mode: {}", escape(sender), escape(mode)),
            ),
            Event::Join { nick, host, channel } => line(
                "join",
                time,
                format!(
                    "*** {} ({}) has joined {}",
                    escape(nick),
                    escape(host),
                    escape(channel)
                ),
            ),
            Event::Part {
                nick,
                host,
                channel,
                reason,
            } => {
                let body = match reason {
                    Some(reason) => format!(
                        "*** {} ({}) has left {} ({})",
                        escape(nick),
                        escape(host),
                        escape(channel),
                        escape(reason)
                    ),
                    None => format!(
                        "*** {} ({}) has left {}",
                        escape(nick),
                        escape(host),
                        escape(channel)
                    ),
                };
                line("part", time, body)
            }
            Event::Quit { nick, host, reason } => line(
                "quit",
                time,
                format!(
                    "*** {} ({}) has quit ({})",
                    escape(nick),
                    escape(host),
                    escape(reason)
                ),
            ),
            Event::Kick { target, by, reason } => {
                let body = match reason {
                    Some(reason) => format!(
                        "*** {} was kicked by {} ({})",
                        escape(target),
                        escape(by),
                        escape(reason)
                    ),
                    None => format!("*** {} was kicked by {}", escape(target), escape(by)),
                };
                line("kick", time, body)
            }
            Event::Kill { target, reason } => line(
                "kill",
                time,
                format!("*** {} was killed ({})", escape(target), escape(reason)),
            ),
            Event::TopicChange { user, channel, topic } => line(
                "topic",
                time,
                format!(
                    "*** {} changes topic for {} to '{}'",
                    escape(user),
                    escape(channel),
                    escape(topic)
                ),
            ),
            Event::TopicJoin1 { channel, topic } => line(
                "topic",
                time,
                format!("*** Topic for {} is '{}'", escape(channel), escape(topic)),
            ),
            Event::TopicJoin2 { user, logtime } => line(
                "topic",
                time,
                format!("*** Topic set by {} on {}", escape(user), escape(logtime)),
            ),
            Event::Server { text } => line("server", time, format!("*** {}", escape(text))),
            Event::Info { text } => line("info", time, format!("*** {}", escape(text))),
            Event::Error { text } => line("error", time, format!("*** Error: {}", escape(text))),
            Event::DayChange { text } => {
                let body = if text.is_empty() {
                    "--- Day changed ---".to_string()
                } else {
                    format!("--- Day changed ({}) ---", escape(text))
                };
                line("daychange", time, body)
            }
            Event::Other => String::new(),
        }
    }
}
