use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use tokio_pg_mapper_derive::PostgresMapper;

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

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "article")]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub author: Option<String>,
    pub content: String,
    pub content_length: i32,
    pub created_on: SystemTime,
    pub is_system: bool,
    pub uploader_id: i32,
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "article")]
pub struct SimpleArticle {
    pub id: i32,
    pub title: String,
    pub author: Option<String>,
    // no content
    pub content_length: i32,
    pub created_on: SystemTime,
    pub is_system: bool,
    // no uploader_id
}

// Request/Response Models

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub native_lang: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

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

// Authentication

#[derive(Serialize, Deserialize)]
pub struct ClaimsUser {
    pub id: i32,
    pub username: String,
    pub created_on: SystemTime,
    pub native_lang: String,
}

impl ClaimsUser {
    pub fn from_user(user: &User) -> ClaimsUser {
        ClaimsUser {
            id: user.id,
            username: user.username.clone(),
            created_on: user.created_on,
            native_lang: user.native_lang.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TokenClaims {
    pub exp: usize,
    pub user: ClaimsUser,
}

// Articles
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
    pub fn new(articles: Vec<SimpleArticle>) -> GetArticlesResponse {
        let count = articles.len() as i64;
        GetArticlesResponse {
            articles: articles,
            count: count,
        }
    }
}

#[derive(Deserialize)]
pub struct NewArticleRequest {
    pub title: String,
    pub author: Option<String>,
    pub content: String,
}

#[derive(Serialize)]
pub struct NewArticleResponse {
    pub article: Article
}

impl NewArticleResponse {
    pub fn from(article: Article) -> NewArticleResponse {
        NewArticleResponse {
            article: article
        }
    }
}