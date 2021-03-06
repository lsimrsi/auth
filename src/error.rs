use actix_web::error::BlockingError;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use r2d2_postgres::r2d2;
use reqwest;
use serde::Serialize;
use serde_json::{json, to_string_pretty};
use std::fmt::{self, Formatter, Result as FmtResult};

#[derive(Debug, Serialize)]
pub struct AuthError {
    client_message: String,
    server_message: String,
    context: String,
    status: u16,
}

impl AuthError {
    pub fn new(
        context: &str,
        client_message: &str,
        server_message: &str,
        status: u16,
    ) -> AuthError {
        let error: &str;

        if server_message == "" {
            error = client_message;
        } else {
            error = server_message;
        }

        AuthError {
            context: context.to_owned(),
            client_message: client_message.to_owned(),
            server_message: error.to_owned(),
            status,
        }
    }

    pub fn new_general(client_message: &str, server_message: &str, status: u16) -> AuthError {
        AuthError::new("general", client_message, server_message, status)
    }

    pub fn internal_error(error: &str) -> AuthError {
        AuthError {
            context: "general".to_owned(),
            client_message: "Something went wrong. Please try again later.".to_owned(),
            server_message: error.to_owned(),
            status: 500,
        }
    }
}

impl From<BlockingError<AuthError>> for AuthError {
    fn from(error: BlockingError<AuthError>) -> Self {
        match error {
            BlockingError::Error(err) => err,
            BlockingError::Canceled => AuthError::new("", "", "", 200),
        }
    }
}

impl From<r2d2::Error> for AuthError {
    fn from(error: r2d2::Error) -> Self {
        AuthError::new("general", "Internal Error.", &error.to_string(), 500)
    }
}

impl From<reqwest::Error> for AuthError {
    fn from(error: reqwest::Error) -> Self {
        AuthError::new("general", "Internal Error.", &error.to_string(), 500)
    }
}

// impl<T> From<Result<T, reqwest::Error>> for AuthError {
//     fn from(error: Result<T, reqwest::Error>) -> Result<T, AuthError> {

//     }
// }

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "{}",
            to_string_pretty(self).expect("Couldn't format AuthError for display.")
        )
    }
}

impl ResponseError for AuthError {
    fn render_response(&self) -> HttpResponse {
        let err_json = json!({
            "type": "error",
            "context": self.context,
            "data": self.client_message
        });
        HttpResponse::build(StatusCode::from_u16(self.status).expect("Invalid status code given."))
            .json(err_json)
    }
}

impl std::error::Error for AuthError {
    fn description(&self) -> &str {
        &self.client_message
    }
}
