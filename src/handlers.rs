use crate::auth::*;
use crate::db;
use crate::lang;
use crate::models::*;
use crate::response::*;

use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use deadpool_postgres::{Client, Pool};

#[get("/")]
pub async fn status() -> impl Responder {
    HttpResponse::Ok().json(StatusResponse {
        status: "Up".to_string(),
    })
}

pub mod user {
    use super::*;

    #[get("/user/")]
    pub async fn get_users(
        db_pool: web::Data<Pool>,
        query: web::Query<GetUsersRequest>,
        _auth_user: ClaimsUser,
    ) -> impl Responder {
        let client: Client = match db_pool.get().await {
            Ok(client) => client,
            Err(err) => {
                eprintln!("{}", err);
                return user_res::get_fetch_users_error();
            }
        };

        let offset = match query.offset {
            Some(offset) => offset,
            None => 0,
        };

        let result = db::user::get_users(&client, &offset).await;

        match result {
            Ok(users) => HttpResponse::Ok().json(GetUsersResponse::new(users)),
            Err(_) => user_res::get_fetch_users_error(),
        }
    }

    #[post("/user/reg/")]
    pub async fn register(
        db_pool: web::Data<Pool>,
        mut json: web::Json<RegisterRequest>,
    ) -> impl Responder {
        let client: Client = db_pool
            .get()
            .await
            .expect("Error connecting to the database");

        let existing_user_result = db::user::get_user(&client, &json.username).await;
        if let Ok(user_opt) = existing_user_result {
            if let Some(_) = user_opt {
                return user_res::get_user_exists_error();
            }
        }

        if let Err(err) = handle_pass_hash(&mut json) {
            eprintln!("{}", err);
            return user_res::get_registration_error();
        };

        let create_result =
            db::user::create_user(&client, &json.username, &json.password, &json.native_lang).await;

        match create_result {
            Ok(user) => HttpResponse::Created().json(RegisterResponse::new(user)),
            Err(_) => user_res::get_registration_error(),
        }
    }

    #[post("/user/log/")]
    pub async fn login(db_pool: web::Data<Pool>, json: web::Json<LoginRequest>) -> impl Responder {
        let client: Client = db_pool
            .get()
            .await
            .expect("Error connecting to the database");

        let result = db::user::get_user(&client, &json.username).await;
        let user = match result {
            Ok(user_opt) => match user_opt {
                Some(user) => user,
                None => return user_res::get_auth_failed_error(),
            },
            Err(_) => return user_res::get_auth_failed_error(),
        };

        match attempt_user_login(json, user) {
            Ok(token) => HttpResponse::Ok().json(LoginResponse { token: token }),
            Err(_) => user_res::get_auth_failed_error(),
        }
    }

    pub mod data {
        use super::*;

        #[get("/user/data/")]
        pub async fn get_user_word_data(
            db_pool: web::Data<Pool>,
            auth_user: ClaimsUser,
        ) -> impl Responder {
            let client: Client = match db_pool.get().await {
                Ok(client) => client,
                Err(err) => {
                    eprintln!("{}", err);
                    return user_res::get_fetch_data_error();
                }
            };

            let result = db::user::word_data::get_user_word_data(&client, &auth_user.id).await;

            match result {
                Ok(data) => HttpResponse::Ok().json(GetWordDataResponse::new(data)),
                Err(_) => user_res::get_fetch_data_error(),
            }
        }

        #[put("/user/data/status/")]
        pub async fn update_word_status(
            db_pool: web::Data<Pool>,
            json: web::Json<UpdateWordStatusRequest>,
            auth_user: ClaimsUser,
        ) -> impl Responder {
            let client: Client = match db_pool.get().await {
                Ok(client) => client,
                Err(err) => {
                    eprintln!("{}", err);
                    return user_res::get_update_word_status_error();
                }
            };

            let result = db::user::word_data::update_word_status(
                &client,
                &auth_user.id,
                &json.lang,
                &json.word,
                &json.status,
            )
            .await;

            match result {
                Ok(()) => get_success(),
                Err(_) => user_res::get_update_word_status_error(),
            }
        }

        #[put("/user/data/definition/")]
        pub async fn update_word_definition(
            db_pool: web::Data<Pool>,
            json: web::Json<UpdateWordDefinitionRequest>,
            auth_user: ClaimsUser,
        ) -> impl Responder {
            let client: Client = match db_pool.get().await {
                Ok(client) => client,
                Err(err) => {
                    eprintln!("{}", err);
                    return user_res::get_update_word_definition_error();
                }
            };

            let result = db::user::word_data::update_word_definition(
                &client,
                &auth_user.id,
                &json.lang,
                &json.word,
                &json.definition,
            )
            .await;

            match result {
                Ok(()) => get_success(),
                Err(_) => user_res::get_update_word_definition_error(),
            }
        }
    }
}

pub mod article {
    use super::*;

    #[post("/article/")]
    pub async fn create_article(
        db_pool: web::Data<Pool>,
        json: web::Json<NewArticleRequest>,
        auth_user: ClaimsUser,
    ) -> impl Responder {
        let client: Client = match db_pool.get().await {
            Ok(client) => client,
            Err(err) => {
                eprintln!("{}", err);
                return article_res::get_create_article_error();
            }
        };

        let words = lang::get_words(&json.content[..], &json.language[..]);
        let unique_words = lang::get_unique_words(&words);

        let result = db::article::create_article(
            &client,
            &json.title,
            &json.author,
            &json.content,
            &auth_user.id,
            &json.language,
            &json.tags,
            &words,
            &unique_words,
            &json.is_private,
        )
        .await;

        match result {
            Ok(article) => HttpResponse::Created().json(NewArticleResponse::from(article)),
            Err(_) => article_res::get_create_article_error(),
        }
    }

