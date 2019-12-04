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

fn p404() -> Result<fs::NamedFile, actix_web::Error> {
    Ok(fs::NamedFile::open("static/404.html")?.set_status_code(http::StatusCode::NOT_FOUND))
}

fn get_server_port() -> u16 {
    env::var("PORT")
        .unwrap_or_else(|_| 5000.to_string())
        .parse()
        .expect("PORT must be a number")
}

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

fn authenticated() -> serde_json::value::Value {
    json!({ "data": {
        "type": "success",
        "context": "signin",
        "message": "Authenticated!"
    } })
}

fn registered() -> serde_json::value::Value {
    json!({ "data": {
        "type": "success",
        "context": "signin",
        "message": "Registered!"
    } })
}

fn get_users_json(data: Vec<String>) -> serde_json::value::Value {
    json!({ "data": {
        "type": "success",
        "context": "users",
        "message": data
    } })
}

fn add_user_json() -> serde_json::value::Value {
    json!({ "data": {
        "type": "success",
        "context": "signup",
        "message": "Registered!"
    } })
}

impl User {
    fn is_valid(&self) -> Result<(), AuthError> {
        if self.email == "" {
            return Err(AuthError::new("email", "Please enter your email.", "", 400));
        }
        if self.username == "" {
            return Err(AuthError::new("username", "Please enter a username.", "", 400));
        }
        if self.pw == "" {
            return Err(AuthError::new("password", "Please enter a password.", "", 400));
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
            .body(get_users_json(res))
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

        let mut username = "".to_owned();
        let rows = match conn.query(
            "SELECT username FROM users WHERE email=$1",
            &[&user.email]) {
            Ok(r) => r,
            Err(err) => return Err(AuthError::internal_error(&err.to_string())),
        };

        for row in &rows {
            username = row.get(0);
            break;
        }

        if username == user.username {
            return Err(AuthError::new("username", "This name has been taken.", "", 400));
        }
        if !(username == "") {
            return Err(AuthError::new("email", "This email has already been registered.", "", 400));
        }

        match conn.execute(
            "INSERT INTO users (email, username, pw) VALUES ($1, $2, $3)",
            &[&user.email, &user.username, &user.pw],
        ) {
            Ok(_) => Ok(()),
            Err(err) => return Err(AuthError::internal_error(&err.to_string())),
        }
    })
    .map_err(|err| {
        println!("add_user: {}", err);
        actix_web::Error::from(AuthError::from(err))
    })
    .and_then(|_| {
        HttpResponse::Ok()
            .content_type("application/json")
            .body(add_user_json())
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

        let mut username = "".to_owned();
        let rows = match conn.query(
            "SELECT username FROM users WHERE email=$1",
            &[&token_data.email]) {
            Ok(r) => r,
            Err(err) => return Err(AuthError::internal_error(&err.to_string())),
        };

        for row in &rows {
            username = row.get(0);
            break;
        }

        if username == "" {
            let rows_updated = conn.execute(
                "INSERT INTO users (email, username, pw) VALUES ($1, $2, $3)",
                &[&token_data.email, &token_data.given_name, &""],
            );

            match rows_updated {
                Ok(_) => Ok(registered()),
                Err(err) => return Err(AuthError::internal_error(&err.to_string())),
            }
        } else {
            // todo: log user in
            Ok(authenticated())
        }
    })
    .map_err(|err| {
        println!("auth_google: {}", err);
        actix_web::Error::from(AuthError::from(err))
    })
    .and_then(|res| {
        HttpResponse::Ok()
            .content_type("application/json")
            .body(res.to_string())
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
