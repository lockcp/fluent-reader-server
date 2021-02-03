use crate::models::User;
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
        .collect::<Vec<User>>()
        .pop()
        .ok_or(io::Error::new(io::ErrorKind::Other, "Error getting user"))
}

pub async fn get_users(client: &Client) -> Result<Vec<User>, io::Error> {
    let statement = client
        .prepare("SELECT * FROM fruser ORDER BY id LIMIT 10")
        .await
        .unwrap();

    let users = client
        .query(&statement, &[])
        .await
        .expect("Error getting users")
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect::<Vec<User>>();

    Ok(users)
}

pub async fn create_user(
    client: &Client,
    username: String,
    password: String,
    native_lang: String,
) -> Result<User, io::Error> {
    let statement = client
        .prepare("INSERT INTO fruser (username, pass, created_on, native_lang) VALUES ($1, $2, NOW(), $3) RETURNING *")
        .await
        .unwrap();

    client
        .query(&statement, &[&username, &password, &native_lang])
        .await
        .expect("Error creating user")
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect::<Vec<User>>()
        .pop()
        .ok_or(io::Error::new(io::ErrorKind::Other, "Error creating user"))
}

// pub async fn get_todos(client: &Client) -> Result<Vec<TodoList>, io::Error> {
//     let statement = client
//         .prepare("SELECT * FROM todo_list ORDER BY id desc LIMIT 10")
//         .await
//         .unwrap();

//     let todos = client
//         .query(&statement, &[])
//         .await
//         .expect("Error getting todo lists")
//         .iter()
//         .map(|row| TodoList::from_row_ref(row).unwrap())
//         .collect::<Vec<TodoList>>();

//     Ok(todos)
// }

// pub async fn get_items(client: &Client, list_id: i32) -> Result<Vec<TodoItem>, io::Error> {
//     let statement = client
//         .prepare("SELECT * FROM todo_item where list_id = $1 ORDER BY id")
//         .await
//         .unwrap();

//     let items = client
//         .query(&statement, &[&list_id])
//         .await
//         .expect("Error getting todo lists")
//         .iter()
//         .map(|row| TodoItem::from_row_ref(row).unwrap())
//         .collect::<Vec<TodoItem>>();

//     Ok(items)
// }

// pub async fn create_todo(client: &Client, title: String) -> Result<TodoList, io::Error> {
//     let statement = client
//         .prepare("INSERT INTO todo_list (title) VALUES ($1) RETURNING id, title")
//         .await
//         .unwrap();

//     client
//         .query(&statement, &[&title])
//         .await
//         .expect("Error creating todo list")
//         .iter()
//         .map(|row| TodoList::from_row_ref(row).unwrap())
//         .collect::<Vec<TodoList>>()
//         .pop()
//         .ok_or(io::Error::new(
//             io::ErrorKind::Other,
//             "Error creating todo list",
//         ))
// }

// pub async fn check_item(client: &Client, list_id: i32, item_id: i32) -> Result<(), io::Error> {
//     let statement = client.prepare("UPDATE todo_item SET checked = true WHERE list_id = $1 AND id = $2 AND checked = false").await.unwrap();

//     let result = client
//         .execute(&statement, &[&list_id, &item_id])
//         .await
//         .expect("Error checking todo item");

//     match result {
//         ref updated if *updated == 1 => Ok(()),
//         _ => Err(io::Error::new(
//             io::ErrorKind::Other,
//             "Failed to check todo item",
//         )),
//     }
// }
