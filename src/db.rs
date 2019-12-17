use crate::auth::{Auth, User};
use crate::error::AuthError;
use r2d2_postgres::r2d2;
use r2d2_postgres::PostgresConnectionManager;

#[derive(Clone)]
pub struct Db {
    pub pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager>,
}

impl Db {
    pub fn new(url: &str) -> Db {
        let manager = PostgresConnectionManager::new(url, r2d2_postgres::TlsMode::None)
            .expect("Couldn't make a new postgres connection manager.");
        let pool = r2d2::Pool::builder()
            .max_size(3)
            .build(manager)
            .expect("Couldn't make the connection pool.");
        Db { pool }
    }

    pub fn add_user(&self, user: &User) -> Result<u64, AuthError> {
        let conn = self.pool.get()?;
        match conn.execute("CALL add_user($1, $2, $3);",
            &[&user.email, &user.username, &user.password],
        ) {
            Ok(modified_rows) => Ok(modified_rows),
            Err(err) => {
                if let Some(dberr) = err.as_db() {
                    println!("some dberr");
                    // 23505 = unique violation
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

    pub fn user_exists(&self, username: &str) -> Result<bool, AuthError> {
        let conn = self.pool.get()?;
        let rows = match conn.query(
            "SELECT username FROM users WHERE username=$1 OR email=$1",
            &[&username],
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

    pub fn get_user_by_email(&self, email: &str) -> Result<String, AuthError> {
        let conn = self.pool.get()?;
        let rows = match conn.query("SELECT username FROM users WHERE email=$1", &[&email]) {
            Ok(r) => r,
            Err(err) => return Err(AuthError::internal_error(&err.to_string())),
        };

        if rows.is_empty() {
            // return ok even if user isn't found
            // ecurity through obscurity
            return Ok("".to_owned());
        }

        let mut username = "".to_owned();
        for row in &rows {
            username = row.get(0);
            break;
        }
        Ok(username)
    }

    pub fn update_user_password(&self, email: &str, password: &str) -> Result<u64, AuthError> {
        let conn = self.pool.get()?;
        match conn.execute(
            "UPDATE users SET password = $1 WHERE username=$2",
            &[&password, &email],
        ) {
            Ok(num) => Ok(num),
            Err(err) => Err(AuthError::internal_error(&err.to_string())),
        }
    }
}
