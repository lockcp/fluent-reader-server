use crate::models::*;
use deadpool_postgres::Client;
use std::io;
use tokio_pg_mapper::FromTokioPostgresRow;

pub async fn get_user(client: &Client, username: String) -> Result<User, io::Error> {
    let statement = client
        .prepare("SELECT * FROM fruser WHERE username = $1")
        .await
        .unwrap();

    client
        .query(&statement, &[&username])
        .await
        .expect("Error getting user")
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .next()
        .ok_or(io::Error::new(io::ErrorKind::Other, "Error getting user"))
}

pub async fn get_users(client: &Client) -> Result<Vec<SimpleUser>, io::Error> {
    let statement = client
        .prepare("SELECT id, username FROM fruser ORDER BY id LIMIT 10")
        .await
        .unwrap();

    let users = client
        .query(&statement, &[])
        .await
        .expect("Error getting users")
        .iter()
        .map(|row| SimpleUser::from_row_ref(row).unwrap())
        .collect::<Vec<SimpleUser>>();

    Ok(users)
}

pub async fn create_user(
    client: &Client,
    username: String,
    password: String,
    native_lang: String,
) -> Result<User, &'static str> {
    let statement = match client
        .prepare("INSERT INTO fruser (username, pass, created_on, native_lang) 
            VALUES ($1, $2, NOW(), $3) RETURNING *")
        .await {
            Ok(statement) => statement,
            Err(err) => {
                eprintln!("{}", err);
                return Err("Error creating user");
            }
        };

    match client
        .query(&statement, &[&username, &password, &native_lang])
        .await
    {
        Ok(result) => result
            .iter()
            .map(|row| User::from_row_ref(row).unwrap())
            .next()
            .ok_or("Error creating user"),
        Err(err) => {
            eprintln!("{}", err);
            return Err("Error creating user");
        }
    }
}

pub async fn get_articles(client: &Client, offset: i64) -> Result<Vec<SimpleArticle>, io::Error> {
    let statement = client
        .prepare("SELECT id, title, author, content_length, created_on, is_system 
            FROM article ORDER BY created_on LIMIT 10 OFFSET $1")
        .await
        .unwrap();

    let articles = client
        .query(&statement, &[&offset])
        .await
        .expect("Error getting articles")
        .iter()
        .map(|row| SimpleArticle::from_row_ref(row).unwrap())
        .collect::<Vec<SimpleArticle>>();

    Ok(articles)
}

pub async fn create_article(
    client: &Client,
    title: String,
    author_option: Option<String>,
    content: String,
    uploader_id: i32
) -> Result<Article, &'static str> {
    let statement = match client
        .prepare("INSERT INTO article (title, author, content, content_length, created_on, is_system, uploader_id) 
            VALUES ($1, $2, $3, $4, NOW(), $5, $6) RETURNING *")
        .await {
            Ok(statement) => statement,
            Err(err) => {
                eprintln!("{}", err);
                return Err("Error creating article");
            }
        };

    match client
        .query(
            &statement,
            &[
                &title,
                &author_option,
                &content,
                &(content.len() as i32),
                &((uploader_id == 1) as bool),
                &uploader_id,
            ],
        )
        .await
    {
        Ok(result) => result
            .iter()
            .map(|row| Article::from_row_ref(row).unwrap())
            .next()
            .ok_or("Error creating article"),
        Err(err) => {
            eprintln!("{}", err);
            return Err("Error creating article");
        }
    }
}
