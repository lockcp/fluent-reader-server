mod app_config;
mod auth;
mod db;
mod handlers;
mod lang;
mod models;
mod response;
mod util;

extern crate argon2;

use crate::app_config::CONFIG;
use crate::handlers::*;

use actix_web::{
    middleware::{Logger, NormalizePath},
    web, App, HttpServer,
};
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

    println!(
        "Starting server at http://{0}:{1}/",
        CONFIG.server.host, CONFIG.server.port
    );

    let address: String = CONFIG.server.host.clone() + ":" + &CONFIG.server.port.to_string();
    let pool = CONFIG.pg.create_pool(NoTls).unwrap();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let json_config = web::JsonConfig::default().limit(CONFIG.server.json_max_size);

    HttpServer::new(move || {
        App::new()
            .wrap(NormalizePath::default())
            .wrap(Logger::default())
            .app_data(json_config.clone())
            .data(pool.clone())
            .service(user::get_users)
            .service(user::login)
            .service(user::register)
            .service(user::data::get_user_word_data)
            .service(user::data::update_word_status)
            .service(user::data::update_word_definition)
            .service(article::create_article)
            .service(article::system::get_articles)
            .service(article::system::get_full_article)
            .service(article::user::get_single_user_article_list)
            .service(article::user::get_all_user_article_list)
            .service(article::user::get_saved_article_list)
            .service(article::user::get_full_article)
            .service(article::user::save_article)
            .service(article::user::remove_saved_article)
            .service(status)
    })
    .bind(address)?
    .run()
    .await
}
