use std::sync::Arc;

use async_graphql::{async_trait, Name, Response, ServerResult, Value, Variables};
use async_graphql::extensions::{Extension, ExtensionContext, ExtensionFactory, NextExecute, NextParseQuery, NextResolve, ResolveInfo};
use async_graphql::indexmap::IndexMap;
use async_graphql::parser::types::{ExecutableDocument, OperationType, Selection};
use async_graphql_value::ConstValue;

pub(crate) struct CustomExtension;

impl CustomExtension {
    pub fn new() -> Self {
        Self {}
    }
}

impl ExtensionFactory for CustomExtension {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(CustomExtension)
    }
}

#[async_trait::async_trait]
impl Extension for CustomExtension {
    async fn parse_query(
        &self,
        ctx: &ExtensionContext<'_>,
        query: &str,
        variables: &Variables,
        next: NextParseQuery<'_>,
    ) -> ServerResult<ExecutableDocument> {
        let document = next.run(ctx, query, variables).await?;
        let is_schema = document
            .operations
            .iter()
            .filter(|(_, operation)| operation.node.ty == OperationType::Query)
            .any(|(_, operation)| operation.node.selection_set.node.items.iter()
                .any(|selection| matches!(&selection.node, Selection::Field(field) if field.node.name.node == "__schema")));
        if !is_schema {
            println!("[Execute] {}", ctx.stringify_execute_doc(&document, variables));
        }
        Ok(document)
    }

    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        // call your custom logic and convert the response to the proper type: `IndexMap<Name, ConstValue>`
        let mut data: IndexMap<Name, ConstValue> = IndexMap::new();
        let resp = Response::new(data);
        
        resp
        
        /*
        let resp = next.run(ctx, operation_name).await;
        if resp.is_err() {
            for err in &resp.errors {
                if !err.path.is_empty() {
                    let mut path = String::new();
                    for (idx, s) in err.path.iter().enumerate() {
                        if idx > 0 {
                            path.push('.');
                        }
                        match s {
                            PathSegment::Index(idx) => {
                                let _ = write!(&mut path, "{}", idx);
                            }
                            PathSegment::Field(name) => {
                                let _ = write!(&mut path, "{}", name);
                            }
                        }
                    }

                    println!("[Error] path={} message={}", path, err.message);
                } else {
                    println!("[Error] message={}", err.message);
                }
            }
        }
        */
    }
}
