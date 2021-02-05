use crate::app_config::AppConfig;
use crate::auth::*;
use crate::db;
use crate::models::*;
use crate::response::*;

use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use deadpool_postgres::{Client, Pool};

#[get("/")]
pub async fn status() -> impl Responder {
    HttpResponse::Ok().json(StatusResponse {
        status: "Up".to_string(),
    })
}

#[get("/users{_:/?}")]
pub async fn get_users(db_pool: web::Data<Pool>) -> impl Responder {
    let client: Client = match db_pool.get().await {
        Ok(client) => client,
        Err(err) => {
            eprintln!("{}", err);
            return get_fetch_users_error();
        }
    };

    let result = db::get_users(&client).await;

    match result {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(_) => get_fetch_users_error(),
    }
}

#[post("/register{_:/?}")]
pub async fn register(
    db_pool: web::Data<Pool>,
    config: web::Data<AppConfig>,
    mut json: web::Json<RegisterRequest>,
) -> impl Responder {
    let client: Client = db_pool
        .get()
        .await
        .expect("Error connecting to the database");

    let existing_user = db::get_user(&client, json.username.clone()).await;
    if let Ok(_) = existing_user {
        return get_user_exists_error();
    }

    if let Err(err) = handle_pass_hash(config, &mut json) {
        eprintln!("{}", err);
        return get_registration_error();
    };

    let create_result = db::create_user(
        &client,
        json.username.clone(),
        json.password.clone(),
        json.native_lang.clone(),
    )
    .await;

    match create_result {
        Ok(user) => HttpResponse::Created().json(user),
        Err(_) => get_registration_error(),
    }
}

#[post("/login{_:/?}")]
pub async fn login(
    db_pool: web::Data<Pool>,
    config: web::Data<AppConfig>,
    json: web::Json<LoginRequest>,
) -> impl Responder {
    let client: Client = db_pool
        .get()
        .await
        .expect("Error connecting to the database");

    let result = db::get_user(&client, json.username.clone()).await;
    let user = match result {
        Ok(user) => user,
        Err(_) => return get_auth_failed_error(),
    };

    match attempt_user_login(config, json, user) {
        Ok(token) => HttpResponse::Ok().json(LoginResponse { token: token }),
        Err(_) => get_auth_failed_error(),
    }
}

#[post("/auth{_:/?}")]
pub async fn auth(req: HttpRequest, config: web::Data<AppConfig>) -> impl Responder {
    if let Err(err) = attempt_token_auth(req, config) {
        eprintln!("{}", err);
        get_auth_failed_error()
    } else {
        get_success()
    }
}

#[get("/articles{_:/?}")]
pub async fn get_articles(
    req: HttpRequest,
    db_pool: web::Data<Pool>,
    config: web::Data<AppConfig>,
    json: Option<web::Json<GetArticlesRequest>>,
) -> impl Responder {
    if let Err(_) = attempt_token_auth(req, config) {
        return get_auth_failed_error();
    }

    let client: Client = match db_pool.get().await {
        Ok(client) => client,
        Err(err) => {
            eprintln!("{}", err);
            return get_fetch_articles_error();
        }
    };

    let offset = match json {
        Some(json) => match json.offset {
            Some(offset) => offset,
            None => 0,
        },
        None => 0
    };

    let result = db::get_articles(&client, offset).await;

    match result {
        Ok(articles) => HttpResponse::Ok().json(GetArticlesResponse::new(articles)),
        Err(_) => get_fetch_articles_error(),
    }
}

#[post("/articles{_:/?}")]
pub async fn create_article(
    req: HttpRequest,
    db_pool: web::Data<Pool>,
    config: web::Data<AppConfig>,
    json: web::Json<NewArticleRequest>,
) -> impl Responder {
    let auth_result = attempt_token_auth(req, config);
    if let Err(_) = auth_result {
        return get_auth_failed_error();
    }
    let user = auth_result.unwrap();

    let client: Client = match db_pool.get().await {
        Ok(client) => client,
        Err(err) => {
            eprintln!("{}", err);
            return get_create_article_error();
        }
    };

    let result = db::create_article(
        &client,
        json.title.clone(),
        json.author.clone(),
        json.content.clone(),
        user.id
    )
    .await;

    match result {
        Ok(article) => HttpResponse::Created().json(NewArticleResponse::from(article)),
        Err(_) => get_create_article_error(),
    }
}
