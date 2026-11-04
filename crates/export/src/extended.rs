use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{ParseError, parse_iso};

/// One row of `Streaming_History_{Audio,Video}_*.json`. Three field sets exist across exports
/// (with or without `username`/`user_agent_decrypted`/audiobook fields), so everything other
/// than `ts` and `ms_played` is optional. IP and user agent fields are intentionally absent.
#[derive(Debug, Deserialize)]
struct RawRow {
    ts: String,
    platform: Option<String>,
    ms_played: i64,
    conn_country: Option<String>,
    master_metadata_track_name: Option<String>,
    master_metadata_album_artist_name: Option<String>,
    master_metadata_album_album_name: Option<String>,
    spotify_track_uri: Option<String>,
    episode_name: Option<String>,
    episode_show_name: Option<String>,
    spotify_episode_uri: Option<String>,
    audiobook_title: Option<String>,
    audiobook_chapter_uri: Option<String>,
    audiobook_chapter_title: Option<String>,
    reason_start: Option<String>,
    reason_end: Option<String>,
    shuffle: Option<bool>,
    skipped: Option<bool>,
    offline: Option<bool>,
    offline_timestamp: Option<serde_json::Number>,
    incognito_mode: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Track,
    Episode,
    Audiobook,
}

impl ItemKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Track => "track",
            Self::Episode => "episode",
            Self::Audiobook => "audiobook",
        }
    }
}

/// A normalized play. `name`/`artist`/`album` describe the item: for episodes `artist` is the
/// show, for audiobooks it is the book title and `name` the chapter.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExtendedPlay {
    pub ts: DateTime<Utc>,
    pub uri: Option<String>,
    pub kind: Option<ItemKind>,
    pub name: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub ms_played: i32,
    pub platform: Option<String>,
    pub conn_country: Option<String>,
    pub reason_start: Option<String>,
    pub reason_end: Option<String>,
    pub shuffle: Option<bool>,
    pub skipped: Option<bool>,
    pub offline: Option<bool>,
    pub offline_ts: Option<DateTime<Utc>>,
    pub incognito: Option<bool>,
    pub is_video: bool,
}

pub fn parse_extended(bytes: &[u8], is_video: bool) -> Result<Vec<ExtendedPlay>, ParseError> {
    let rows: Vec<RawRow> = serde_json::from_slice(bytes)?;

    rows.into_iter()
        .enumerate()
        .map(|(row, raw)| match parse_iso(&raw.ts) {
            Some(ts) => Ok(normalize(ts, raw, is_video)),
            None => Err(ParseError::Timestamp { row, value: raw.ts }),
        })
        .collect()
}

fn normalize(ts: DateTime<Utc>, raw: RawRow, is_video: bool) -> ExtendedPlay {
    let (uri, kind, name, artist, album) = if let Some(uri) = non_empty(raw.spotify_track_uri) {
        (
            Some(uri),
            Some(ItemKind::Track),
            raw.master_metadata_track_name,
            raw.master_metadata_album_artist_name,
            raw.master_metadata_album_album_name,
        )
    } else if let Some(uri) = non_empty(raw.spotify_episode_uri) {
        (
            Some(uri),
            Some(ItemKind::Episode),
            raw.episode_name,
            raw.episode_show_name,
            None,
        )
    } else if let Some(uri) = non_empty(raw.audiobook_chapter_uri) {
        (
            Some(uri),
            Some(ItemKind::Audiobook),
            raw.audiobook_chapter_title,
            raw.audiobook_title,
            None,
        )
    } else {
        (
            None,
            None,
            raw.master_metadata_track_name,
            raw.master_metadata_album_artist_name,
            raw.master_metadata_album_album_name,
        )
    };

    ExtendedPlay {
        ts,
        uri,
        kind,
        name: trimmed(name),
        artist: trimmed(artist),
        album: trimmed(album),
        ms_played: raw.ms_played.clamp(0, i32::MAX as i64) as i32,
        platform: non_empty(raw.platform),
        conn_country: non_empty(raw.conn_country),
        reason_start: non_empty(raw.reason_start),
        reason_end: non_empty(raw.reason_end),
        shuffle: raw.shuffle,
        skipped: raw.skipped,
        offline: raw.offline,
        offline_ts: raw.offline_timestamp.as_ref().and_then(offline_ts),
        incognito: raw.incognito_mode,
        is_video,
    }
}

