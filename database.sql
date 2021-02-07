/*
    Database Creation
*/

CREATE DATABASE fr WITH ENCODING 'UTF8';

/* 
    Table Creation
*/

DROP TABLE IF EXISTS user_word_data;
DROP TABLE IF EXISTS saved_articles;
DROP TABLE IF EXISTS fruser;
DROP TABLE IF EXISTS article;

SET timezone = 'PRC';

CREATE TABLE fruser (
    id SERIAL PRIMARY KEY,
    username VARCHAR(100) UNIQUE NOT NULL,
    pass VARCHAR(128) NOT NULL,
    created_on TIMESTAMP NOT NULL,
    native_lang VARCHAR(6)
);

CREATE TABLE user_word_data (
    fruser_id INTEGER UNIQUE NOT NULL,
    FOREIGN KEY (fruser_id) REFERENCES fruser(id),
    word_status_data JSONB NOT NULL,
    word_definition_data JSONB NOT NULL
);

CREATE TABLE article (
    id SERIAL PRIMARY KEY,
    title VARCHAR(250) NOT NULL,
    author VARCHAR,
    content VARCHAR NOT NULL,
    content_length INTEGER NOT NULL,
    created_on TIMESTAMP NOT NULL,
    is_system BOOLEAN NOT NULL,
    uploader_id INTEGER NOT NULL,
    lang VARCHAR(6) NOT NULL,
    FOREIGN KEY (uploader_id) REFERENCES fruser(id)
);

CREATE TABLE saved_articles (
    fruser_id INTEGER NOT NULL,
    FOREIGN KEY (fruser_id) REFERENCES fruser(id),
    article_id INTEGER NOT NULL,
    FOREIGN KEY (article_id) REFERENCES article(id)
);
