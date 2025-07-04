use actix_cors::Cors;
use actix_web::{
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
    dev::ServiceRequest,
    get,
    http::header,
    middleware, post,
    web::{self, Data},
};
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::middleware::HttpAuthentication;
use dotenvy;
use juniper_actix::{graphiql_handler, graphql_handler, playground_handler};

use crate::api::{Context, Schema, schema};

mod auth;
mod errors;

#[get("/")]
async fn homepage() -> impl Responder {
    HttpResponse::Ok()
        .insert_header(("content-type", "text/html"))
        .message_body(
            "<html><h1>Axtell</h1>\
                   <div>visit <a href=\"/api/graphiql\">GraphiQL</a></div>\
                   <div>visit <a href=\"/api/playground\">GraphQL Playground</a></div>\
             </html>",
        )
}

#[post("/api/grapghql")]
async fn graphql(
    req: HttpRequest,
    payload: web::Payload,
    schema: Data<Schema>,
) -> Result<HttpResponse, Error> {
    let ctx = Context::try_new().map_err(|e| Into::<Box<dyn std::error::Error>>::into(e))?;
    graphql_handler(&schema, &ctx, req, payload).await
}

#[get("/api/playground")]
async fn playground() -> Result<HttpResponse, Error> {
    playground_handler("/api/graphql", None).await
}

#[get("/api/graphiql")]
async fn graphiql() -> Result<HttpResponse, Error> {
    graphiql_handler("/api/graphql", None).await
}

async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let config = req
        .app_data::<Config>()
        .map(|data| data.clone())
        .unwrap_or_else(Default::default);
    match auth::validate_token(credentials.token()).await {
        Ok(res) => {
            if res == true {
                Ok(req)
            } else {
                Err((AuthenticationError::from(config).into(), req))
            }
        }
        Err(_) => Err((AuthenticationError::from(config).into(), req)),
    }
}

pub async fn serve() -> std::io::Result<()> {
    HttpServer::new(|| {
        let auth = HttpAuthentication::bearer(validator);
        App::new()
            .app_data(Data::new(schema()))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["POST", "GET"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .wrap(auth)
            .service(graphql)
            .service(graphiql)
            .service(playground)
            .service(homepage)
    })
    .bind({
        dotenvy::dotenv().expect("could not load environment vars from .env");
        (
            dotenvy::var("WEBSERVER_BIND").unwrap_or("127.0.0.1".to_string()),
            dotenvy::var("WEBSERVER_PORT")
                .unwrap_or("8080".to_string())
                .parse()
                .expect("WEBSERVER_PORT env var must be an integer"),
        )
    })?
    .run()
    .await
}
