//! Forward-only, offset-based Relay connections. Cursors are row offsets; stats lists are
//! recomputed per request, so keyset cursors would buy nothing.

use async_graphql::OutputType;
use async_graphql::connection::{Connection, ConnectionNameType, Edge, EdgeNameType, EmptyFields};
use async_graphql::{ObjectType, SimpleObject};

const MAX_PAGE: i32 = 100;

pub struct Page {
    pub offset: i64,
    limit: i64,
}

impl Page {
    pub fn new(
        first: Option<i32>,
        after: Option<String>,
        default: i32,
    ) -> async_graphql::Result<Self> {
        let offset = match after {
            Some(cursor) => {
                cursor
                    .parse::<i64>()
                    .map_err(|_| async_graphql::Error::new(format!("invalid cursor {cursor:?}")))?
                    + 1
            }
            None => 0,
        };
        let limit = first.unwrap_or(default).clamp(1, MAX_PAGE) as i64;
        Ok(Self { offset, limit })
    }

    /// Rows to fetch: one extra to learn whether there is a next page.
    pub fn fetch(&self) -> i64 {
        self.limit + 1
    }

    pub fn connection<N, E, C, EN>(
        &self,
        rows: impl IntoIterator<Item = (N, E)>,
    ) -> Connection<usize, N, EmptyFields, E, C, EN>
    where
        N: OutputType,
        E: ObjectType,
        C: ConnectionNameType,
        EN: EdgeNameType,
    {
        let mut rows: Vec<_> = rows.into_iter().collect();
        let has_next = rows.len() as i64 > self.limit;
        rows.truncate(self.limit as usize);

        let mut connection = Connection::new(self.offset > 0, has_next);
        connection
            .edges
            .extend(rows.into_iter().enumerate().map(|(i, (node, fields))| {
                Edge::with_additional_fields(self.offset as usize + i, node, fields)
            }));
        connection
    }
}

/// Edge fields for ranked lists (top artists, tracks, albums).
#[derive(SimpleObject)]
pub struct Ranked {
    /// 1-based position in the ranking.
    pub rank: i32,
    pub plays: i32,
    pub minutes_played: i32,
}

pub struct RankedConnectionName;

impl ConnectionNameType for RankedConnectionName {
    fn type_name<T: OutputType>() -> String {
        format!("Ranked{}Connection", T::type_name())
    }
}

pub struct RankedEdgeName;

impl EdgeNameType for RankedEdgeName {
    fn type_name<T: OutputType>() -> String {
        format!("Ranked{}Edge", T::type_name())
    }
}

pub type RankedConnection<N> =
    Connection<usize, N, EmptyFields, Ranked, RankedConnectionName, RankedEdgeName>;
pub type PlainConnection<N> = Connection<usize, N>;

impl Page {
    pub fn ranked<N: OutputType>(
        &self,
        rows: impl IntoIterator<Item = (N, i32, i32)>,
    ) -> RankedConnection<N> {
        let offset = self.offset as i32;
        self.connection(
            rows.into_iter()
                .enumerate()
                .map(|(i, (node, plays, minutes))| {
                    (
                        node,
                        Ranked {
                            rank: offset + i as i32 + 1,
                            plays,
                            minutes_played: minutes,
                        },
                    )
                }),
        )
    }

    pub fn plain<N: OutputType>(&self, rows: impl IntoIterator<Item = N>) -> PlainConnection<N> {
        self.connection(rows.into_iter().map(|n| (n, EmptyFields)))
    }
}
