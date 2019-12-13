use crate::error::AuthError;
use argon2::{self, Config};
use chrono::{Duration, Utc};
use jsonwebtoken as jwt;
use jwt::{decode, encode, Header, Validation};
use serde::{Serialize, Deserialize};

pub static AUTH_APP: &'static str = "Auth App";

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    id: Option<i32>,
    pub email: String,
    pub username: String,
    pub password: String,
}

impl User {
    pub fn new(email: &str, username: &str, password: &str) -> User {
        User {
            id: None,
            email: email.to_owned(),
            username: username.to_owned(),
            password: password.to_owned(),
        }
    }

    pub fn set_password(&mut self, password: &str) {
        self.password = password.to_owned();
    }

    pub fn is_valid_email(&self, context: &str) -> Result<(), AuthError> {
        if self.email == "" {
            return Err(AuthError::new(
                &format!("{}{}", context, "Email"),
                "Please enter your email.",
                "",
                400,
            ));
        }
        Ok(())
    }

    pub fn is_valid_username(&self) -> Result<(), AuthError> {
        if self.username == "" {
            return Err(AuthError::new(
                "username",
                "Please enter a username.",
                "",
                400,
            ));
        }
        Ok(())
    }

    pub fn is_valid_password(&self, context: &str) -> Result<(), AuthError> {
        if self.password == "" {
            return Err(AuthError::new(
                &format!("{}{}", context, "Password"),
                "Please enter a password.",
                "",
                400,
            ));
        }
        Ok(())
    }

    pub fn is_valid_signup(&self) -> Result<(), AuthError> {
        self.is_valid_email("signup")?;
        self.is_valid_username()?;
        self.is_valid_password("signup")
    }

    pub fn is_valid_signin(&self) -> Result<(), AuthError> {
        self.is_valid_email("signin")?;
        self.is_valid_password("signin")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // subject
    iss: String,     // issuer
    exp: usize,      // expiration (time)
    nbf: usize,      // not before (time)
}

pub enum TokenDuration {
    Weeks2,
    Hours24,
    Minutes5,
}

impl Claims {
    pub fn new(username: String, duration: TokenDuration) -> Claims {
        let exp = match duration {
            TokenDuration::Weeks2 => (Utc::now() + Duration::weeks(2)).timestamp() as usize,
            TokenDuration::Hours24 => (Utc::now() + Duration::hours(24)).timestamp() as usize,
            TokenDuration::Minutes5 => (Utc::now() + Duration::minutes(5)).timestamp() as usize,
        };

        Claims {
            sub: username,
            iss: AUTH_APP.to_owned(),
            nbf: Utc::now().timestamp() as usize,
            exp,
        }
    }
}

#[derive(Clone)]
pub struct Auth {
    jwt_secret: String,
    salt: String,
}

impl Auth {
    pub fn new(jwt_secret: String, salt: String) -> Auth {
        Auth { jwt_secret, salt }
    }

    pub fn create_hash(&self, password: &str) -> String {
        let config = Config::default();
        argon2::hash_encoded(password.as_bytes(), self.salt.as_bytes(), &config).expect("hash failed")
    }

    pub fn verify_hash(hash: String, password: String) -> bool {
        match argon2::verify_encoded(&hash, password.as_bytes()) {
            Ok(value) => value,
            Err(_) => false,
        }
    }

    pub fn create_token(
        &self,
        username: &str,
        duration: TokenDuration,
    ) -> Result<String, AuthError> {
        let claims = Claims::new(username.to_owned(), duration);

        match encode(&Header::default(), &claims, self.jwt_secret.as_bytes()) {
            Ok(token) => Ok(token),
            Err(err) => Err(AuthError::internal_error(&err.to_string())),
        }
    }

    pub fn decode_token(&self, token: &str) -> Result<Claims, AuthError> {
        let validation = Validation {
            iss: Some(AUTH_APP.to_owned()),
            ..Default::default()
        };
        match decode::<Claims>(&token, self.jwt_secret.as_bytes(), &validation) {
            Ok(token_data) => Ok(token_data.claims),
            Err(err) => Err(AuthError::new(
                "auth",
                "Please log in or sign up to access this resource.",
                &err.to_string(),
                401,
            )),
        }
    }
}
