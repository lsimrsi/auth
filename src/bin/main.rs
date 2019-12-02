#[macro_use]
extern crate serde_derive;

use actix_files as fs;
use actix_web::{guard, http, middleware, web, App, HttpRequest, HttpResponse, ResponseError, HttpServer};
use actix_web::http::StatusCode;
use failure::Error;
use futures::Future;
use r2d2_postgres::r2d2;
use r2d2_postgres::PostgresConnectionManager;
use serde_json;
use serde_json::{json, to_string_pretty};
use std::env;
use auth::auth_google;
use std::fmt::{self, Formatter, Result as FmtResult};

fn p404() -> Result<fs::NamedFile, Error> {
    Ok(fs::NamedFile::open("static/404.html")?.set_status_code(http::StatusCode::NOT_FOUND))
}

fn get_server_port() -> u16 {
    env::var("PORT")
        .unwrap_or_else(|_| 5000.to_string())
        .parse()
        .expect("PORT must be a number")
}

#[derive(Debug, Serialize)]
struct AuthError {
    msg: String,
    status: u16,
}

impl AuthError {
    fn new(msg: &str, status: u16) -> AuthError {
        AuthError {
            msg: msg.to_owned(),
            status
        }
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", to_string_pretty(self).unwrap())
    }
}

impl ResponseError for AuthError {
    // builds the actual response to send back when an error occurs
    fn render_response(&self) -> HttpResponse {
        let err_json = json!({ "error": self.msg });
        HttpResponse::build(StatusCode::from_u16(self.status).unwrap()).json(err_json)
    }
}

impl std::error::Error for AuthError {
    fn description(&self) -> &str { &self.msg }
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

impl User {
    fn is_valid(&self) -> Result<(), AuthError> {
        if self.username == "" {
            return Err(AuthError::new("Nah", 500));
        }
        if self.email == "" {
            return Err(AuthError::new("Nah", 500));
        }
        if self.pw == "" {
            return Err(AuthError::new("Nah", 500));
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
        for row in &conn.query("SELECT username FROM users;", &[])? {
            let username: String = row.get(0);
            users.push(username);
        }

        if users.is_empty() {
            Err(failure::err_msg("No users".to_owned()))
        } else {
            let json_users = serde_json::to_string(&users)?;
            Ok(json_users)
        }
    })
    .map_err(|err| {
        println!("get_users: {}", err);
        actix_web::Error::from(failure::err_msg("No results".to_owned()))
    })
    .and_then(|res| {
        HttpResponse::Ok()
            .content_type("application/json")
            .body(res.to_owned())
    })
}

fn add_user(
    _req: HttpRequest,
    user: web::Json<User>,
    pool: web::Data<r2d2::Pool<PostgresConnectionManager>>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    actix_web::web::block(move || {
        user.is_valid()?;

        let conn = match pool.get() {
            Ok(conn) => conn,
            Err(err) => {
                println!("add_user: {}", err);
                return Err(AuthError {
                    msg: "Internal Error".to_string(),
                    status: 500,
                })
            }
        };
        let rows_updated = conn.execute(
            "INSERT INTO users (email, username, pw) VALUES ($1, $2, $3)",
            &[&user.email, &user.username, &user.pw],
        );

        match rows_updated {
            Ok(num) => Ok(num),
            Err(err) => {
                println!("add_user: {}", err);
                Err(AuthError {
                    msg: "Internal Error".to_string(),
                    status: 500,
                })
            },
        }
    })
    .map_err(|err| {
        println!("add_user: {}", err);
        actix_web::Error::from(err)
    })
    .and_then(|res| {
        let res_json = serde_json::to_string(&res.to_owned()).unwrap_or("".to_owned());
        HttpResponse::Ok()
            .content_type("application/json")
            .body(res_json)
    })
}

fn auth_google(
    _req: HttpRequest,
    token: web::Json<GoogleToken>,
    pool: web::Data<r2d2::Pool<PostgresConnectionManager>>,
    google: web::Data<auth_google::GoogleSignin>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    actix_web::web::block(move || {
        let token_data = google.decode_token(&token.id_token)?;

        let conn = pool.get()?;
        let mut username = "".to_owned();
        for row in &conn.query("
            SELECT username FROM users
            WHERE email=$1 AND username=$2;", &[&token_data.email, &token_data.given_name])? {
            username = row.get(0);
            break;
        }

        if username == "" {
            let rows_updated = conn.execute(
                "INSERT INTO users (email, username, pw) VALUES ($1, $2, $3)",
                &[&token_data.email, &token_data.name, &""],
            );

            match rows_updated {
                Ok(num) => Ok(num),
                Err(err) => Err(failure::err_msg(err.to_string())),
            }
        } else {
            Ok(1)
        }
    })
    .map_err(|err| {
        println!("auth_google: {}", err);
        let json_error = serde_json::to_string("Could not validate token").unwrap();
        actix_web::Error::from(failure::err_msg(json_error))
    })
    .and_then(|res| {
        let res_json = serde_json::to_string(&res.to_owned()).unwrap_or("".to_owned());
        HttpResponse::Ok()
            .content_type("application/json")
            .body(res_json)
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
            .service(
                web::scope("/auth").route("/google", web::post().to_async(auth_google)),
            )
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
