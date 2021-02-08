use crate::models::*;
use deadpool_postgres::Client;
use std::io;
use tokio_pg_mapper::FromTokioPostgresRow;

pub mod user {
    use super::*;

    pub async fn get_user(
        client: &Client,
        username: &String,
    ) -> Result<Option<User>, &'static str> {
        let statement = client
            .prepare("SELECT * FROM fruser WHERE username = $1")
            .await
            .unwrap();

        match client.query_opt(&statement, &[username]).await {
            Ok(ref row) => match row {
                Some(ref user) => match User::from_row_ref(user) {
                    Ok(user) => Ok(Some(user)),
                    Err(err) => {
                        eprintln!("{}", err);
                        return Err("Error getting user");
                    }
                },
                None => Ok(None),
            },
            Err(err) => {
                eprintln!("{}", err);
                return Err("Error getting user");
            }
        }
    }

    pub async fn get_users(client: &Client, offset: &i64) -> Result<Vec<SimpleUser>, io::Error> {
        let statement = client
            .prepare("SELECT id, username FROM fruser ORDER BY id LIMIT 10 OFFSET $1")
            .await
            .unwrap();

        let users = client
            .query(&statement, &[offset])
            .await
            .expect("Error getting users")
            .iter()
            .map(|row| SimpleUser::from_row_ref(row).unwrap())
            .collect::<Vec<SimpleUser>>();

        Ok(users)
    }

    pub async fn create_user(
        client: &Client,
        username: &String,
        password: &String,
        native_lang: &String,
    ) -> Result<User, &'static str> {
        let statement = match client
            .prepare(
                "INSERT INTO fruser (username, pass, created_on, native_lang) 
                VALUES ($1, $2, NOW(), $3) RETURNING *",
            )
            .await
        {
            Ok(statement) => statement,
            Err(err) => {
                eprintln!("{}", err);
                return Err("Error creating user");
            }
        };

        match client
            .query_one(&statement, &[username, password, native_lang])
            .await
        {
            Ok(result) => match User::from_row_ref(&result) {
                Ok(user) => Ok(user),
                Err(err) => {
                    eprintln!("{}", err);
                    return Err("Error creating user");
                }
            },
            Err(err) => {
                eprintln!("{}", err);
                return Err("Error creating user");
            }
        }
    }
}

pub mod article {
    use super::*;

    pub async fn get_article(
        client: &Client,
        article_id: &i32,
    ) -> Result<Option<Article>, &'static str> {
        let statement = client
            .prepare("SELECT * FROM article WHERE id = $1")
            .await
            .unwrap();

        match client.query_opt(&statement, &[article_id]).await {
            Ok(ref row_opt) => match row_opt {
                Some(ref row) => match Article::from_row_ref(row) {
                    Ok(article) => Ok(Some(article)),
                    Err(err) => {
                        eprintln!("{}", err);
                        return Err("Error getting article");
                    }
                },
                None => Ok(None),
            },
            Err(err) => {
                eprintln!("{}", err);
                return Err("Error getting article");
            }
        }
    }

    pub async fn get_articles(
        client: &Client,
        offset: &i64,
    ) -> Result<Vec<SimpleArticle>, io::Error> {
        let statement = client
            .prepare(
                "SELECT id, title, author, content_length, created_on, is_system, lang, tags 
                FROM article ORDER BY created_on DESC LIMIT 10 OFFSET $1",
            )
            .await
            .unwrap();

        let articles = client
            .query(&statement, &[offset])
            .await
            .expect("Error getting articles")
            .iter()
            .map(|row| SimpleArticle::from_row_ref(row).unwrap())
            .collect::<Vec<SimpleArticle>>();

        Ok(articles)
    }

    pub async fn create_article(
        client: &Client,
        title: &String,
        author_option: &Option<String>,
        content: &String,
        uploader_id: &i32,
        language: &String,
        tags_option: &Option<Vec<String>>,
        words: &Vec<&str>,
        unique_words: &serde_json::Value,
    ) -> Result<Article, &'static str> {
        let statement = match client
            .prepare("INSERT INTO article (title, author, content, content_length, created_on, is_system, uploader_id, lang, tags, words, unique_words) 
                VALUES ($1, $2, $3, $4, NOW(), $5, $6, $7, $8, $9, $10) RETURNING *")
            .await {
                Ok(statement) => statement,
                Err(err) => {
                    eprintln!("{}", err);
                    return Err("Error creating article");
                }
            };

        let tags: Vec<String> = match tags_option {
            Some(tags) => tags.clone(),
            None => vec![],
        };

        match client
            .query_one(
                &statement,
                &[
                    title,
                    author_option,
                    content,
                    &(content.len() as i32),
                    &((*uploader_id == 1) as bool),
                    uploader_id,
                    language,
                    &tags,
                    words,
                    unique_words,
                ],
            )
            .await
        {
            Ok(result) => match Article::from_row_ref(&result) {
                Ok(article) => Ok(article),
                Err(err) => {
                    eprintln!("{}", err);
                    return Err("Error creating article");
                }
            },
            Err(err) => {
                eprintln!("{}", err);
                return Err("Error creating article");
            }
        }
    }
}
