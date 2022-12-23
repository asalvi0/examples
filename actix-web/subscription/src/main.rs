use actix_web::{guard, web, web::Data, App, HttpRequest, HttpResponse, HttpServer, Result};
use async_graphql::{http::GraphiQLSource, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use books::{BooksSchema, MutationRoot, QueryRoot, Storage, SubscriptionRoot};

async fn index(schema: web::Data<BooksSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn index_graphiql() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            GraphiQLSource::build()
                .endpoint("/")
                .subscription_endpoint("/")
                .finish(),
        ))
}

async fn index_ws(
    schema: web::Data<BooksSchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    GraphQLSubscription::new(Schema::clone(&*schema)).start(&req, payload)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let schema = Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(Storage::default())
        .finish();

    println!("GraphiQL IDE: http://localhost:8000");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(schema.clone()))
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(
                web::resource("/")
                    .guard(guard::Get())
                    .guard(guard::Header("upgrade", "websocket"))
                    .to(index_ws),
            )
            .service(web::resource("/").guard(guard::Get()).to(index_graphiql))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
