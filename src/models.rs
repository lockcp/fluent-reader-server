use actix_web::error::ErrorUnauthorized;
use actix_web::{dev, Error, FromRequest, HttpRequest};
use futures_util::future::{err, ok, Ready};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use tokio_pg_mapper_derive::PostgresMapper;

use crate::auth;

// Database Models

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "fruser")]
pub struct User {
    pub id: i32,
    pub username: String,
    pub pass: String,
    pub created_on: SystemTime,
    pub native_lang: String,
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "fruser")]
pub struct SimpleUser {
    pub id: i32,
    pub username: String,
}

impl SimpleUser {
    #[inline]
    pub fn new(user: User) -> SimpleUser {
        SimpleUser {
            id: user.id,
            username: user.username,
        }
    }
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "article")]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub author: Option<String>,
    pub content: String,
    pub content_length: i32,
    pub words: Vec<String>,
    pub unique_words: serde_json::Value,
    pub created_on: SystemTime,
    pub is_system: bool,
    pub uploader_id: i32,
    pub lang: String,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "article")]
pub struct SimpleArticle {
    pub id: i32,
    pub title: String,
    pub author: Option<String>,
    // no content
    pub content_length: i32,
    // no words
    // no unique words
    pub created_on: SystemTime,
    pub is_system: bool,
    // no uploader_id
    pub lang: String,
    pub tags: Vec<String>,
}

// Request/Response Models

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: String,
}

// General

#[derive(Serialize)]
pub struct ResultResponse {
    pub success: bool,
}

#[derive(Serialize)]
pub struct Message {
    pub message: &'static str,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: &'static str,
}

// Registration

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub native_lang: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub user: SimpleUser,
}

impl RegisterResponse {
    #[inline]
    pub fn new(user: User) -> RegisterResponse {
        RegisterResponse {
            user: SimpleUser::new(user),
        }
    }
}

// Login

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

// Authentication

#[derive(Serialize, Deserialize)]
pub struct ClaimsUser {
    pub id: i32,
    pub username: String,
    pub created_on: SystemTime,
    pub native_lang: String,
}

impl ClaimsUser {
    #[inline]
    pub fn from_user(user: &User) -> ClaimsUser {
        ClaimsUser {
            id: user.id,
            username: user.username.clone(),
            created_on: user.created_on,
            native_lang: user.native_lang.clone(),
        }
    }
}

impl FromRequest for ClaimsUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    #[inline]
    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        match auth::attempt_token_auth(req) {
            Ok(user) => ok(user),
            Err(error) => {
                eprintln!("{}", error);
                err(ErrorUnauthorized("auth_fail"))
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TokenClaims {
    pub exp: usize,
    pub user: ClaimsUser,
}

// Users

#[derive(Deserialize)]
pub struct GetUsersRequest {
    pub offset: Option<i64>,
}

#[derive(Serialize)]
pub struct GetUsersResponse {
    pub users: Vec<SimpleUser>,
    pub count: i64,
}

impl GetUsersResponse {
    #[inline]
    pub fn new(users: Vec<SimpleUser>) -> GetUsersResponse {
        let count = users.len() as i64;
        GetUsersResponse {
            users: users,
            count: count,
        }
    }
}

// Articles

// get article list
#[derive(Deserialize)]
pub struct GetArticlesRequest {
    pub offset: Option<i64>,
}

#[derive(Serialize)]
pub struct GetArticlesResponse {
    pub articles: Vec<SimpleArticle>,
    pub count: i64,
}

impl GetArticlesResponse {
    #[inline]
    pub fn new(articles: Vec<SimpleArticle>) -> GetArticlesResponse {
        let count = articles.len() as i64;
        GetArticlesResponse {
            articles: articles,
            count: count,
        }
    }
}

// get full article

#[derive(Deserialize)]
pub struct GetFullArticleRequest {
    pub article_id: i64,
}

#[derive(Serialize)]
pub struct GetFullArticleResponse {
    pub article: Article,
}

impl GetFullArticleResponse {
    #[inline]
    pub fn new(article: Article) -> GetFullArticleResponse {
        GetFullArticleResponse { article: article }
    }
}

// post new article
#[derive(Deserialize)]
pub struct NewArticleRequest {
    pub title: String,
    pub author: Option<String>,
    pub content: String,
    pub language: String,
    pub tags: Option<Vec<String>>,
    pub is_private: bool
}

#[derive(Serialize)]
pub struct NewArticleResponse {
    pub article: Article,
}

impl NewArticleResponse {
    #[inline]
    pub fn from(article: Article) -> NewArticleResponse {
        NewArticleResponse { article: article }
    }
}
