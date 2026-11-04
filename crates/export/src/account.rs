//! Files from the "Account data" export. Contact details, identity and payment files are
//! classified as `FileKind::Private` and never parsed; `Userdata.json` is read for the
//! username only.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;

use crate::{ParseError, parse_iso, parse_minute};

/// `StreamingHistory<N>.json`: minute precision, no URIs.
#[derive(Debug, Clone, PartialEq)]
pub struct BasicPlay {
    pub end_time: DateTime<Utc>,
    pub artist: Option<String>,
    pub track: Option<String>,
    pub ms_played: i32,
}

pub fn parse_basic_history(bytes: &[u8]) -> Result<Vec<BasicPlay>, ParseError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Raw {
        end_time: String,
        artist_name: String,
        track_name: String,
        ms_played: i64,
    }

    let rows: Vec<Raw> = serde_json::from_slice(bytes)?;
    rows.into_iter()
        .enumerate()
        .map(|(row, raw)| {
            let end_time = parse_minute(&raw.end_time).ok_or_else(|| ParseError::Timestamp {
                row,
                value: raw.end_time.clone(),
            })?;
            Ok(BasicPlay {
                end_time,
                // The 2024 export uses these placeholders for ~0.5% of rows.
                artist: known(raw.artist_name, "Unknown Artist"),
                track: known(raw.track_name, "Unknown Track"),
                ms_played: clamp_ms(raw.ms_played),
            })
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryKind {
    Track,
    Album,
    Artist,
}

impl LibraryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Track => "track",
            Self::Album => "album",
            Self::Artist => "artist",
        }
    }
}

/// One saved item from `YourLibrary.json`. Shows, episodes and bans have always been empty in
/// observed exports and are ignored.
#[derive(Debug, Clone, PartialEq)]
pub struct LibraryItem {
    pub kind: LibraryKind,
    pub uri: Option<String>,
    pub name: String,
    pub artist: Option<String>,
    pub album: Option<String>,
}

pub fn parse_library(bytes: &[u8]) -> Result<Vec<LibraryItem>, ParseError> {
    #[derive(Deserialize)]
    struct Raw {
        #[serde(default)]
        tracks: Vec<RawTrack>,
        #[serde(default)]
        albums: Vec<RawAlbum>,
        #[serde(default)]
        artists: Vec<RawArtist>,
    }

    #[derive(Deserialize)]
    struct RawTrack {
        artist: Option<String>,
        album: Option<String>,
        track: String,
        uri: Option<String>,
    }

    #[derive(Deserialize)]
    struct RawAlbum {
        artist: Option<String>,
        album: String,
        uri: Option<String>,
    }

    #[derive(Deserialize)]
    struct RawArtist {
        name: String,
        uri: Option<String>,
    }

    let raw: Raw = serde_json::from_slice(bytes)?;
    let tracks = raw.tracks.into_iter().map(|t| LibraryItem {
        kind: LibraryKind::Track,
        uri: non_empty(t.uri),
        name: t.track,
        artist: non_empty(t.artist),
        album: non_empty(t.album),
    });
    let albums = raw.albums.into_iter().map(|a| LibraryItem {
        kind: LibraryKind::Album,
        uri: non_empty(a.uri),
        name: a.album,
        artist: non_empty(a.artist),
        album: None,
    });
    let artists = raw.artists.into_iter().map(|a| LibraryItem {
        kind: LibraryKind::Artist,
        uri: non_empty(a.uri),
        name: a.name,
        artist: None,
        album: None,
    });

    Ok(tracks.chain(albums).chain(artists).collect())
}

#[derive(Debug, Clone, PartialEq)]
pub struct Playlist {
    pub name: String,
    pub last_modified: Option<NaiveDate>,
    pub description: Option<String>,
    pub followers: i32,
    pub items: Vec<PlaylistItem>,
}

/// Playlist entries are tracks in every observed export; episodes and local tracks are kept
/// by name when present.
#[derive(Debug, Clone, PartialEq)]
pub struct PlaylistItem {
    pub uri: Option<String>,
    pub name: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    /// Only present in exports from 2024 onwards.
    pub added_date: Option<NaiveDate>,
}

