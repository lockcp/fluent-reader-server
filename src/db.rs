use crate::models;
use deadpool_postgres::Client;
use futures::future;
use std::io;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_postgres::types;
use tokio_postgres::Statement;
use types::{ToSql, Type};

#[inline]
fn get_article_query(from_clause: &str, where_clause: &str, order_by_clause: &str) -> String {
    format!(
        r#"
            SELECT 
                id, title, author, content_length, created_on, is_system, lang, tags
                FROM {} 
            WHERE 
                COALESCE(lang = $1, TRUE) AND
                COALESCE(title &@~ $2, TRUE)
                {}
            ORDER BY {} DESC 
            LIMIT 10 
            OFFSET $3
        "#,
        from_clause, where_clause, order_by_clause
    )
}

pub mod user {
    use super::*;

    pub async fn get_user(
        client: &Client,
        username: &String,
    ) -> Result<Option<models::db::User>, &'static str> {
        let statement = client
            .prepare("SELECT * FROM fruser WHERE username = $1")
            .await
            .unwrap();

        match client.query_opt(&statement, &[username]).await {
            Ok(ref row) => match row {
                Some(ref user) => match models::db::User::from_row_ref(user) {
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

    pub async fn get_user_by_id(
        client: &Client,
        user_id: &i32,
    ) -> Result<Option<models::db::User>, &'static str> {
        let statement = client
            .prepare("SELECT * FROM fruser WHERE id = $1")
            .await
            .unwrap();

        match client.query_opt(&statement, &[user_id]).await {
            Ok(ref row) => match row {
                Some(ref user) => match models::db::User::from_row_ref(user) {
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

    fn extract_opt_inc_param<'a, T, U>(
        params: &mut [&'a (dyn ToSql + Sync); 6],
        current_param: &mut usize,
        opt: &'a Option<T>,
        name: &str,
        func: &mut U,
    ) where
        T: ToSql + Sync,
        U: FnMut(&str, &usize),
    {
        if let Some(val) = opt {
            params[*current_param] = val;
            (*func)(name, current_param);
            *current_param = *current_param + 1;
        }
    }

    pub async fn update_user(
        client: &Client,
        user_id: &i32,
        update: &models::db::UpdateUserOpt,
    ) -> Result<(), &'static str> {
        let mut params: [&'_ (dyn ToSql + Sync); 6] = [&0; 6];
        let mut current_param: usize = 0;

        let mut update_statements: Vec<String> = vec![];

        let mut add_to_statement = |name: &str, curr: &usize| {
            update_statements.push(format!(" {} = ${}", name, curr + 1));
        };

        extract_opt_inc_param(
            &mut params,
            &mut current_param,
            &update.username,
            "username",
            &mut add_to_statement,
        );
        extract_opt_inc_param(
            &mut params,
            &mut current_param,
            &update.pass,
            "password",
            &mut add_to_statement,
        );
        extract_opt_inc_param(
            &mut params,
            &mut current_param,
            &update.study_lang,
            "study_lang",
            &mut add_to_statement,
        );
        extract_opt_inc_param(
            &mut params,
            &mut current_param,
            &update.display_lang,
            "display_lang",
            &mut add_to_statement,
        );
        extract_opt_inc_param(
            &mut params,
            &mut current_param,
            &update.refresh_token,
            "refresh_token",
            &mut add_to_statement,
        );

        let set_clause = update_statements.join(",");

        params[current_param] = user_id;
        current_param = current_param + 1;

        let statement = client
            .prepare(
                &format!(
                    r#"
                        UPDATE fruser
                        SET {}
                        WHERE id = ${}
                    "#,
                    set_clause, current_param
                )[..],
            )
            .await
            .unwrap();

        match client.execute(&statement, &params[..current_param]).await {
            Ok(_) => Ok(()),
            Err(err) => {
                eprintln!("{}", err);
                return Err("Error updating user");
            }
        }
    }

    pub async fn get_users(
        client: &Client,
        offset: &i64,
    ) -> Result<Vec<models::db::SimpleUser>, io::Error> {
        let statement = client
            .prepare("SELECT id, username FROM fruser ORDER BY id LIMIT 10 OFFSET $1")
            .await
            .unwrap();

        let users = client
            .query(&statement, &[offset])
            .await
            .expect("Error getting users")
            .iter()
            .map(|row| models::db::SimpleUser::from_row_ref(row).unwrap())
            .collect::<Vec<models::db::SimpleUser>>();

        Ok(users)
    }

    #[inline]
    async fn prepare_user_creation_statements(
        client: &Client,
    ) -> Result<(Statement, Statement), tokio_postgres::error::Error> {
        let insert_user_ft = client.prepare(
            "INSERT INTO fruser (username, pass, created_on, study_lang, display_lang, refresh_token)
                VALUES ($1, $2, NOW(), $3, $4, $5) RETURNING *",
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
        study_lang: &String,
        display_lang: &String,
    ) -> Result<models::db::User, &'static str> {
        let prepare_result = prepare_user_creation_statements(client).await;

        if let Err(err) = prepare_result {
            eprintln!("{}", err);
            return Err("Error creating user");
        }

        let (insert_user, insert_word_data) = prepare_result.unwrap();

        let insert_user_result: Result<models::db::User, &'static str> = match client
            .query_one(
                &insert_user,
                &[username, password, study_lang, display_lang, &""],
            )
            .await
        {
            Ok(result) => match models::db::User::from_row_ref(&result) {
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
        ) -> Result<models::db::UserWordData, &'static str> {
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
                Ok(result) => match models::db::UserWordData::from_row_ref(&result) {
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
                            &[Type::TEXT, Type::TEXT],
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
                            &[Type::TEXT, Type::TEXT],
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
                            &[Type::TEXT, Type::TEXT],
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
                    &[Type::TEXT, Type::TEXT, Type::TEXT],
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
    ) -> Result<Option<models::db::Article>, &'static str> {
        let statement = client
            .prepare("SELECT * FROM article WHERE id = $1")
            .await
            .unwrap();

        match client.query_opt(&statement, &[article_id]).await {
            Ok(ref row_opt) => match row_opt {
                Some(ref row) => match models::db::Article::from_row_ref(row) {
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
        content_length: &usize,
        uploader_id: &i32,
        language: &String,
        tags_option: &Option<Vec<String>>,
        words: &Vec<&str>,
        unique_words: &serde_json::Value,
        is_private: &bool,
    ) -> Result<models::db::Article, &'static str> {
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
                    &(*content_length as i32),
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
            Ok(result) => match models::db::Article::from_row_ref(&result) {
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
        ) -> Result<Option<models::db::Article>, &'static str> {
            let statement = client
                .prepare("SELECT * FROM article WHERE id = $1 AND is_system = true")
                .await
                .unwrap();

            match client.query_opt(&statement, &[article_id]).await {
                Ok(ref row_opt) => match row_opt {
                    Some(ref row) => match models::db::Article::from_row_ref(row) {
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
            lang: &Option<String>,
            search: &Option<String>,
        ) -> Result<Vec<models::db::SimpleArticle>, io::Error> {
            let order_by_str = if search.is_some() {
                "pgroonga_score(tableoid, ctid)"
            } else {
                "created_on"
            };

            let statement = client
                .prepare_typed(
                    &get_article_query(
                        "article",
                        r#"
                            AND 
                            is_system = true AND 
                            is_private = false
                        "#,
                        order_by_str,
                    )[..],
                    &[Type::TEXT, Type::TEXT],
                )
                .await
                .unwrap();

            let articles = client
                .query(&statement, &[lang, search, offset])
                .await
                .expect("Error getting articles")
                .iter()
                .map(|row| models::db::SimpleArticle::from_row_ref(row).unwrap())
                .collect::<Vec<models::db::SimpleArticle>>();

            Ok(articles)
        }
    }

    pub mod user {
        use super::*;

        pub async fn get_user_article(
            client: &Client,
            article_id: &i32,
            user_id: &i32,
        ) -> Result<Option<models::db::Article>, &'static str> {
            let statement = client
                .prepare(
                    r#"
                    SELECT * 
                        FROM article 
                    WHERE 
                        id = $1 AND 
                        (NOT is_private OR uploader_id = $2) AND
                        is_system = false
                "#,
                )
                .await
                .unwrap();

            match client.query_opt(&statement, &[article_id, user_id]).await {
                Ok(ref row_opt) => match row_opt {
                    Some(ref row) => match models::db::Article::from_row_ref(row) {
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
            let insert_statement = client
                .prepare(
                    r#"
                    INSERT INTO saved_article (fruser_id, article_id, saved_on)
                    VALUES 
                    (
                        $1, 
                        (
                            SELECT id 
                            FROM article
                            WHERE 
                                id = $2 AND
                                (
                                    NOT is_private OR 
                                    uploader_id = $1
                                )
                        ), 
                        NOW()
                    )
                "#,
                )
                .await
                .unwrap();

            match client
                .execute(&insert_statement, &[user_id, article_id])
                .await
            {
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
            lang: &Option<String>,
            search: &Option<String>,
        ) -> Result<Vec<models::db::SimpleArticle>, io::Error> {
            let order_by_str = if search.is_some() {
                "pgroonga_score(a.tableoid, a.ctid)"
            } else {
                "s.saved_on"
            };

            let statement = client
                .prepare_typed(
                    &get_article_query(
                        r#"
                            saved_article AS s
                            INNER JOIN article AS a
                                ON a.id = s.article_id  
                        "#,
                        r#"
                            AND
                            s.fruser_id = $4 AND 
                            (NOT a.is_private OR a.uploader_id = $4)
                        "#,
                        order_by_str,
                    )[..],
                    &[Type::TEXT, Type::TEXT],
                )
                .await
                .unwrap();

            let articles = client
                .query(&statement, &[lang, search, offset, user_id])
                .await
                .expect("Error getting articles")
                .iter()
                .map(|row| models::db::SimpleArticle::from_row_ref(row).unwrap())
                .collect::<Vec<models::db::SimpleArticle>>();

            Ok(articles)
        }

        pub async fn get_user_uploaded_article_list(
            client: &Client,
            req_user_id: &i32,
            want_user_id: &i32,
            offset: &i64,
            lang: &Option<String>,
            search: &Option<String>,
        ) -> Result<Vec<models::db::SimpleArticle>, io::Error> {
            let order_by_str = if search.is_some() {
                "pgroonga_score(tableoid, ctid)"
            } else {
                "created_on"
            };

            let statement = client
                .prepare_typed(
                    &get_article_query(
                        "article",
                        r#"
                            AND
                            uploader_id = $4 AND 
                            ($5 OR NOT is_private)
                        "#,
                        order_by_str,
                    )[..],
                    &[Type::TEXT, Type::TEXT],
                )
                .await
                .unwrap();

            let articles = client
                .query(
                    &statement,
                    &[
                        lang,
                        search,
                        offset,
                        want_user_id,
                        &(want_user_id == req_user_id),
                    ],
                )
                .await
                .expect("Error getting articles")
                .iter()
                .map(|row| models::db::SimpleArticle::from_row_ref(row).unwrap())
                .collect::<Vec<models::db::SimpleArticle>>();

            Ok(articles)
        }

        pub async fn get_all_user_uploaded_article_list(
            client: &Client,
            req_user_id: &i32,
            offset: &i64,
            lang: &Option<String>,
            search: &Option<String>,
        ) -> Result<Vec<models::db::SimpleArticle>, io::Error> {
            let order_by_str = if search.is_some() {
                "pgroonga_score(tableoid, ctid)"
            } else {
                "created_on"
            };

            let statement = client
                .prepare(
                    &format!(
                        r#"
                            SELECT id, title, author, content_length, created_on, is_system, lang, tags 
                                FROM article 
                            WHERE 
                                is_system = false AND
                                (NOT is_private OR uploader_id = $1) AND
                                COALESCE(lang = $2, TRUE) AND
                                COALESCE(title &@~ $3, TRUE)
                            ORDER BY {} DESC 
                            LIMIT 10 
                            OFFSET $4
                        "#, 
                        order_by_str
                    )[..]
                )
                .await
                .unwrap();

            let articles = client
                .query(&statement, &[req_user_id, lang, search, offset])
                .await
                .expect("Error getting articles")
                .iter()
                .map(|row| models::db::SimpleArticle::from_row_ref(row).unwrap())
                .collect::<Vec<models::db::SimpleArticle>>();

            Ok(articles)
        }
    }
}
