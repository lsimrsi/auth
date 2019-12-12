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
}
