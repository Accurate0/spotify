use async_graphql::{EmptySubscription, Schema, SchemaBuilder};

mod mutation;
pub mod node;
pub mod objects;
mod query;

pub use mutation::MutationRoot;
pub use query::QueryRoot;

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn builder() -> SchemaBuilder<QueryRoot, MutationRoot, EmptySubscription> {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
}

pub fn sdl() -> String {
    builder().finish().sdl()
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
}
