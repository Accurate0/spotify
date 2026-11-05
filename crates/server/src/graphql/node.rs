//! Relay global object identification: IDs are `spotify:<node>:<key>`, where `<node>` is the
//! GraphQL type name without the `Spotify` prefix, in lowerCamelCase. The graph router's node
//! plugin turns that into `__typename`. Keys are database ids for rows, the URI for items and the
//! name for artists (exports have no artist URIs).

use std::fmt::Display;

use async_graphql::{ID, Interface};

use super::catalog::{Artist, Item};
use super::objects::{Account, Export, ExportFile};

#[derive(Interface)]
#[graphql(field(name = "id", ty = "ID"))]
pub enum Node {
    Account(Account),
    Export(Export),
    ExportFile(ExportFile),
    Item(Item),
    Artist(Artist),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Account,
    Export,
    ExportFile,
    Item,
    Artist,
}

impl NodeType {
    const ALL: [Self; 5] = [
        Self::Account,
        Self::Export,
        Self::ExportFile,
        Self::Item,
        Self::Artist,
    ];

    fn as_str(self) -> &'static str {
        match self {
            Self::Account => "account",
            Self::Export => "export",
            Self::ExportFile => "exportFile",
            Self::Item => "item",
            Self::Artist => "artist",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.as_str() == value)
    }
}

const SERVICE: &str = "spotify";

pub fn encode(ty: NodeType, key: impl Display) -> ID {
    ID(format!("{SERVICE}:{}:{key}", ty.as_str()))
}

pub fn decode(id: &ID) -> async_graphql::Result<(NodeType, String)> {
    let invalid = || async_graphql::Error::new(format!("invalid id {:?}", id.as_str()));

    let mut parts = id.splitn(3, ':');
    let (Some(SERVICE), Some(ty), Some(key)) = (parts.next(), parts.next(), parts.next()) else {
        return Err(invalid());
    };

    Ok((NodeType::parse(ty).ok_or_else(invalid)?, key.to_owned()))
}

/// Decodes an id that must be of a specific type, e.g. a mutation's `exportId`.
pub fn decode_as(id: &ID, expected: NodeType) -> async_graphql::Result<String> {
    match decode(id)? {
        (ty, key) if ty == expected => Ok(key),
        _ => Err(async_graphql::Error::new(format!(
            "expected a {} id",
            expected.as_str()
        ))),
    }
}

/// Like `decode_as` for types keyed by a database id.
pub fn decode_row_id(id: &ID, expected: NodeType) -> async_graphql::Result<i64> {
    decode_as(id, expected)?
        .parse()
        .map_err(|_| async_graphql::Error::new(format!("invalid id {:?}", id.as_str())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let id = encode(NodeType::ExportFile, 42);
        assert_eq!(id.as_str(), "spotify:exportFile:42");
        assert_eq!(decode_row_id(&id, NodeType::ExportFile).unwrap(), 42);
        assert!(decode(&ID("home:exportFile:42".into())).is_err());
        assert!(decode_as(&id, NodeType::Account).is_err());
        assert!(decode(&ID("nope".into())).is_err());

        // Keys may contain the separator.
        let id = encode(NodeType::Item, "spotify:track:abc");
        assert_eq!(id.as_str(), "spotify:item:spotify:track:abc");
        assert_eq!(
            decode(&id).unwrap(),
            (NodeType::Item, "spotify:track:abc".to_owned())
        );
    }
}
