mod custom_extension;

use std::convert::Infallible;

use hyper::{Body, header, Method, Request, Response, Server, StatusCode};
use hyper::header::HeaderValue;
use hyper::service::{make_service_fn, service_fn};

use async_graphql::{BatchRequest, EmptyMutation, EmptySubscription, http::GraphiQLSource, Schema};
use starwars::{QueryRoot, StarWars};
use crate::custom_extension::NeureloExtension;

fn graphiql() -> Response<Body> {
    let html = GraphiQLSource::build().endpoint("/").finish();

    Response::builder()
        .header("Content-Type", "text/html")
        .body(Body::from(html))
        .unwrap()
}

#[tokio::main]
async fn main() {
    let make_svc = make_service_fn(|_| {
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| async move {
                let ext = NeureloExtension::new();

                let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
                    .data(StarWars::new())
                    .extension(ext)
                    .finish();

                match (req.method(), req.uri().path()) {
                    (&Method::GET, "/") | (&Method::GET, "/index.html") => Ok::<_, Infallible>(graphiql()),
                    (&Method::POST, "/") => {
                        let content_type = req
                            .headers()
                            .get(header::CONTENT_TYPE)
                            .map(HeaderValue::to_str);

                        let mut body: Body = Body::empty();

                        let request = match content_type {
                            Some(Ok("application/json")) => parse_json_request(req.into_body()).await,
                            Some(Ok("text/plain;charset=UTF-8")) => parse_json_request(req.into_body()).await,
                            Some(Ok("application/graphql")) => parse_graphql_request(req.into_body()).await,
                            _ => None,
                        };

                        if let Some(request) = request {
                            let res = schema.execute_batch(request).await;
                            //body = Body::from(serde_json::to_string_pretty(&res).unwrap());
                            body = Body::from(serde_json::to_string(&res).unwrap());
                        }

                        let response = Response::builder()
                            .header("Content-Type", "application/json")
                            .body(body)
                            .unwrap();

                        Ok::<_, Infallible>(response)
                    }
                    _ => {
                        Ok::<_, Infallible>(Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::from("Not Found"))
                            .unwrap())
                    }
                }
            }))
        }
    });

    println!("GraphiQL IDE: http://localhost:8000");

    Server::bind(&"127.0.0.1:8000".parse().unwrap())
        .serve(make_svc)
        .await
        .unwrap();
}

async fn parse_graphql_request(body: Body) -> Option<BatchRequest> {
    let chunk = hyper::body::to_bytes(body).await.unwrap();
    let query = String::from_utf8(chunk.iter().cloned().collect()).unwrap();

    Some(BatchRequest::Single(async_graphql::Request::new(query)))
}

async fn parse_json_request(body: Body) -> Option<BatchRequest> {
    let chunk = hyper::body::to_bytes(body).await.unwrap();
    let input = String::from_utf8(chunk.iter().cloned().collect()).unwrap();
    //let input= serde_json::to_string_pretty(&chunk)

    Some(serde_json::from_str::<BatchRequest>(&input).unwrap())
}
