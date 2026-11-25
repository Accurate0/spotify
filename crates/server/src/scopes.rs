//! Enforces `#[graphql(requires_scopes = "...")]` in the subgraph itself, as crates/subgraph does
//! in the k8s repo: each entry is one alternative whose space-separated scopes must all be held.

use std::collections::BTreeSet;
use std::sync::Arc;

use async_graphql::extensions::{
    Extension, ExtensionContext, ExtensionFactory, NextResolve, ResolveInfo,
};
use async_graphql::registry::MetaType;
use async_graphql::{ErrorExtensions, ServerResult, Value};

use crate::auth::Claims;

pub struct Scopes;

impl ExtensionFactory for Scopes {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(ScopesExtension)
    }
}

struct ScopesExtension;

fn satisfied(required: &[String], held: &BTreeSet<&str>) -> bool {
    required.is_empty()
        || required
            .iter()
            .any(|all| all.split_whitespace().all(|scope| held.contains(scope)))
}

fn type_scopes(meta: &MetaType) -> &[String] {
    match meta {
        MetaType::Object {
            requires_scopes, ..
        }
        | MetaType::Interface {
            requires_scopes, ..
        } => requires_scopes,
        _ => &[],
    }
}

#[async_graphql::async_trait::async_trait]
impl Extension for ScopesExtension {
    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextResolve<'_>,
    ) -> ServerResult<Option<Value>> {
        if !info.is_for_introspection
            && let Some(parent) = ctx.schema_env.registry.types.get(info.parent_type)
        {
            let held = ctx
                .data_opt::<Claims>()
                .map(Claims::scopes)
                .unwrap_or_default();
            let field = parent
                .field_by_name(info.name)
                .map(|field| field.requires_scopes.as_slice())
                .unwrap_or_default();

            for required in [type_scopes(parent), field] {
                if !satisfied(required, &held) {
                    return Err(async_graphql::Error::new(format!(
                        "missing scope {}",
                        required.join(" or ")
                    ))
                    .extend_with(|_, e| e.set("code", "FORBIDDEN"))
                    .into_server_error(info.field.name.pos));
                }
            }
        }

        next.run(ctx, info).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_alternative_with_all_its_scopes_is_enough() {
        let held: BTreeSet<&str> = ["spotify:history:read"].into();
        assert!(satisfied(&[], &held));
        assert!(satisfied(&["spotify:history:read".to_owned()], &held));
        assert!(!satisfied(&["spotify:export:write".to_owned()], &held));
    }
}
