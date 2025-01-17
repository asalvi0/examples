use async_graphql::{
    extensions::OpenTelemetry, EmptyMutation, EmptySubscription, Object, Result, Schema,
};
use async_graphql_poem::GraphQL;
use opentelemetry::sdk::export::trace::stdout;
use poem::{listener::TcpListener, post, EndpointExt, Route, Server};

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self) -> Result<String> {
        Ok("World".to_string())
    }
}

#[tokio::main]
async fn main() {
    let tracer = stdout::new_pipeline().install_simple();
    let opentelemetry_extension = OpenTelemetry::new(tracer);

    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .extension(opentelemetry_extension)
        .finish();

    let app = Route::new()
        .at("/", post(GraphQL::new(schema.clone())))
        .data(schema);

    let example_curl = "\
    curl '127.0.0.1:8000' \
    -X POST \
    -H 'content-type: application/json' \
    --data '{ \"query\": \"{ hello }\" }'";

    println!("Run this curl command from another terminal window to see opentelemetry output in this terminal.\n\n{example_curl}\n\n");

    Server::new(TcpListener::bind("127.0.0.1:8000"))
        .run(app)
        .await
        .unwrap();
}
