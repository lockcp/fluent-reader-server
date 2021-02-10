use crate::models::*;
use deadpool_postgres::Client;
use futures::future;
use std::io;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_postgres::Statement;

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

    #[inline]
    async fn prepare_user_creation_statements(
        client: &Client,
    ) -> Result<(Statement, Statement), tokio_postgres::error::Error> {
        let insert_user_ft = client.prepare(
            "INSERT INTO fruser (username, pass, created_on, native_lang)
                VALUES ($1, $2, NOW(), $3) RETURNING *",
        );

        let insert_word_data_ft = client
            .prepare(
                r#"
                INSERT INTO user_word_data (fruser_id, word_status_data, word_definition_data)
                VALUES
                    (
                        $1,
                        '{ "en": { "learning": {}, "known": {} }, "zh": { "learning": {}, "known": {} } }',
                        '{ "en": {}, "zh": {} }'
                    )
            "#
            );

        future::try_join(insert_user_ft, insert_word_data_ft).await
    }

    pub async fn create_user(
        client: &Client,
        username: &String,
        password: &String,
        native_lang: &String,
    ) -> Result<User, &'static str> {
        let prepare_result = prepare_user_creation_statements(client).await;

        if let Err(err) = prepare_result {
            eprintln!("{}", err);
            return Err("Error creating user");
        }

        let (insert_user, insert_word_data) = prepare_result.unwrap();

        let insert_user_result: Result<User, &'static str> = match client
            .query_one(&insert_user, &[username, password, native_lang])
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
        };

        let user = insert_user_result.unwrap();

        let insert_word_data_result = client.query_opt(&insert_word_data, &[&user.id]).await;

        if let Err(err) = insert_word_data_result {
            // TODO:
            // attempt to remove the already created user
            // if the insert_word_data statement fails
            // so that login is not still successful if the
            // user creation succeeded but this part did not

            eprintln!("{}", err);
            return Err("Error creating user");
        }

        Ok(user)
    }

    pub mod word_data {
        use super::*;

        pub async fn get_user_word_data(
            client: &Client,
            user_id: &i32,
        ) -> Result<UserWordData, &'static str> {
            let statement = match client
                .prepare(
                    r#"
                    SELECT word_status_data, word_definition_data
                        FROM user_word_data
                    WHERE fruser_id = $1
                "#,
                )
                .await
            {
                Ok(statement) => statement,
                Err(err) => {
                    eprintln!("{}", err);
                    return Err("Error getting word data");
                }
            };

            match client.query_one(&statement, &[user_id]).await {
                Ok(result) => match UserWordData::from_row_ref(&result) {
                    Ok(word_data) => Ok(word_data),
                    Err(err) => {
                        eprintln!("{}", err);
                        Err("Error getting word data")
                    }
                },
                Err(err) => {
                    eprintln!("{}", err);
                    Err("Error getting word data")
                }
            }
        }

        pub async fn update_word_status(
            client: &Client,
            user_id: &i32,
            lang: &String,
            word: &String,
            new_status: &String,
        ) -> Result<(), &'static str> {
            let statement_result = match &new_status[..] {
                "known" => {
                    client
                        .prepare_typed(
                            r#"
                            UPDATE user_word_data
                            SET word_status_data = 
                                jsonb_set(
                                    (word_status_data #- 
                                        CAST(FORMAT('{%s, learning, %s}', $1, $2) AS TEXT[])
                                    ), 
                                    CAST(FORMAT('{%s, known, %s}', $1, $2) AS TEXT[]),
                                    '1'
                                )
                            WHERE fruser_id = $3;
                        "#,
                            &[
                                tokio_postgres::types::Type::TEXT,
                                tokio_postgres::types::Type::TEXT,
                            ],
                        )
                        .await
                }
                "learning" => {
                    client
                        .prepare_typed(
                            r#"
                            UPDATE user_word_data
                            SET word_status_data = 
                                jsonb_set(
                                    (word_status_data #- 
                                        CAST(FORMAT('{%s, known, %s}', $1, $2) AS TEXT[])
                                    ), 
                                    CAST(FORMAT('{%s, learning, %s}', $1, $2) AS TEXT[]),
                                    '1'
                                )
                            WHERE fruser_id = $3;
                        "#,
                            &[
                                tokio_postgres::types::Type::TEXT,
                                tokio_postgres::types::Type::TEXT,
                            ],
                        )
                        .await
                }
                "new" => {
                    client
                        .prepare_typed(
                            r#"
                            UPDATE user_word_data
                            SET word_status_data = word_status_data 
                                #- CAST(FORMAT('{%s, known, %s}', $1, $2) AS TEXT[])
                                #- CAST(FORMAT('{%s, learning, %s}', $1, $2) AS TEXT[])
                            WHERE fruser_id = $3
                        "#,
                            &[
                                tokio_postgres::types::Type::TEXT,
                                tokio_postgres::types::Type::TEXT,
                            ],
                        )
                        .await
                }
                _ => return Err("Invalid status"),
            };

            let statement = match statement_result {
                Ok(statement) => statement,
                Err(err) => {
                    eprintln!("{}", err);
                    return Err("Error updating word status");
                }
            };

            match client.execute(&statement, &[lang, word, user_id]).await {
                Ok(_) => Ok(()),
                Err(err) => {
                    eprintln!("{}", err);
                    return Err("Error updating word status");
                }
            }
        }

        pub async fn update_word_definition(
            client: &Client,
            user_id: &i32,
            lang: &String,
            word: &String,
            definition: &String,
        ) -> Result<(), &'static str> {
            let statement = match client
                .prepare_typed(
                    r#"
                        UPDATE user_word_data
                        SET word_definition_data = 
                            jsonb_set(
                                word_definition_data, 
                                CAST(FORMAT('{ %s, %s }', $1, $2) AS TEXT[]), 
                                FORMAT('"%s"', $3)::jsonb
                            )
                        WHERE fruser_id = $4
                "#,
                    &[
                        tokio_postgres::types::Type::TEXT,
                        tokio_postgres::types::Type::TEXT,
                        tokio_postgres::types::Type::TEXT,
                    ],
                )
                .await
            {
                Ok(statement) => statement,
                Err(err) => {
                    eprintln!("{}", err);
                    return Err("Error updating word definition");
                }
            };

            match client
                .execute(&statement, &[lang, word, definition, user_id])
                .await
            {
                Ok(_) => Ok(()),
                Err(err) => {
                    eprintln!("{}", err);
                    Err("Error updating word definition")
                }
            }
        }
    }
}

