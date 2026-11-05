use async_graphql::dataloader::DataLoader;
use async_graphql::{EmptySubscription, SDLExportOptions, Schema, SchemaBuilder};
use sqlx::PgPool;

/// Runs a query over `scoped_plays($1..$7)`, binding a `Scope`'s seven filter parameters so each
/// query lists only its own, starting at `$8`. `query_as!(T)` maps rows into `T`.
macro_rules! scoped {
    (query_as!($ty:ident), $scope:expr, $sql:literal $(, $arg:expr)* $(,)?) => {
        sqlx::query_as!(
            $ty,
            $sql,
            &$scope.accounts,
            $scope.from,
            $scope.to,
            &$scope.kinds,
            $scope.min_ms,
            $scope.item_uri.as_deref(),
            $scope.artist.as_deref()
            $(, $arg)*
        )
    };
    (query!, $scope:expr, $sql:literal $(, $arg:expr)* $(,)?) => {
        sqlx::query!(
            $sql,
            &$scope.accounts,
            $scope.from,
            $scope.to,
            &$scope.kinds,
            $scope.min_ms,
            $scope.item_uri.as_deref(),
            $scope.artist.as_deref()
            $(, $arg)*
        )
    };
}

mod catalog;
mod insights;
mod loaders;
mod mutation;
pub mod node;
pub mod objects;
mod page;
mod query;
mod stats;
mod v2;

pub use mutation::MutationRoot;
pub use query::QueryRoot;
pub use v2::{MutationRootV2, QueryRootV2};

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;
pub type AppSchemaV2 = Schema<QueryRootV2, MutationRootV2, EmptySubscription>;

pub fn builder() -> SchemaBuilder<QueryRoot, MutationRoot, EmptySubscription> {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .enable_federation()
        .extension(crate::telemetry::graphql::Tracing)
}

/// The supergraph's schema: every root field of v1 prefixed with `spotify`.
pub fn builder_v2() -> SchemaBuilder<QueryRootV2, MutationRootV2, EmptySubscription> {
    Schema::build(QueryRootV2, MutationRootV2, EmptySubscription)
        .enable_federation()
        .extension(crate::telemetry::graphql::Tracing)
}

pub fn schema(pool: PgPool) -> AppSchema {
    with_data(builder(), pool)
}

pub fn schema_v2(pool: PgPool) -> AppSchemaV2 {
    with_data(builder_v2(), pool)
}

/// Attaches the pool and batch loaders (uncached, so no stale reads).
fn with_data<Q, M>(
    builder: SchemaBuilder<Q, M, EmptySubscription>,
    pool: PgPool,
) -> Schema<Q, M, EmptySubscription>
where
    Q: async_graphql::ObjectType + 'static,
    M: async_graphql::ObjectType + 'static,
{
    builder
        .data(DataLoader::new(
            loaders::AccountLoader(pool.clone()),
            tokio::spawn,
        ))
        .data(DataLoader::new(
            loaders::ItemLoader(pool.clone()),
            tokio::spawn,
        ))
        .data(DataLoader::new(
            loaders::ItemMetaLoader(pool.clone()),
            tokio::spawn,
        ))
        .data(DataLoader::new(
            loaders::ItemArtistsLoader(pool.clone()),
            tokio::spawn,
        ))
        .data(DataLoader::new(
            loaders::ArtistMetaLoader(pool.clone()),
            tokio::spawn,
        ))
        .data(pool)
        .finish()
}

pub fn sdl() -> String {
    builder().finish().sdl()
}

/// The federation SDL the `graph` router composes from v2, with `@key`s and `@link`.
pub fn subgraph_sdl() -> String {
    builder_v2()
        .finish()
        .sdl_with_options(SDLExportOptions::new().federation())
}

#[cfg(test)]
mod tests {
    /// Keeps the committed `schema.graphql` (read by the CLI's cynic codegen and the web app's
    /// relay-compiler) in sync. Rewrites it on drift and fails so the change gets committed.
    #[test]
    fn schema_is_current() {
        let sdl = super::sdl();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schema.graphql");
        let current = std::fs::read_to_string(&path).unwrap_or_default();

        if current != sdl {
            std::fs::write(&path, &sdl).expect("write schema.graphql");
            panic!("schema.graphql was out of date and has been regenerated, commit it");
        }
    }

    /// Every v1 root field is in v2 as `spotify<Field>`, and v2 has nothing else.
    #[test]
    fn v2_prefixes_every_v1_root_field() {
        let roots = |sdl: String| {
            let doc = async_graphql::parser::parse_schema(sdl).expect("parse sdl");
            let mut fields = Vec::new();
            for definition in doc.definitions {
                let async_graphql::parser::types::TypeSystemDefinition::Type(ty) = definition
                else {
                    continue;
                };
                let name = ty.node.name.node.to_string();
                if !matches!(name.as_str(), "Query" | "Mutation") {
                    continue;
                }
                if let async_graphql::parser::types::TypeKind::Object(object) = ty.node.kind {
                    for field in object.fields {
                        let field = field.node.name.node.to_string();
                        if !field.starts_with('_') {
                            fields.push(format!("{name}.{field}"));
                        }
                    }
                }
            }
            fields.sort();
            fields
        };

        let mut prefixed: Vec<String> = roots(super::sdl())
            .into_iter()
            .map(|path| {
                let (ty, field) = path.split_once('.').unwrap();
                let mut chars = field.chars();
                let first = chars.next().unwrap().to_ascii_uppercase();
                format!("{ty}.spotify{first}{}", chars.as_str())
            })
            .collect();
        prefixed.sort();

        assert_eq!(roots(super::builder_v2().finish().sdl()), prefixed);
    }

    /// Keeps the committed `subgraph.graphql`, which `ortzi app supergraph` fetches, in sync.
    #[test]
    fn subgraph_is_current() {
        let sdl = super::subgraph_sdl();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../subgraph.graphql");
        let current = std::fs::read_to_string(&path).unwrap_or_default();

        if current != sdl {
            std::fs::write(&path, &sdl).expect("write subgraph.graphql");
            panic!("subgraph.graphql was out of date and has been regenerated, commit it");
        }
    }
}
