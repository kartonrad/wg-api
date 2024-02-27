//! Actix Web juniper example
//!
//! A simple example integrating juniper in Actix Web

extern crate core;

use std::{io, sync::Arc};
use std::str::FromStr;

use actix_cors::Cors;
use actix_web::{
    get, middleware, route,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use actix_web_lab::respond::Html;
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};
use sqlx::PgPool;
use crate::auth::{TryIdentity};

pub mod schema;
// pub mod file_uploads;
pub mod auth;
//pub mod routes;
pub mod embedded_asset_serve;
//pub mod db_types;

use crate::schema::{AppContext, create_schema, Schema};

/// GraphiQL playground UI
#[get("/graphiql")]
async fn graphql_playground() -> impl Responder {
    Html(graphiql_source("/graphql", None))
}

/// GraphQL endpoint
#[route("/graphql", method = "GET", method = "POST")]
async fn graphql(st: web::Data<Schema>, db_pool: web::Data<PgPool>, data: web::Json<GraphQLRequest>, identity: TryIdentity) -> impl Responder {
    let ctx = AppContext {
        authenticated_user: identity.0,
        db_pool: db_pool.get_ref().clone(),
        server_origin: "localhost:8080".to_string()
    };
    let user = data.execute(&st, &ctx).await;
    HttpResponse::Ok().json(user)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Create Juniper schema
    let schema = Arc::new(create_schema());

    log::info!("starting HTTP server on port 8080");
    log::info!("GraphiQL playground: http://localhost:8080/graphiql");

    let server_host = std::env::var("BIND_HOST").unwrap_or("127.0.0.1".to_string());
    let server_port = u16::from_str(
        &std::env::var("BIND_PORT").unwrap_or("8080".to_string())
    ).expect("BIND_PORT Variable to be a number");

    let pool = sqlx::PgPool::connect(
        &std::env::var("DATABASE_URL").expect("Enviroment Variable DATABASE_URL is required")
    ).await.expect("Postgres failed to connect");

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(Data::from(schema.clone()))
            .app_data(web::Data::new(pool.clone()))
            .configure(auth::config)
            .configure(embedded_asset_serve::config)
            .service(graphql)
            .service(graphql_playground)
            //.service(file_uploads::get_uploads_service)
            // the graphiql UI requires CORS to be enabled
            .wrap(Cors::permissive())
            .wrap(middleware::Logger::default())
    })
        .workers(2)
        .bind((server_host, server_port))?
        .run()
        .await
}