pub mod article {
    use super::*;

    // WARNING: THIS DOES NOT MAKE ANY CHECKS FOR SYSTEM/NON-SYSTEM, PRIVATE/NON-PRIVATE
    //
    // PLEASE CONSIDER THE FOLLOWING METHODS INSTEAD
    //
    // SINGLE ARTICLE METHODS
    // FOR GETTING A SYSTEM ARTICLE, PLEASE USE get_system_article
    // FOR GETTING A USER ARTICLE, PLEASE USE get_user_article
    //
    // ARTICLE LIST METHODS
    // FOR GETTING SYSTEM ARTICLES, PLEASE USE get_system_article_list
    // FOR GETTING ARTICLES A USER HAS SAVED, PLEASE USE get_user_saved_article_list
    // FOR GETTING ARTICLES A USER UPLOADED, PLEASE USE get_user_uploaded_article_list
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
                        Err("Error getting article")
                    }
                },
                None => Ok(None),
            },
            Err(err) => {
                eprintln!("{}", err);
                Err("Error getting article")
            }
        }
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
        is_private: &bool,
    ) -> Result<Article, &'static str> {
        let statement = match client
            .prepare(
                r#"
                INSERT INTO article 
                        (title, author, content, content_length, 
                            created_on, is_system, uploader_id, lang, 
                            tags, words, unique_words, is_private) 
                VALUES ($1, $2, $3, $4, NOW(), $5, $6, $7, $8, $9, $10, $11) 
                RETURNING *
            "#,
            )
            .await
        {
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
                    is_private,
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

    pub mod system {
        use super::*;

        pub async fn get_system_article(
            client: &Client,
            article_id: &i32,
        ) -> Result<Option<Article>, &'static str> {
            let statement = client
                .prepare("SELECT * FROM article WHERE id = $1 AND is_system = true")
                .await
                .unwrap();

            match client.query_opt(&statement, &[article_id]).await {
                Ok(ref row_opt) => match row_opt {
                    Some(ref row) => match Article::from_row_ref(row) {
                        Ok(article) => Ok(Some(article)),
                        Err(err) => {
                            eprintln!("{}", err);
                            Err("Error getting article")
                        }
                    },
                    None => Ok(None),
                },
                Err(err) => {
                    eprintln!("{}", err);
                    Err("Error getting article")
                }
            }
        }

        pub async fn get_system_article_list(
            client: &Client,
            offset: &i64,
        ) -> Result<Vec<SimpleArticle>, io::Error> {
            let statement = client
                .prepare(
                    r#"
                    SELECT id, title, author, content_length, created_on, is_system, lang, tags 
                        FROM article 
                    WHERE is_system = true AND is_private = false
                    ORDER BY created_on DESC 
                    LIMIT 10 
                    OFFSET $1
                "#,
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
    }

    pub mod user {
        use super::*;

        pub async fn get_user_article(
            client: &Client,
            article_id: &i32,
            user_id: &i32,
        ) -> Result<Option<Article>, &'static str> {
            let statement = client
                .prepare(
                    r#"
                    SELECT * 
                        FROM article 
                    WHERE 
                        id = $1 AND 
                        (NOT is_private OR uploader_id = $2)
                "#,
                )
                .await
                .unwrap();

            match client.query_opt(&statement, &[article_id, user_id]).await {
                Ok(ref row_opt) => match row_opt {
                    Some(ref row) => match Article::from_row_ref(row) {
                        Ok(article) => Ok(Some(article)),
                        Err(err) => {
                            eprintln!("{}", err);
                            Err("Error getting article")
                        }
                    },
                    None => Ok(None),
                },
                Err(err) => {
                    eprintln!("{}", err);
                    Err("Error getting article")
                }
            }
        }

        pub async fn user_save_article(
            client: &Client,
            user_id: &i32,
            article_id: &i32,
        ) -> Result<(), &'static str> {
            let statement = client
                .prepare(
                    r#"
                    INSERT INTO saved_article (fruser_id, article_id, saved_on)
                    VALUES ($1, $2, NOW())
                "#,
                )
                .await
                .unwrap();

            match client.execute(&statement, &[user_id, article_id]).await {
                Ok(_) => Ok(()),
                Err(err) => {
                    eprintln!("{}", err);
                    Err("Error saving article")
                }
            }
        }

        pub async fn user_delete_saved_article(
            client: &Client,
            user_id: &i32,
            article_id: &i32,
        ) -> Result<(), &'static str> {
            let statement = client
                .prepare(
                    r#"
                    DELETE FROM saved_article
                    WHERE fruser_id = $1 AND article_id = $2
                "#,
                )
                .await
                .unwrap();

            match client.execute(&statement, &[user_id, article_id]).await {
                Ok(_) => Ok(()),
                Err(err) => {
                    eprintln!("{}", err);
                    return Err("Failed to delete saved article");
                }
            }
        }

        pub async fn get_user_saved_article_list(
            client: &Client,
            user_id: &i32,
            offset: &i64,
        ) -> Result<Vec<SimpleArticle>, io::Error> {
            let statement = client
                .prepare(
                    r#"
                    SELECT id, title, author, content_length, created_on, is_system, lang, tags 
                        FROM saved_article AS s
                        INNER JOIN article AS a
                            ON a.id = s.article_id
                    WHERE s.fruser_id = $1 AND (NOT a.is_private OR a.uploader_id = $1)
                    ORDER BY s.saved_on DESC
                    LIMIT 10 
                    OFFSET $2
                "#,
                )
                .await
                .unwrap();

            let articles = client
                .query(&statement, &[user_id, offset])
                .await
                .expect("Error getting articles")
                .iter()
                .map(|row| SimpleArticle::from_row_ref(row).unwrap())
                .collect::<Vec<SimpleArticle>>();

            Ok(articles)
        }

        pub async fn get_user_uploaded_article_list(
            client: &Client,
            user_id: &i32,
            offset: &i64,
        ) -> Result<Vec<SimpleArticle>, io::Error> {
            let statement = client
                .prepare(
                    r#"
                    SELECT id, title, author, content_length, created_on, is_system, lang, tags 
                        FROM article 
                    WHERE uploader_id = $1
                    ORDER BY created_on DESC 
                    LIMIT 10 
                    OFFSET $2
                "#,
                )
                .await
                .unwrap();

            let articles = client
                .query(&statement, &[user_id, offset])
                .await
                .expect("Error getting articles")
                .iter()
                .map(|row| SimpleArticle::from_row_ref(row).unwrap())
                .collect::<Vec<SimpleArticle>>();

            Ok(articles)
        }
    }
}
