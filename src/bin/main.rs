#[macro_use]
extern crate serde_derive;

use actix_files as fs;
use actix_web::{guard, http, middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use auth_app::auth_error::*;
use auth_app::auth_google;
use futures::Future;
use r2d2_postgres::r2d2;
use r2d2_postgres::PostgresConnectionManager;
use serde;
use serde_json::{self, json};
use std::env;
use jsonwebtoken as jwt;
use jwt::{encode, decode, Header, Validation};
use chrono::{Duration, Utc};
use argon2::{self, Config};

static AUTH_APP: &'static str = "Auth App";

#[derive(Serialize, Deserialize)]
struct GoogleToken {
    id_token: String,
}

#[derive(Serialize, Deserialize)]
struct User {
    id: Option<i32>,
    email: String,
    username: String,
    password: String,
}

impl User {
    fn is_valid_email(&self) -> Result<(), AuthError> {
        if self.email == "" {
            return Err(AuthError::new("email", "Please enter your email.", "", 400));
        }
        Ok(())
    }

    fn is_valid_username(&self) -> Result<(), AuthError> {
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

    fn is_valid_password(&self) -> Result<(), AuthError> {
        if self.password == "" {
            return Err(AuthError::new(
                "password",
                "Please enter a password.",
                "",
                400,
            ));
        }
        Ok(())
    }

    fn is_valid_signup(&self) -> Result<(), AuthError> {
        self.is_valid_email()?;
        self.is_valid_username()?;
        self.is_valid_password()
    }

    fn is_valid_signin(&self) -> Result<(), AuthError> {
        self.is_valid_email()?;
        self.is_valid_password()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // subject
    iss: String, // issuer
    exp: usize, // expiration (time)
    nbf: usize, // not before (time)
}

impl Claims {
    fn new(username: String) -> Claims {
        Claims {
            sub: username,
            iss: AUTH_APP.to_owned(),
            nbf: Utc::now().timestamp() as usize,
            exp: (Utc::now() + Duration::weeks(2)).timestamp() as usize,
        }
    }
}


#[derive(Clone)]
struct Auth {
    jwt_secret: String,
    salt: String,
}

impl Auth {
    fn new(jwt_secret: String, salt: String) -> Auth {
        Auth {
            jwt_secret,
            salt,
        }
    }

    fn create_hash(&self, password: &str) -> String {
        let config = Config::default();
        argon2::hash_encoded(password.as_bytes(), self.salt.as_bytes(), &config).unwrap()
    }

    fn verify_hash(hash: String, password: String) -> bool {
        argon2::verify_encoded(&hash, password.as_bytes()).unwrap()
    }

    fn create_token(&self, username: String) -> Result<String, AuthError> {
        let claims = Claims::new(username);

        match encode(&Header::default(), &claims, self.jwt_secret.as_bytes()) {
            Ok(token) => Ok(token),
            Err(err) => Err(AuthError::internal_error(&err.to_string())),
        }
    }
}

fn make_success_json<T>(context: &str, data: T) -> serde_json::value::Value
where
    T: Into<serde_json::value::Value> + serde::Serialize,
{
    json!({
        "type": "success",
        "context": context,
        "data": data
    })
}

fn check_username(
    pool: web::Data<r2d2::Pool<PostgresConnectionManager>>,
    user: web::Json<User>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    actix_web::web::block(move || {
        let conn = pool.get()?;
        let rows = match conn.query("SELECT username FROM users WHERE username=$1", &[&user.username]) {
            Ok(r) => r,
            Err(err) => return Err(AuthError::internal_error(&err.to_string())),
        };

        if rows.is_empty() {
            Ok(())
        } else {
            return Err(AuthError::new(
                "username",
                "This username has already been taken.",
                "",
                500,
            ))
        }
    })
    .map_err(|err| {
        println!("check_username: {}", err);
        actix_web::Error::from(AuthError::from(err))
    })
    .and_then(|_| {
        HttpResponse::Ok()
            .content_type("application/json")
            .body(make_success_json("username", true))
    })
}

fn get_users(
    req: HttpRequest,
    pool: web::Data<r2d2::Pool<PostgresConnectionManager>>,
    auth: web::Data<Auth>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {

    let token_string = if let Some(header) = req.headers().get("Authorization") {
        let mut value = header.to_str().unwrap_or("").to_string();
        value.drain(0..7); // remove "Bearer " from value
        value
    } else {
        "".to_owned()
    };

    actix_web::web::block(move || {
        let validation = Validation {iss: Some(AUTH_APP.to_owned()), ..Default::default()};
        if let Err(err) = decode::<Claims>(&token_string, auth.jwt_secret.as_bytes(), &validation) {
            return Err(AuthError::new("auth", "Please log in or sign up to access this resource.", &err.to_string(), 401));
        };

        let mut users: Vec<String> = Vec::new();
        let conn = pool.get()?;

        let rows = match conn.query("SELECT username FROM users", &[]) {
            Ok(r) => r,
            Err(err) => return Err(AuthError::internal_error(&err.to_string())),
        };

        for row in &rows {
            let username: String = row.get(0);
            users.push(username);
        }

        Ok(users)
    })
    .map_err(|err| {
        println!("get_users: {}", err);
        actix_web::Error::from(AuthError::from(err))
    })
    .and_then(|res| {
        HttpResponse::Ok()
            .content_type("application/json")
            .body(make_success_json("users", res))
    })
}

fn verify_user(
    user: web::Json<User>,
    pool: web::Data<r2d2::Pool<PostgresConnectionManager>>,
    auth: web::Data<Auth>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    actix_web::web::block(move || {
        user.is_valid_signin()?;

        let conn = pool.get()?;

        let rows = match conn.query(
            "SELECT email, username, password FROM users WHERE email=$1",
            &[&user.email],
        ) {
            Ok(r) => r,
            Err(err) => return Err(AuthError::internal_error(&err.to_string()))
        };

        let mut hashed_password = "".to_owned();
        for row in &rows {
            hashed_password = row.get(2);
            break;
        }
        if Auth::verify_hash(hashed_password, user.password.clone()) {
            auth.create_token(user.username.clone())
        } else {
            return Err(AuthError::new("general", "Email and password combo not found.", "Token hash wasn't verified.", 400))
        }
    })
    .map_err(|err| {
        println!("verify_user: {}", err);
        actix_web::Error::from(AuthError::from(err))
    })
    .and_then(|token| {
        HttpResponse::Ok()
            .content_type("application/json")
            .body(make_success_json("signin", token))
    })
}

fn add_user(
    user: web::Json<User>,
    pool: web::Data<r2d2::Pool<PostgresConnectionManager>>,
    auth: web::Data<Auth>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    actix_web::web::block(move || {
        user.is_valid_signup()?;

        let conn = pool.get()?;

        let hashed_password = auth.create_hash(&user.password.clone());
        match conn.execute(
            "INSERT INTO users (email, username, password) VALUES ($1, $2, $3)",
            &[&user.email, &user.username, &hashed_password],
        ) {
            Ok(_) => auth.create_token(user.username.clone()),
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
                                    "email",
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
    })
    .map_err(|err| {
        println!("add_user: {}", err);
        actix_web::Error::from(AuthError::from(err))
    })
    .and_then(|token| {
        HttpResponse::Ok()
            .content_type("application/json")
            .body(make_success_json("signup", token))
    })
}

fn auth_google(
    _req: HttpRequest,
    token: web::Json<GoogleToken>,
    pool: web::Data<r2d2::Pool<PostgresConnectionManager>>,
    google: web::Data<auth_google::GoogleSignin>,
    auth: web::Data<Auth>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    actix_web::web::block(move || {
        let token_data = match google.decode_token(&token.id_token) {
            Ok(td) => td,
            Err(err) => return Err(AuthError::internal_error(&err.to_string())),
        };

        let conn = pool.get()?;

        let mut username = "".to_owned();
        let rows = match conn.query("SELECT username FROM users WHERE email=$1", &[&token_data.email]) {
            Ok(r) => r,
            Err(err) => return Err(AuthError::internal_error(&err.to_string())),
        };

        for row in &rows {
            username = row.get(0);
            break;
        }

        if username == "" {
            if let Err(err) = conn.execute(
                "INSERT INTO users (email, username, password) VALUES ($1, $2, $3)",
                &[&token_data.email, &token_data.given_name, &""],
            ) {
                return Err(AuthError::internal_error(&err.to_string()));
            }
            // new user
            auth.create_token(token_data.given_name)
        } else {
            // returning user
            auth.create_token(username)
        }
    })
    .map_err(|err| {
        println!("auth_google: {}", err);
        actix_web::Error::from(AuthError::from(err))
    })
    .and_then(|res| {
        HttpResponse::Ok()
            .content_type("application/json")
            .body(make_success_json("google", res))
    })
}

fn main() {
    let jwt_secret = env::var("AUTH_JWT_SECRET").expect("auth jwt secret not found");
    let salt = env::var("AUTH_SALT").expect("auth salt not found");
    let database_url = env::var("TSDB_URL").expect("tsdb url not found");
    let google_client_secret =
        env::var("GOOGLE_CLIENT_SECRET").expect("google client secret not found");

    let auth = Auth::new(jwt_secret, salt);
    
    let manager =
        PostgresConnectionManager::new(database_url, r2d2_postgres::TlsMode::None).unwrap();
    let pool = r2d2::Pool::builder().max_size(3).build(manager).unwrap();
    let google = auth_google::GoogleSignin::new(&google_client_secret);

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(google.clone())
            .data(auth.clone())
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/auth-db")
                    // .default_service(web::get().to_async(unsplash_get))
                    .route("/get-users", web::get().to_async(get_users))
                    .route("/add-user", web::post().to_async(add_user))
                    .route("/verify-user", web::post().to_async(verify_user))
                    .route("/check-username", web::post().to_async(check_username)),
            )
            .service(web::scope("/auth").route("/google", web::post().to_async(auth_google)))
            .service(fs::Files::new("/", "static/build").index_file("index.html"))
            .default_service(
                // 404 for GET request
                web::resource("")
                    .route(web::get().to(p404))
                    // all requests that are not GET
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
            )
    })
    .bind(("0.0.0.0", get_server_port()))
    .unwrap()
    .run()
    .unwrap();
}

fn p404() -> Result<fs::NamedFile, actix_web::Error> {
    Ok(fs::NamedFile::open("static/404.html")?.set_status_code(http::StatusCode::NOT_FOUND))
}

fn get_server_port() -> u16 {
    env::var("PORT")
        .unwrap_or_else(|_| 5000.to_string())
        .parse()
        .expect("PORT must be a number")
}