    pub mod system {
        use super::*;

        #[get("/article/system/")]
        pub async fn get_articles(
            db_pool: web::Data<Pool>,
            query: web::Query<GetArticlesRequest>,
            _auth_user: ClaimsUser,
        ) -> impl Responder {
            let client: Client = match db_pool.get().await {
                Ok(client) => client,
                Err(err) => {
                    eprintln!("{}", err);
                    return article_res::get_fetch_articles_error();
                }
            };

            let offset = match query.offset {
                Some(offset) => offset,
                None => 0,
            };

            let result = db::article::system::get_system_article_list(&client, &offset).await;

            match result {
                Ok(articles) => HttpResponse::Ok().json(GetArticlesResponse::new(articles)),
                Err(_) => article_res::get_fetch_articles_error(),
            }
        }

        #[get("/article/system/{article_id}/")]
        pub async fn get_full_article(
            db_pool: web::Data<Pool>,
            web::Path(article_id): web::Path<i32>,
            _auth_user: ClaimsUser,
        ) -> impl Responder {
            let client: Client = match db_pool.get().await {
                Ok(client) => client,
                Err(err) => {
                    eprintln!("{}", err);
                    return article_res::get_fetch_article_error();
                }
            };

            let result = db::article::system::get_system_article(&client, &article_id).await;

            match result {
                Ok(article_opt) => match article_opt {
                    Some(article) => HttpResponse::Ok().json(GetFullArticleResponse::new(article)),
                    None => article_res::get_article_not_found(),
                },
                Err(_) => article_res::get_fetch_article_error(),
            }
        }
    }

    pub mod user {
        use super::*;

        #[get("/article/user/uploaded/")]
        pub async fn get_user_uploaded_articles(
            db_pool: web::Data<Pool>,
            query: web::Query<GetArticlesRequest>,
            auth_user: ClaimsUser,
        ) -> impl Responder {
            let client: Client = match db_pool.get().await {
                Ok(client) => client,
                Err(err) => {
                    eprintln!("{}", err);
                    return article_res::get_fetch_articles_error();
                }
            };

            let offset = match query.offset {
                Some(offset) => offset,
                None => 0,
            };

            let result =
                db::article::user::get_user_uploaded_article_list(&client, &auth_user.id, &offset)
                    .await;

            match result {
                Ok(articles) => HttpResponse::Ok().json(GetArticlesResponse::new(articles)),
                Err(_) => article_res::get_fetch_articles_error(),
            }
        }

        #[get("/article/user/saved/")]
        pub async fn get_saved_articles(
            db_pool: web::Data<Pool>,
            query: web::Query<GetArticlesRequest>,
            auth_user: ClaimsUser,
        ) -> impl Responder {
            let client: Client = match db_pool.get().await {
                Ok(client) => client,
                Err(err) => {
                    eprintln!("{}", err);
                    return article_res::get_fetch_articles_error();
                }
            };

            let offset = match query.offset {
                Some(offset) => offset,
                None => 0,
            };

            let result =
                db::article::user::get_user_saved_article_list(&client, &auth_user.id, &offset)
                    .await;

            match result {
                Ok(articles) => HttpResponse::Ok().json(GetArticlesResponse::new(articles)),
                Err(_) => article_res::get_fetch_articles_error(),
            }
        }

        #[get("/article/user/saved/{article_id}/")]
        pub async fn get_full_article(
            db_pool: web::Data<Pool>,
            web::Path(article_id): web::Path<i32>,
            auth_user: ClaimsUser,
        ) -> impl Responder {
            let client: Client = match db_pool.get().await {
                Ok(client) => client,
                Err(err) => {
                    eprintln!("{}", err);
                    return article_res::get_fetch_article_error();
                }
            };

            let result =
                db::article::user::get_user_article(&client, &article_id, &auth_user.id).await;

            match result {
                Ok(article_opt) => match article_opt {
                    Some(article) => HttpResponse::Ok().json(GetFullArticleResponse::new(article)),
                    None => article_res::get_article_not_found(),
                },
                Err(_) => article_res::get_fetch_article_error(),
            }
        }

        #[put("/article/user/saved/")]
        pub async fn save_article(
            db_pool: web::Data<Pool>,
            json: web::Json<ArticleRequest>,
            auth_user: ClaimsUser,
        ) -> impl Responder {
            let client: Client = match db_pool.get().await {
                Ok(client) => client,
                Err(err) => {
                    eprintln!("{}", err);
                    return article_res::get_save_article_error();
                }
            };

            let result =
                db::article::user::user_save_article(&client, &auth_user.id, &json.article_id)
                    .await;

            match result {
                Ok(()) => get_success(),
                Err(_) => article_res::get_save_article_error(),
            }
        }

        #[delete("/article/user/saved/{article_id}/")]
        pub async fn remove_saved_article(
            db_pool: web::Data<Pool>,
            web::Path(article_id): web::Path<i32>,
            auth_user: ClaimsUser,
        ) -> impl Responder {
            let client: Client = match db_pool.get().await {
                Ok(client) => client,
                Err(err) => {
                    eprintln!("{}", err);
                    return article_res::get_delete_article_error();
                }
            };

            let result =
                db::article::user::user_delete_saved_article(&client, &auth_user.id, &article_id)
                    .await;

            match result {
                Ok(()) => get_success(),
                Err(_) => article_res::get_delete_article_error(),
            }
        }
    }
}
