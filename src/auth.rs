use crate::app_config::AppConfig;
use crate::models::*;

use actix_web::{web, HttpRequest};
use argon2::{self, Config as ArgonConfig};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

pub fn handle_pass_hash(
    config: web::Data<AppConfig>,
    json: &mut web::Json<RegisterRequest>,
) -> Result<(), &'static str> {
    let mut hash_config = ArgonConfig::default();
    hash_config.hash_length = 10;

    let hashed = match argon2::hash_encoded(
        json.password.as_bytes(),
        config.get_ref().server.salt.as_bytes(),
        &hash_config,
    ) {
        Ok(hash_result) => hash_result,
        Err(_) => return Err("Failed to hash password"),
    };

    // overwrite the plaintext password memory before dropping it
    // by reassigning the new hashed password string to it
    let password_bytes = unsafe { json.password.as_bytes_mut() };
    for i in 0..password_bytes.len() {
        password_bytes[i] = 0;
    }

    json.password = hashed;

    Ok(())
}

pub fn attempt_user_login(
    config: web::Data<AppConfig>,
    json: web::Json<LoginRequest>,
    user: User,
) -> Result<String, &'static str> {
    let matches = argon2::verify_encoded(&user.pass, json.password.as_bytes()).unwrap();

    if matches {
        let expiration = Utc::now()
            .checked_add_signed(chrono::Duration::seconds(
                config.get_ref().server.token_time,
            ))
            .expect("valid timestamp")
            .timestamp();

        let claims = TokenClaims {
            user: ClaimsUser::from_user(&user),
            exp: expiration as usize,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(config.get_ref().server.secret.as_bytes()),
        )
        .unwrap();

        return Ok(token);
    } else {
        return Err("Password doesn't match");
    }
}

pub fn attempt_token_auth(
    req: HttpRequest,
    config: web::Data<AppConfig>,
) -> Result<ClaimsUser, &'static str> {
    if let Some(header_value) = req.headers().get("authorization") {
        let header_str = header_value.to_str().unwrap_or("");
        if !header_str.starts_with("Bearer ") {
            return Err("Missing bearer");
        }

        let mut split_iter = header_str.split(" ");
        // iterate over "Bearer" string
        split_iter.next();

        let token = split_iter.next().unwrap();
        match decode::<ClaimsUser>(
            &token,
            &DecodingKey::from_secret(config.get_ref().server.secret.as_bytes()),
            &Validation::default(),
        ) {
            Ok(token_data) => Ok(token_data.claims),
            Err(err) => {
                eprintln!("{}", err);
                Err("Decoding failed")
            }
        }
    } else {
        return Err("Missing authorization header");
    }
}