pub fn parse_playlists(bytes: &[u8]) -> Result<Vec<Playlist>, ParseError> {
    #[derive(Deserialize)]
    struct Raw {
        playlists: Vec<RawPlaylist>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct RawPlaylist {
        name: String,
        last_modified_date: Option<String>,
        description: Option<String>,
        number_of_followers: Option<i64>,
        #[serde(default)]
        items: Vec<RawItem>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct RawItem {
        track: Option<RawTrack>,
        episode: Option<serde_json::Value>,
        local_track: Option<serde_json::Value>,
        added_date: Option<String>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct RawTrack {
        track_name: Option<String>,
        artist_name: Option<String>,
        album_name: Option<String>,
        track_uri: Option<String>,
    }

    fn field(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
        keys.iter()
            .find_map(|k| value.get(k)?.as_str())
            .map(str::to_owned)
    }

    let raw: Raw = serde_json::from_slice(bytes)?;
    Ok(raw
        .playlists
        .into_iter()
        .map(|p| Playlist {
            name: p.name,
            last_modified: p.last_modified_date.as_deref().and_then(parse_date),
            description: non_empty(p.description),
            followers: p.number_of_followers.unwrap_or(0).clamp(0, i32::MAX as i64) as i32,
            items: p
                .items
                .into_iter()
                .map(|item| {
                    let added_date = item.added_date.as_deref().and_then(parse_date);
                    if let Some(t) = item.track {
                        return PlaylistItem {
                            uri: non_empty(t.track_uri),
                            name: non_empty(t.track_name),
                            artist: non_empty(t.artist_name),
                            album: non_empty(t.album_name),
                            added_date,
                        };
                    }
                    let other = item.episode.or(item.local_track).unwrap_or_default();
                    PlaylistItem {
                        uri: field(&other, &["episodeUri", "uri"]),
                        name: field(&other, &["episodeName", "trackName", "name"]),
                        artist: field(&other, &["showName", "artistName"]),
                        album: field(&other, &["albumName"]),
                        added_date,
                    }
                })
                .collect(),
        })
        .collect())
}

#[derive(Debug, Clone, PartialEq)]
pub struct Search {
    pub ts: DateTime<Utc>,
    pub platform: Option<String>,
    pub query: String,
    pub interaction_uris: Vec<String>,
}

pub fn parse_searches(bytes: &[u8]) -> Result<Vec<Search>, ParseError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Raw {
        platform: Option<String>,
        search_time: String,
        search_query: String,
        #[serde(default, rename = "searchInteractionURIs")]
        search_interaction_uris: Vec<String>,
    }

    let rows: Vec<Raw> = serde_json::from_slice(bytes)?;
    rows.into_iter()
        .enumerate()
        .map(|(row, raw)| {
            let ts = parse_iso(&raw.search_time).ok_or_else(|| ParseError::Timestamp {
                row,
                value: raw.search_time.clone(),
            })?;
            Ok(Search {
                ts,
                platform: non_empty(raw.platform),
                query: raw.search_query,
                interaction_uris: raw.search_interaction_uris,
            })
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    pub follower_count: i32,
    pub following_users_count: i32,
    pub dismissing_users_count: i32,
}

pub fn parse_follow(bytes: &[u8]) -> Result<Follow, ParseError> {
    Ok(serde_json::from_slice(bytes)?)
}

/// Spotify's listener segment per artist, e.g. "Light listeners".
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Marquee {
    pub artist_name: String,
    pub segment: String,
}

pub fn parse_marquee(bytes: &[u8]) -> Result<Vec<Marquee>, ParseError> {
    Ok(serde_json::from_slice(bytes)?)
}

pub fn parse_inferences(bytes: &[u8]) -> Result<Vec<String>, ParseError> {
    #[derive(Deserialize)]
    struct Raw {
        #[serde(default)]
        inferences: Vec<String>,
    }

    Ok(serde_json::from_slice::<Raw>(bytes)?.inferences)
}

/// Only the username is read from `Userdata.json`; the rest is contact details.
pub fn parse_username(bytes: &[u8]) -> Result<Option<String>, ParseError> {
    #[derive(Deserialize)]
    struct Raw {
        username: Option<String>,
    }

    Ok(non_empty(serde_json::from_slice::<Raw>(bytes)?.username))
}

fn parse_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|v| !v.trim().is_empty())
}

fn known(value: String, placeholder: &str) -> Option<String> {
    (!value.is_empty() && value != placeholder).then_some(value)
}

fn clamp_ms(ms: i64) -> i32 {
    ms.clamp(0, i32::MAX as i64) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_history() {
        let plays = parse_basic_history(
            r#"[{"endTime":"2021-08-13 18:36","artistName":"Beach Bunny","trackName":"Good Girls (Don’t Get Used)","msPlayed":215402},
                {"endTime":"2023-02-01 00:00","artistName":"Unknown Artist","trackName":"Unknown Track","msPlayed":5}]"#
                .as_bytes(),
        )
        .unwrap();
        assert_eq!(plays[0].end_time.to_rfc3339(), "2021-08-13T18:36:00+00:00");
        assert_eq!(
            plays[0].track.as_deref(),
            Some("Good Girls (Don’t Get Used)")
        );
        assert_eq!(
            (plays[1].artist.as_deref(), plays[1].track.as_deref()),
            (None, None)
        );
    }

    #[test]
    fn library() {
        let items = parse_library(
            br#"{"tracks":[{"artist":"A","album":"B","track":"T","uri":"spotify:track:t"}],
                 "albums":[{"artist":"A","album":"B","uri":"spotify:album:b"}],
                 "artists":[{"name":"A","uri":"spotify:artist:a"}],
                 "shows":[],"episodes":[],"bannedTracks":[],"bannedArtists":[],"other":[]}"#,
        )
        .unwrap();
        let kinds: Vec<_> = items.iter().map(|i| i.kind).collect();
        assert_eq!(
            kinds,
            [LibraryKind::Track, LibraryKind::Album, LibraryKind::Artist]
        );
        assert_eq!(items[1].name, "B");
    }

    #[test]
    fn playlists_with_and_without_added_date() {
        let playlists = parse_playlists(
            br#"{"playlists":[{"name":"P","lastModifiedDate":"2023-12-01","description":null,"numberOfFollowers":2,
                 "items":[{"track":{"trackName":"T","artistName":"A","albumName":"B","trackUri":"spotify:track:t"},"episode":null,"localTrack":null,"addedDate":"2023-11-30"},
                          {"track":{"trackName":"U","artistName":"A","albumName":"B","trackUri":"spotify:track:u"},"episode":null,"localTrack":null}]}]}"#,
        )
        .unwrap();
        let p = &playlists[0];
        assert_eq!(p.last_modified, NaiveDate::from_ymd_opt(2023, 12, 1));
        assert_eq!(p.followers, 2);
        assert_eq!(p.items[0].added_date, NaiveDate::from_ymd_opt(2023, 11, 30));
        assert_eq!(p.items[1].added_date, None);
        assert_eq!(p.items[1].uri.as_deref(), Some("spotify:track:u"));
    }

    #[test]
    fn searches() {
        let searches = parse_searches(
            br#"[{"platform":"","searchTime":"2023-10-09T12:37:14.004Z[UTC]","searchQuery":"beach","searchInteractionURIs":["spotify:artist:a"]}]"#,
        )
        .unwrap();
        assert_eq!(searches[0].platform, None);
        assert_eq!(searches[0].interaction_uris, ["spotify:artist:a"]);
    }

    #[test]
    fn small_files() {
        let follow = parse_follow(
            br#"{"followerCount":13,"followingUsersCount":13,"dismissingUsersCount":0}"#,
        )
        .unwrap();
        assert_eq!(follow.follower_count, 13);

        let marquee =
            parse_marquee(br#"[{"artistName":"A","segment":"Light listeners"}]"#).unwrap();
        assert_eq!(marquee[0].segment, "Light listeners");

        let inferences =
            parse_inferences(br#"{"inferences":["1P_Custom_Auto_Mitsubishi"]}"#).unwrap();
        assert_eq!(inferences.len(), 1);

        let username = parse_username(
            br#"{"username":"abc","email":"x@example.com","birthdate":"2000-01-01"}"#,
        )
        .unwrap();
        assert_eq!(username.as_deref(), Some("abc"));
    }
}
