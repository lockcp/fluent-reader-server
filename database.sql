/*
    Database Creation
*/

\c postgres;
DROP DATABASE fr;
CREATE DATABASE fr WITH ENCODING 'UTF8';
\c fr;

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
    words VARCHAR[] NOT NULL,
    unique_words JSONB NOT NULL,
    created_on TIMESTAMP NOT NULL,
    is_system BOOLEAN NOT NULL,
    is_private BOOLEAN NOT NULL,
    uploader_id INTEGER NOT NULL,
    FOREIGN KEY (uploader_id) REFERENCES fruser(id),
    lang VARCHAR(6) NOT NULL,
    tags VARCHAR(50)[] NOT NULL
);

CREATE INDEX article_content_index ON article USING pgroonga (content);
CREATE INDEX article_tag_index ON article USING pgroonga (tags);
CREATE INDEX article_lang_index ON article USING HASH (lang);

CREATE TABLE saved_article (
    fruser_id INTEGER NOT NULL,
    FOREIGN KEY (fruser_id) REFERENCES fruser(id),
    article_id INTEGER NOT NULL,
    FOREIGN KEY (article_id) REFERENCES article(id),
    saved_on TIMESTAMP NOT NULL,
    PRIMARY KEY(fruser_id, article_id)
);
