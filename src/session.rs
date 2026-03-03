use chrono::{DateTime, Local, Utc};
use std::time::SystemTime;

pub struct Session {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub modified: SystemTime,
    pub first_message: Option<String>,
    pub message_count: usize,
    pub cwd: Option<String>,
}

impl Session {
    pub fn relative_time(&self) -> String {
        let local: DateTime<Local> = self.start_time.into();
        let now = Local::now();
        let diff = now.signed_duration_since(local);
        if diff.num_seconds() < 60 {
            "just now".into()
        } else if diff.num_minutes() < 60 {
            format!("{}m ago", diff.num_minutes())
        } else if diff.num_hours() < 24 {
            format!("{}h ago", diff.num_hours())
        } else if diff.num_days() < 7 {
            format!("{}d ago", diff.num_days())
        } else {
            local.format("%Y-%m-%d").to_string()
        }
    }

    pub fn display_time(&self) -> String {
        let local: DateTime<Local> = self.start_time.into();
        local.format("%m/%d %H:%M").to_string()
    }

    pub fn preview(&self) -> &str {
        self.first_message.as_deref().unwrap_or("(no messages)")
    }
}
