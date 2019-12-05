#[macro_use]
extern crate serde_derive;

use actix_files as fs;
use actix_web::{guard, http, middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use auth::auth_error::*;
use auth::auth_google;
use futures::Future;
use r2d2_postgres::r2d2;
use r2d2_postgres::PostgresConnectionManager;
use serde;
use serde_json::{self, json};
use std::env;
use jsonwebtoken as jwt;
use jwt::{encode, decode, Header, Validation};
use chrono::{Duration, Utc};

static JWT_SECRET: &'static str = "wegotasecretoverhere";
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

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
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

impl User {
    fn is_valid(&self) -> Result<(), AuthError> {
        if self.email == "" {
            return Err(AuthError::new("email", "Please enter your email.", "", 400));
        }
        if self.username == "" {
            return Err(AuthError::new(
                "username",
                "Please enter a username.",
                "",
                400,
            ));
        }
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
        if let Err(err) = decode::<Claims>(&token_string, JWT_SECRET.as_ref(), &validation) {
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

fn add_user(
    _req: HttpRequest,
    user: web::Json<User>,
    pool: web::Data<r2d2::Pool<PostgresConnectionManager>>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    actix_web::web::block(move || {
        user.is_valid()?;

        let conn = pool.get()?;

        match conn.execute(
            "INSERT INTO users (email, username, password) VALUES ($1, $2, $3)",
            &[&user.email, &user.username, &user.password],
        ) {
            Ok(_) => Ok(user.username.clone()),
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
    .and_then(|username| {
        let claims = Claims::new(username);

        let token = match encode(&Header::default(), &claims, JWT_SECRET.as_bytes()) {
            Ok(token) => token,
            Err(err) => {
                let error = AuthError::internal_error(&err.to_string());
                return HttpResponse::from_error(actix_web::Error::from(error));
            }
        };
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
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    actix_web::web::block(move || {
        let token_data = match google.decode_token(&token.id_token) {
            Ok(td) => td,
            Err(err) => return Err(AuthError::internal_error(&err.to_string())),
        };

        let conn = pool.get()?;

        let mut id = 0;
        let rows = match conn.query("SELECT id FROM users WHERE email=$1", &[&token_data.email]) {
            Ok(r) => r,
            Err(err) => return Err(AuthError::internal_error(&err.to_string())),
        };

        for row in &rows {
            id = row.get(0);
            break;
        }

        // todo: send token
        if id == 0 {
            if let Err(err) = conn.execute(
                "INSERT INTO users (email, username, password) VALUES ($1, $2, $3)",
                &[&token_data.email, &token_data.given_name, &""],
            ) {
                return Err(AuthError::internal_error(&err.to_string()));
            }
            Ok("Registered!")
        } else {
            Ok("Authenticated!")
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
    let database_url = env::var("TSDB_URL").expect("the database url must be set");
    let google_client_secret =
        env::var("GOOGLE_CLIENT_SECRET").expect("google client secret env variable not present");

    let manager =
        PostgresConnectionManager::new(database_url, r2d2_postgres::TlsMode::None).unwrap();
    let pool = r2d2::Pool::builder().max_size(4).build(manager).unwrap();
    let google = auth_google::GoogleSignin::new(&google_client_secret);

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(google.clone())
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/auth-db")
                    // .default_service(web::get().to_async(unsplash_get))
                    .route("/get-users", web::get().to_async(get_users))
                    .route("/add-user", web::post().to_async(add_user))
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
