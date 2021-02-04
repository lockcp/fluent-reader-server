mod app_config;
mod auth;
mod db;
mod handlers;
mod models;
mod response;

extern crate argon2;

use crate::app_config::AppConfig;
use crate::handlers::*;
use std::process;

use actix_web::{App, HttpServer};
use dotenv::dotenv;
use tokio_postgres::NoTls;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = AppConfig::from_env().unwrap();

    let pool = config.pg.create_pool(NoTls).unwrap();

    // Allow database to not yet be ready when server is starting
    // if let Err(e) = pool.get().await {
    //     eprintln!("Couldn't connect to database. Aborting with error:\n{}", e);
    //     process::exit(1);
    // }

    println!(
        "Starting server at http://{0}:{1}/",
        config.server.host, config.server.port
    );

    let address: String = config.server.host.clone() + ":" + &config.server.port.to_string();

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(config.clone())
            .service(login)
            .service(auth)
            .service(get_users)
            .service(register)
            .service(status)
        // .service(get_todos)
        // .service(create_todo)
        // .service(get_items)
        // .service(check_item)
    })
    .bind(address)?
    .run()
    .await
}
