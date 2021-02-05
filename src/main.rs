mod app_config;
mod auth;
mod db;
mod handlers;
mod models;
mod response;

extern crate argon2;

use crate::app_config::AppConfig;
use crate::handlers::*;

use actix_web::{middleware::Logger, App, HttpServer};
use dotenv::dotenv;
use env_logger::Env;
use std::process;
use tokio_postgres::NoTls;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(err) = dotenv() {
        eprintln!("Couldn't parse .env file. Exiting with error:\n{}", err);
        process::exit(1);
    }

    let config = AppConfig::from_env().unwrap();

    let pool = config.pg.create_pool(NoTls).unwrap();

    println!(
        "Starting server at http://{0}:{1}/",
        config.server.host, config.server.port
    );
    
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let address: String = config.server.host.clone() + ":" + &config.server.port.to_string();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .data(pool.clone())
            .data(config.clone())
            .service(get_articles)
            .service(create_article)
            .service(login)
            .service(auth)
            .service(get_users)
            .service(register)
            .service(status)
    })
    .bind(address)?
    .run()
    .await
}
