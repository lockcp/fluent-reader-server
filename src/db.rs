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
