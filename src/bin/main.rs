#[macro_use]
extern crate serde_derive;

use actix_files as fs;
use actix_web::{guard, http, middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use auth::auth_error::*;
use auth::auth_google;
use futures::Future;
use r2d2_postgres::r2d2;
use r2d2_postgres::PostgresConnectionManager;
use serde_json::{self, json};
use std::env;
use serde;

#[derive(Serialize, Deserialize)]
struct GoogleToken {
    id_token: String,
}

#[derive(Serialize, Deserialize)]
struct User {
    id: Option<i32>,
    email: String,
    username: String,
    pw: String,
}

fn make_success_json<T: Into<serde_json::value::Value> + serde::Serialize>(context: &str, message: T) -> serde_json::value::Value {
    json!({
        "type": "success",
        "context": context,
        "message": message
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
        if self.pw == "" {
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

fn get_users(
    pool: web::Data<r2d2::Pool<PostgresConnectionManager>>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    actix_web::web::block(move || {
        let mut users: Vec<String> = Vec::new();

        let conn = pool.get()?;

        let rows = match conn.query("SELECT username FROM users;", &[]) {
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
            "INSERT INTO users (email, username, pw) VALUES ($1, $2, $3)",
            &[&user.email, &user.username, &user.pw],
        ) {
            Ok(_) => Ok(()),
            Err(err) => {
                if let Some(dberr) = err.as_db() {
                    println!("some dberr");
                    // unique violation
                    if !(dberr.code.code() == "23505") {
                        println!("code doesn't equal 23505");
                        return Err(AuthError::internal_error(&err.to_string()));
                    }
                    if let Some(constraint) = &dberr.constraint {
                        println!("constraint: {}", constraint);
                        match constraint.as_ref() {
                            "users_email_key" => return Err(AuthError::new(
                                "email",
                                "This email has already been registered.",
                                "",
                                500,
                            )),
                            "users_username_key" => return Err(AuthError::new(
                                "username",
                                "This username has already been taken.",
                                "",
                                500,
                            )),
                            _ => return Err(AuthError::internal_error(&err.to_string()))
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
    .and_then(|_| {
        HttpResponse::Ok()
            .content_type("application/json")
            .body(make_success_json("signup", "Registered!"))
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
        let rows = match conn.query(
            "SELECT id FROM users WHERE email=$1",
            &[&token_data.email],
        ) {
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
                "INSERT INTO users (email, username, pw) VALUES ($1, $2, $3)",
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
                    .route("/add-user", web::post().to_async(add_user)),
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