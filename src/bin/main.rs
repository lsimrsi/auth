use actix_files as fs;
use actix_web::{guard, http, middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use auth_app::auth::{self, Auth, ClaimsDuration};
use auth_app::auth_error::*;
use auth_app::db::Db;
use auth_app::*;
use futures::Future;
use serde;
use serde_json::{self, json};
use std::env;

fn check_username(
    db: web::Data<Db>,
    user: web::Json<auth::User>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    actix_web::web::block(move || {
        let exists = db.user_exists(&user.username)?;
        match exists {
            true => Err(AuthError::new("username", "This username has already been taken.", "", 500)),
            false => Ok(())
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
    db: web::Data<Db>,
    auth: web::Data<auth::Auth>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let token_string = get_authorization_header(req.headers());

    actix_web::web::block(move || {
        auth.decode_token(&token_string)?;
        db.get_all_users()
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
    db: web::Data<Db>,
    auth: web::Data<auth::Auth>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    actix_web::web::block(move || {
        user.is_valid_signin()?;
        let username = db.verify_user(&user)?;
        auth.create_token(&username, ClaimsDuration::Weeks2)
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
    db: web::Data<Db>,
    auth: web::Data<Auth>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    actix_web::web::block(move || {
        user.is_valid_signup()?;
        db.insert_user(&user, &auth)?;
        auth.create_token(&user.username, ClaimsDuration::Weeks2)
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
    db: web::Data<Db>,
    google: web::Data<auth_google::GoogleSignin>,
    auth: web::Data<Auth>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    actix_web::web::block(move || {
        // decode the google token or throw an error
        let token_data = match google.decode_token(&token.id_token) {
            Ok(td) => td,
            Err(err) => return Err(AuthError::internal_error(&err.to_string())),
        };

        // create User from token data
        let user = auth::User::new(&token_data.email, &token_data.given_name, "");

        // check that the user is valid by our own standards or throw an error
        user.is_valid_email("email")?;
        user.is_valid_username()?;

        // check if user exists in our db
        let exists = db.user_exists(&user.username)?;

        if !exists {
            // if user doesn't exist, create a new user
            db.insert_user(&user, &auth)?;
        }
        // todo: prevent google users from changing their username
        auth.create_token(&user.username, ClaimsDuration::Weeks2)
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
    db: web::Data<Db>,
    send_grid: web::Data<send_grid::SendGrid>,
    user: web::Json<auth::User>,
    auth: web::Data<Auth>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    actix_web::web::block(move || {
        // check if the email is valid
        user.is_valid_email("email")?;

        // if the email exists in the database,
        // the username is returned
        let username = db.get_user_by_email(&user.email)?;

        // if username is not empty, send a password reset email
        if !username.is_empty() {
            let token = auth.create_token(&username, ClaimsDuration::Minutes5)?;
            send_grid.send_forgot_email(&user.email, &token)?;
        }

        // return ok even if the username is empty
        // security through obscurity
        Ok("Email sent!")
    })
    .map_err(|err| {
        println!("forgot_password: {}", err);
        actix_web::Error::from(AuthError::from(err))
    })
    .and_then(|res| {
        HttpResponse::Ok()
            .content_type("application/json")
            .body(make_success_json("forgotPassword", res))
    })
}

fn reset_password(
    req: HttpRequest,
    db: web::Data<Db>,
    user: web::Json<auth::User>,
    auth: web::Data<Auth>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let token_string = get_authorization_header(req.headers());

    actix_web::web::block(move || {
        user.is_valid_password("resetPassword")?;
        let claims = auth.decode_token(&token_string)?;
        let hashed_password = auth.create_hash(&user.password);
        let num = db.update_user_password(&claims.sub, &hashed_password)?;
        if num != 0 {
            auth.create_token(&user.username, ClaimsDuration::Weeks2)
        } else {
            Err(AuthError::internal_error("No rows modified for updating password."))
        }
    })
    .map_err(|err| {
        println!("forgot_password: {}", err);
        actix_web::Error::from(AuthError::from(err))
    })
    .and_then(|res| {
        HttpResponse::Ok()
            .content_type("application/json")
            .body(make_success_json("resetPassword", res))
    })
}

fn main() {
    let jwt_secret = env::var("AUTH_JWT_SECRET").expect("auth jwt secret not found");
    let salt = env::var("AUTH_SALT").expect("auth salt not found");
    let database_url = env::var("TSDB_URL").expect("tsdb url not found");
    let send_grid_key = env::var("AUTH_SEND_GRID_KEY").expect("send grid key not found");

    let auth = Auth::new(jwt_secret, salt);
    // let manager =
    // PostgresConnectionManager::new(database_url.clone(), r2d2_postgres::TlsMode::None).unwrap();
    // let pool = r2d2::Pool::builder().max_size(3).build(manager).unwrap();
    let google = auth_google::GoogleSignin::new();
    let send_grid = send_grid::SendGrid::new(&send_grid_key);
    let db = Db::new(&database_url);

    HttpServer::new(move || {
        App::new()
            .data(db.clone())
            .data(google.clone())
            .data(auth.clone())
            .data(send_grid.clone())
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/auth")
                    // .default_service(web::get().to_async(unsplash_get))
                    .route("/add-user", web::post().to_async(add_user))
                    .route("/verify-user", web::post().to_async(verify_user))
                    .route("/check-username", web::post().to_async(check_username))
                    .route("/forgot-password", web::post().to_async(forgot_password))
                    .route("/reset-password", web::post().to_async(reset_password))
                    .route("/google", web::post().to_async(auth_google)),
            )
            .service(web::scope("/protected").route("/users", web::get().to_async(get_users)))
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

fn get_authorization_header(header_map: &actix_web::http::header::HeaderMap) -> String {
    if let Some(header) = header_map.get("Authorization") {
        let mut value = header.to_str().unwrap_or("").to_string();
        value.drain(0..7); // remove "Bearer " from value
        return value;
    }
    "".to_owned()
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
