use actix_files as fs;
use actix_web::{guard, http, middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use auth_app::auth::{self, Auth};
use auth_app::auth_error::*;
use auth_app::*;
use futures::Future;
use r2d2_postgres::r2d2;
use r2d2_postgres::PostgresConnectionManager;
use serde;
use serde_json::{self, json};
use std::env;

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
    user: web::Json<auth::User>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    actix_web::web::block(move || {
        let conn = pool.get()?;
        let rows = match conn.query(
            "SELECT username FROM users WHERE username=$1",
            &[&user.username],
        ) {
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
            ));
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
    auth: web::Data<auth::Auth>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let token_string = if let Some(header) = req.headers().get("Authorization") {
        let mut value = header.to_str().unwrap_or("").to_string();
        value.drain(0..7); // remove "Bearer " from value
        value
    } else {
        "".to_owned()
    };

    actix_web::web::block(move || {
        auth.decode_token(&token_string)?;

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
    user: web::Json<auth::User>,
    pool: web::Data<r2d2::Pool<PostgresConnectionManager>>,
    auth: web::Data<auth::Auth>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    actix_web::web::block(move || {
        user.is_valid_signin()?;

        let conn = pool.get()?;

        let rows = match conn.query(
            "SELECT email, username, password FROM users WHERE email=$1",
            &[&user.email],
        ) {
            Ok(r) => r,
            Err(err) => return Err(AuthError::internal_error(&err.to_string())),
        };

        let mut hashed_password = "".to_owned();
        for row in &rows {
            hashed_password = row.get(2);
            break;
        }
        if Auth::verify_hash(hashed_password, user.password.clone()) {
            auth.create_token(user.username.clone())
        } else {
            return Err(AuthError::new(
                "signin",
                "Email and password combo not found.",
                "Token hash wasn't verified.",
                400,
            ));
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
    user: web::Json<auth::User>,
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
    token: web::Json<auth::GoogleToken>,
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
        let rows = match conn.query(
            "SELECT username FROM users WHERE email=$1",
            &[&token_data.email],
        ) {
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
            // returning user, maybe they changed username (todo)
            auth.create_token(username)
        }
    })
    .map_err(|err| {
        println!("auth_google: {}", err);
        actix_web::Error::from(AuthError::from(err))
    })
    .and_then(|token| {
        HttpResponse::Ok()
            .content_type("application/json")
            .body(make_success_json("google", token))
    })
}

fn forgot_password(
    pool: web::Data<r2d2::Pool<PostgresConnectionManager>>,
    send_grid: web::Data<send_grid::SendGrid>,
    user: web::Json<auth::User>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    actix_web::web::block(move || {
        user.is_valid_username()?;
        let conn = pool.get()?;
        let rows = match conn.query(
            "SELECT email FROM users WHERE username=$1",
            &[&user.username],
        ) {
            Ok(r) => r,
            Err(err) => return Err(AuthError::internal_error(&err.to_string())),
        };

        if rows.is_empty() {
            // return ok even if user isn't found
            // security through obscurity
            return Ok(())
        }

        let mut email = "".to_owned();
        for row in &rows {
            email = row.get(0);
            break;
        }

        send_grid.send_forgot_email(&email)
    })
    .map_err(|err| {
        println!("forgot_password: {}", err);
        actix_web::Error::from(AuthError::from(err))
    })
    .and_then(|_| {
        HttpResponse::Ok()
            .content_type("application/json")
            .body(make_success_json("forgotPassword", "Email sent!"))
    })
}

fn main() {
    let jwt_secret = env::var("AUTH_JWT_SECRET").expect("auth jwt secret not found");
    let salt = env::var("AUTH_SALT").expect("auth salt not found");
    let database_url = env::var("TSDB_URL").expect("tsdb url not found");
    let send_grid_key = env::var("AUTH_SEND_GRID_KEY").expect("send grid key not found");

    let auth = Auth::new(jwt_secret, salt);
    let manager =
        PostgresConnectionManager::new(database_url, r2d2_postgres::TlsMode::None).unwrap();
    let pool = r2d2::Pool::builder().max_size(3).build(manager).unwrap();
    let google = auth_google::GoogleSignin::new();
    let send_grid = send_grid::SendGrid::new(&send_grid_key);

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(google.clone())
            .data(auth.clone())
            .data(send_grid.clone())
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/auth-db")
                    // .default_service(web::get().to_async(unsplash_get))
                    .route("/get-users", web::get().to_async(get_users))
                    .route("/add-user", web::post().to_async(add_user))
                    .route("/verify-user", web::post().to_async(verify_user))
                    .route("/check-username", web::post().to_async(check_username))
                    .route("/forgot-password", web::post().to_async(forgot_password)),
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
