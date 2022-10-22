// Following guide:: https://cloudmaker.dev/how-to-create-a-rest-api-in-rust/

//--comfort features
//auto restart
use listenfd::ListenFd;
//env vars
use dotenvy::dotenv;

//pretty logs
extern crate pretty_env_logger;
#[macro_use] extern crate log;
#[allow(unused_imports)]
use std::{env, io::Error};

//--IMPORT-ANT IMPORTS
use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware::Logger};
use sqlx::{postgres::{PgPool, PgPoolOptions}};
use lazy_static::lazy_static;
use async_once::AsyncOnce;

/* 
use time::{OffsetDateTime, UtcOffset};
use std::{env, fs::{read_to_string, read_dir}, fmt::Display};
use rust_decimal::prelude::*;*/

pub mod auth;
pub mod routes;
pub mod embedded_asset_serve;

//-------ROUTES
#[get("/")]
async fn genesis() -> impl Responder {
    trace!("Greeting User");
    return "✨ New Rust🦀 Project! ✨"
}

lazy_static! {
    static ref DB_POOL: AsyncOnce<PgPool> = AsyncOnce::new(async{
        init_db().await
    });
}
async fn init_db() -> PgPool {
    // Create a connection pool
    info!("Initializing Database Pool...");
    let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&env::var("DATABASE_URL").unwrap()).await.unwrap();
    pool
}
macro_rules! db {
    () => {
        DB_POOL.get().await
    };
}

///------INIT^
#[allow(unreachable_code)]
#[actix_web::main]
async fn main() -> Result<(),std::io::Error> {
    dotenv().ok();
    pretty_env_logger::init();
    info!("Hello, world!");

    // Migrate db
    info!("Migrating Database...");
    sqlx::migrate!().run(db!()).await.map_err(|err| Error::new(std::io::ErrorKind::Other, format!("{:?}", err)))?;

    // Setup Server
    let mut server = HttpServer::new( || {
        App::new()
            .wrap(Logger::new(r#"[%a] "%r" %s %bb "%{Referer}i" "%{User-Agent}i" %Dms"#))
            .configure(auth::config)
            .configure(embedded_asset_serve::config)
            .service(genesis)
    } );

    // take over socket from old process, if available
    let mut listenfd = ListenFd::from_env();
    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => {
            info!("Reusing Socket for: http://{}", listener.local_addr()?.to_string());
            server.listen(listener)?
        },
        None => {
            // Bind to Adress specified in .env
            let host = env::var("HOST").expect("Host not set");
            let port = env::var("PORT").expect("Port not set");
            info!("Binding Server to http://{}:{}", host, port);
            server.bind(format!("{}:{}", host, port))?
        }
    };

    //block 
    server.run().await.into()
}

fn handle_err_except_duplicate(err: sqlx::Error) -> Result<(), sqlx::Error>{
    if let Some(db_err) = err.as_database_error() {
        if let Some(err_code) = db_err.code() {
            if err_code == "23505" {
                trace!("Report Group already in database.");
                return Ok(());
            } 
        }
    } 
    Err(err)
}
