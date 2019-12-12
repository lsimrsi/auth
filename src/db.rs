use crate::auth::{User, Auth, ClaimsDuration};
use crate::auth_error::AuthError;
use r2d2_postgres::r2d2;
use r2d2_postgres::PostgresConnectionManager;
// use std::sync::Arc;

#[derive(Clone)]
pub struct Db {
    pub pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager>
}

impl Db {
    pub fn new(url: &str) -> Db {
        let manager = PostgresConnectionManager::new(url, r2d2_postgres::TlsMode::None).unwrap();
        let pool = r2d2::Pool::builder().max_size(3).build(manager).unwrap();
        Db {
            pool
        }
    }

    pub fn insert_user(&self, user: &User, auth: &Auth) -> Result<u64, AuthError> {
        let hashed_password = auth.create_hash(&user.password.clone());

        let conn = self.pool.get()?;
        match conn.execute(
            "INSERT INTO users (email, username, password) VALUES ($1, $2, $3)",
            &[&user.email, &user.username, &hashed_password],
        ) {
            Ok(modified_rows) => Ok(modified_rows),
            Err(err) => {
                if let Some(dberr) = err.as_db() {
                    println!("some dberr");
                    // unique violation
                    if !(dberr.code.code() == "23505") {
                        println!("code doesn't equal 23505");
                        return Err(AuthError::internal_error(&err.to_string()));
                    }
                    if let Some(constraint) = &dberr.constraint {
                        match constraint.as_ref() {
                            "users_email_key" => {
                                return Err(AuthError::new(
                                    "signupEmail",
                                    "This email has already been registered.",
                                    "",
                                    500,
                                ))
                            }
                            "users_username_key" => {
                                return Err(AuthError::new(
                                    "username",
                                    "This username has already been taken.",
                                    "",
                                    500,
                                ))
                            }
                            _ => return Err(AuthError::internal_error(&err.to_string())),
                        }
                    }
                }
                Err(AuthError::internal_error(&err.to_string()))
            }
        }
    }

    pub fn user_exists(&self, user: &User) -> Result<bool, AuthError> {
        let conn = self.pool.get()?;
        let rows = match conn.query(
            "SELECT username FROM users WHERE username=$1",
            &[&user.username],
        ) {
            Ok(r) => r,
            Err(err) => return Err(AuthError::internal_error(&err.to_string())),
        };

        if rows.is_empty() {
            return Ok(false);
        }
        Ok(true)
    }

    pub fn get_all_users(&self) -> Result<Vec<String>, AuthError> {
        let mut users: Vec<String> = Vec::new();
        let conn = self.pool.get()?;

        let rows = match conn.query("SELECT username FROM users", &[]) {
            Ok(r) => r,
            Err(err) => return Err(AuthError::internal_error(&err.to_string())),
        };

        for row in &rows {
            let username: String = row.get(0);
            users.push(username);
        }

        Ok(users)
    }

    pub fn verify_user(&self, user: &User) -> Result<String, AuthError> {

        let conn = self.pool.get()?;

        let rows = match conn.query(
            "SELECT username, password FROM users WHERE email=$1",
            &[&user.email],
        ) {
            Ok(r) => r,
            Err(err) => return Err(AuthError::internal_error(&err.to_string())),
        };

        let mut hashed_password = "".to_owned();
        let mut username = "".to_owned();
        for row in &rows {
            username = row.get(0);
            hashed_password = row.get(1);
            break;
        }
        if Auth::verify_hash(hashed_password, user.password.clone()) {
            Ok(username)
        } else {
            return Err(AuthError::new(
                "signin",
                "Email and password combo not found.",
                "Token hash wasn't verified.",
                400,
            ));
        }
    }
}
