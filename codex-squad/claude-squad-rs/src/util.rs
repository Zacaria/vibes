use std::path::Path;

use time::{format_description::well_known::Rfc3339, OffsetDateTime};

pub fn now() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

pub fn ensure_parent(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub fn format_time(time: OffsetDateTime) -> String {
    time.format(&Rfc3339)
        .unwrap_or_else(|_| "unknown".to_string())
}

#[allow(dead_code)]
pub fn redact(input: &str) -> String {
    if input.len() <= 4 {
        "****".into()
    } else {
        let last = &input[input.len() - 4..];
        format!("****{}", last)
    }
}