/// `offline_timestamp` is 0 for "none", and otherwise epoch seconds (10 digits) or epoch
/// milliseconds (13 digits), sometimes mixed within one file.
fn offline_ts(value: &serde_json::Number) -> Option<DateTime<Utc>> {
    let n = value
        .as_i64()
        .or_else(|| value.as_f64().map(|f| f as i64))?;

    match n {
        n if n <= 0 => None,
        n if n >= 100_000_000_000 => DateTime::from_timestamp_millis(n),
        n => DateTime::from_timestamp(n, 0),
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|v| !v.is_empty())
}

fn trimmed(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Field sets from the 2024-01 export (username, ip_addr_decrypted, user_agent_decrypted),
    // the 2024-12 export (ip_addr) and the 2025 export (audiobook fields).
    const JAN_2024: &str = r#"[{"ts":"2023-05-01T10:00:00Z","username":"u","platform":"Android OS 11 API 30 (Google, Pixel 5)","ms_played":1000,"conn_country":"AU","ip_addr_decrypted":"1.2.3.4","user_agent_decrypted":"unknown","master_metadata_track_name":"Timber (feat. Ke$ha)","master_metadata_album_artist_name":"Pitbull","master_metadata_album_album_name":"Global Warming","spotify_track_uri":"spotify:track:a","episode_name":null,"episode_show_name":null,"spotify_episode_uri":null,"reason_start":"","reason_end":null,"shuffle":false,"skipped":null,"offline":false,"offline_timestamp":0,"incognito_mode":false}]"#;
    const DEC_2024: &str = r#"[{"ts":"2023-05-01T10:00:01Z","platform":"android","ms_played":1000,"conn_country":"AU","ip_addr":"::1","master_metadata_track_name":null,"master_metadata_album_artist_name":null,"master_metadata_album_album_name":null,"spotify_track_uri":null,"episode_name":"  Ep 1 ","episode_show_name":"Show","spotify_episode_uri":"spotify:episode:b","reason_start":"fwdbtn","reason_end":"trackdone","shuffle":true,"skipped":false,"offline":true,"offline_timestamp":1734444739,"incognito_mode":false}]"#;
    const FEB_2025: &str = r#"[{"ts":"2024-01-01T00:00:00Z","platform":"ios","ms_played":5,"conn_country":"AU","ip_addr":"x","master_metadata_track_name":"T","master_metadata_album_artist_name":"A","master_metadata_album_album_name":"B","spotify_track_uri":"spotify:track:c","episode_name":null,"episode_show_name":null,"spotify_episode_uri":null,"audiobook_title":null,"audiobook_uri":null,"audiobook_chapter_uri":null,"audiobook_chapter_title":null,"reason_start":"clickrow","reason_end":"endplay","shuffle":false,"skipped":true,"offline":false,"offline_timestamp":1704067200000,"incognito_mode":false}]"#;

    #[test]
    fn parses_all_field_sets() {
        let jan = &parse_extended(JAN_2024.as_bytes(), false).unwrap()[0];
        assert_eq!(jan.uri.as_deref(), Some("spotify:track:a"));
        assert_eq!(jan.kind, Some(ItemKind::Track));
        assert_eq!(jan.reason_start, None);
        assert_eq!(jan.offline_ts, None);
        assert_eq!(jan.skipped, None);

        let dec = &parse_extended(DEC_2024.as_bytes(), true).unwrap()[0];
        assert_eq!(dec.kind, Some(ItemKind::Episode));
        assert_eq!(dec.name.as_deref(), Some("Ep 1"));
        assert_eq!(dec.artist.as_deref(), Some("Show"));
        assert_eq!(dec.offline_ts.unwrap().timestamp(), 1734444739);
        assert!(dec.is_video);

        let feb = &parse_extended(FEB_2025.as_bytes(), false).unwrap()[0];
        assert_eq!(feb.offline_ts.unwrap().timestamp(), 1704067200);
    }

    #[test]
    fn reports_bad_timestamp() {
        let err = parse_extended(br#"[{"ts":"nope","ms_played":1}]"#, false).unwrap_err();
        assert!(matches!(err, ParseError::Timestamp { row: 0, ref value } if value == "nope"));
    }
}
