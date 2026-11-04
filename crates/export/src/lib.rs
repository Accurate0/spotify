//! Parsing for the files found in Spotify "Account data" and "Extended streaming history"
//! exports. Anything personally identifying (IPs, user agents, contact details) is never
//! deserialized.

mod extended;
mod kind;

pub use extended::{ExtendedPlay, ItemKind, parse_extended};
pub use kind::FileKind;

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("invalid json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("row {row}: invalid timestamp {value:?}")]
    Timestamp { row: usize, value: String },
}

/// Newest timestamp in a file, used to order exports by recency. Only streaming history and
/// search files carry timestamps; other kinds return `None`.
pub fn max_timestamp(kind: FileKind, bytes: &[u8]) -> Result<Option<DateTime<Utc>>, ParseError> {
    #[derive(Deserialize)]
    struct Ts {
        ts: Option<String>,
        #[serde(rename = "endTime")]
        end_time: Option<String>,
        #[serde(rename = "searchTime")]
        search_time: Option<String>,
    }

    if !matches!(
        kind,
        FileKind::ExtendedAudio
            | FileKind::ExtendedVideo
            | FileKind::AccountStreamingHistory
            | FileKind::SearchQueries
    ) {
        return Ok(None);
    }

    let rows: Vec<Ts> = serde_json::from_slice(bytes)?;
    let max = rows
        .iter()
        .filter_map(|r| {
            r.ts.as_deref()
                .and_then(parse_iso)
                .or_else(|| r.end_time.as_deref().and_then(parse_minute))
                .or_else(|| r.search_time.as_deref().and_then(parse_iso))
        })
        .max();

    Ok(max)
}

/// `2024-04-27T15:19:56Z`, or search history's `2023-10-09T12:37:14.004Z[UTC]`.
pub fn parse_iso(value: &str) -> Option<DateTime<Utc>> {
    let value = value.strip_suffix("[UTC]").unwrap_or(value);
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|d| d.with_timezone(&Utc))
}

/// Account data's `2021-08-13 18:36`, which is UTC.
pub fn parse_minute(value: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M")
        .ok()
        .map(|d| d.and_utc())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_timestamp_formats() {
        assert_eq!(
            parse_iso("2023-10-09T12:37:14.004Z[UTC]")
                .unwrap()
                .to_rfc3339(),
            "2023-10-09T12:37:14.004+00:00"
        );
        assert_eq!(
            parse_minute("2021-08-13 18:36").unwrap().to_rfc3339(),
            "2021-08-13T18:36:00+00:00"
        );
    }

    #[test]
    fn max_timestamp_by_kind() {
        let history = br#"[{"endTime":"2021-08-13 18:36","msPlayed":1},{"endTime":"2021-08-14 01:00","msPlayed":2}]"#;
        let max = max_timestamp(FileKind::AccountStreamingHistory, history).unwrap();
        assert_eq!(max.unwrap().to_rfc3339(), "2021-08-14T01:00:00+00:00");
        assert_eq!(max_timestamp(FileKind::Library, b"{}").unwrap(), None);
    }
}
