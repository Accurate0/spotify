//! Relay global object identification: IDs are base64 of `<Type>:<key>`. Keys are database ids
//! for rows, the URI for items and the name for artists (exports have no artist URIs).

use std::fmt::Display;

use async_graphql::{ID, Interface};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

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
            Self::Account => "Account",
            Self::Export => "Export",
            Self::ExportFile => "ExportFile",
            Self::Item => "Item",
            Self::Artist => "Artist",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.as_str() == value)
    }
}

pub fn encode(ty: NodeType, key: impl Display) -> ID {
    ID(URL_SAFE_NO_PAD.encode(format!("{}:{key}", ty.as_str())))
}

pub fn decode(id: &ID) -> async_graphql::Result<(NodeType, String)> {
    let invalid = || async_graphql::Error::new(format!("invalid id {:?}", id.as_str()));

    let raw = URL_SAFE_NO_PAD
        .decode(id.as_bytes())
        .map_err(|_| invalid())?;
    let raw = String::from_utf8(raw).map_err(|_| invalid())?;
    let (ty, key) = raw.split_once(':').ok_or_else(invalid)?;

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
        assert_eq!(decode_row_id(&id, NodeType::ExportFile).unwrap(), 42);
        assert!(decode_as(&id, NodeType::Account).is_err());
        assert!(decode(&ID("nope".into())).is_err());

        // Keys may contain the separator.
        let id = encode(NodeType::Item, "spotify:track:abc");
        assert_eq!(
            decode(&id).unwrap(),
            (NodeType::Item, "spotify:track:abc".to_owned())
        );
    }
}
