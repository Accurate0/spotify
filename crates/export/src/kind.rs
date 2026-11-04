use serde::{Deserialize, Serialize};

/// Every file kind found across the export formats. Kinds that are recognized but never
/// imported (PII, technical logs) are still classified so the CLI can report them as skipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FileKind {
    /// `Streaming_History_Audio_*.json`
    ExtendedAudio,
    /// `Streaming_History_Video_*.json`
    ExtendedVideo,
    /// `StreamingHistory<N>.json` from account data
    AccountStreamingHistory,
    Library,
    Playlist,
    SearchQueries,
    Follow,
    Marquee,
    Inferences,
    Userdata,
    /// Contains personal data that is deliberately not imported.
    Private,
}

impl FileKind {
    pub fn classify(file_name: &str) -> Option<Self> {
        let stem = file_name.strip_suffix(".json")?;

        let kind = if stem.starts_with("Streaming_History_Audio_") {
            Self::ExtendedAudio
        } else if stem.starts_with("Streaming_History_Video_") {
            Self::ExtendedVideo
        } else if let Some(n) = stem.strip_prefix("StreamingHistory") {
            numbered(n, Self::AccountStreamingHistory)?
        } else if let Some(n) = stem.strip_prefix("Playlist") {
            numbered(n, Self::Playlist)?
        } else {
            match stem {
                "YourLibrary" => Self::Library,
                "SearchQueries" => Self::SearchQueries,
                "Follow" => Self::Follow,
                "Marquee" => Self::Marquee,
                "Inferences" => Self::Inferences,
                "Userdata" => Self::Userdata,
                "Identity" | "Identifiers" | "Payments" | "DuoNewFamily" => Self::Private,
                _ => return None,
            }
        };

        Some(kind)
    }

    pub const ALL: [Self; 11] = [
        Self::ExtendedAudio,
        Self::ExtendedVideo,
        Self::AccountStreamingHistory,
        Self::Library,
        Self::Playlist,
        Self::SearchQueries,
        Self::Follow,
        Self::Marquee,
        Self::Inferences,
        Self::Userdata,
        Self::Private,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExtendedAudio => "EXTENDED_AUDIO",
            Self::ExtendedVideo => "EXTENDED_VIDEO",
            Self::AccountStreamingHistory => "ACCOUNT_STREAMING_HISTORY",
            Self::Library => "LIBRARY",
            Self::Playlist => "PLAYLIST",
            Self::SearchQueries => "SEARCH_QUERIES",
            Self::Follow => "FOLLOW",
            Self::Marquee => "MARQUEE",
            Self::Inferences => "INFERENCES",
            Self::Userdata => "USERDATA",
            Self::Private => "PRIVATE",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.as_str() == value)
    }

    /// Whether the server can import this kind yet.
    pub fn is_importable(self) -> bool {
        matches!(self, Self::ExtendedAudio | Self::ExtendedVideo)
    }
}

fn numbered(suffix: &str, kind: FileKind) -> Option<FileKind> {
    (!suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_digit())).then_some(kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_export_files() {
        let cases = [
            (
                "Streaming_History_Audio_2014-2016_0.json",
                Some(FileKind::ExtendedAudio),
            ),
            (
                "Streaming_History_Video_2018-2024.json",
                Some(FileKind::ExtendedVideo),
            ),
            (
                "StreamingHistory6.json",
                Some(FileKind::AccountStreamingHistory),
            ),
            ("Playlist1.json", Some(FileKind::Playlist)),
            ("YourLibrary.json", Some(FileKind::Library)),
            ("DuoNewFamily.json", Some(FileKind::Private)),
            ("StreamingHistory.json", None),
            ("ReadMeFirst_ExtendedStreamingHistory.pdf", None),
            ("spotify.py", None),
        ];

        for (name, expected) in cases {
            assert_eq!(FileKind::classify(name), expected, "{name}");
        }
    }
}
