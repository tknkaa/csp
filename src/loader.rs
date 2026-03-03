use crate::session::Session;
use anyhow::{Context, Result};
use chrono::DateTime;
use std::{
    fs,
    io::{self, BufRead},
    path::PathBuf,
    time::SystemTime,
};

pub fn resolve_events(entry: &PathBuf) -> Option<(String, PathBuf)> {
    if entry.is_dir() {
        let id = entry.file_name()?.to_str()?.to_string();
        let p = entry.join("events.jsonl");
        if p.exists() {
            return Some((id, p));
        }
        let p = entry.join("events.json");
        if p.exists() {
            return Some((id, p));
        }
        None
    } else {
        let name = entry.file_name()?.to_str()?;
        let id = name
            .strip_suffix(".jsonl")
            .or_else(|| name.strip_suffix(".json"))?
            .to_string();
        Some((id, entry.clone()))
    }
}

pub fn parse_session(id: String, events: &PathBuf) -> Result<Session> {
    let modified = fs::metadata(events)?.modified()?;
    let reader = io::BufReader::new(fs::File::open(events)?);

    let mut start_time: Option<DateTime<chrono::Utc>> = None;
    let mut first_message: Option<String> = None;
    let mut message_count = 0usize;
    let mut cwd: Option<String> = None;

    for line in reader.lines().flatten() {
        if line.trim().is_empty() {
            continue;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };

        match v["type"].as_str().unwrap_or("") {
            "session.start" => {
                let ts = v["data"]["startTime"]
                    .as_str()
                    .or_else(|| v["timestamp"].as_str());
                if let Some(ts) = ts {
                    start_time = ts.parse().ok();
                }

                if let Some(c) = v["data"]["context"]["cwd"].as_str() {
                    cwd = Some(c.to_string());
                }
            }
            "user.message" => {
                message_count += 1;
                if first_message.is_none() {
                    if let Some(c) = v["data"]["content"].as_str() {
                        first_message = Some(c.to_string());
                    }
                }
            }
            _ => {}
        }
    }

    let start_time = start_time.unwrap_or_else(|| {
        modified
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()
            .and_then(|d| DateTime::from_timestamp(d.as_secs() as i64, 0))
            .unwrap_or_default()
    });

    Ok(Session {
        id,
        start_time,
        modified,
        first_message,
        message_count,
        cwd,
    })
}

pub fn load_sessions(dir: &PathBuf) -> Result<Vec<Session>> {
    let mut sessions = Vec::new();
    for entry in fs::read_dir(dir).context("cannot read session-state dir")? {
        let path = entry?.path();
        if let Some((id, events)) = resolve_events(&path) {
            if let Ok(s) = parse_session(id, &events) {
                sessions.push(s);
            }
        }
    }
    sessions.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(sessions)
}
