use std::fmt::{self, Formatter, Result as FmtResult};
use actix_web::{HttpResponse, HttpRequest, ResponseError, Responder};
use actix_web::http::StatusCode;
use actix_web::error::BlockingError;
use serde_json::{json, to_string_pretty};
use serde_json::value::Value;
use r2d2_postgres::r2d2;

#[derive(Debug, Serialize)]
pub struct AuthError {
    client_message: String,
    server_message: String,
    context: String,
    status: u16,
}

impl AuthError {
    pub fn new(context: &str, client_message: &str, server_message: &str, status: u16) -> AuthError {
        let error: &str;

        // if server message is empty, just make it the same as client message
        if server_message == "" {
            error = client_message;
        } else {
            error = server_message;
        }

        AuthError {
            context: context.to_owned(),
            client_message: client_message.to_owned(),
            server_message: error.to_owned(),
            status
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
            status: 500
        }
    }
}

impl From<BlockingError<AuthError>> for AuthError {
    fn from(error: BlockingError<AuthError>) -> Self {
        match error {
            BlockingError::Error(err) => err,
            _ => AuthError::new("", "", "", 200),
        }
    }
}

impl From<r2d2::Error> for AuthError {
    fn from(error: r2d2::Error) -> Self {
        AuthError::new("general", "Internal Error.", &error.to_string(), 500)
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", to_string_pretty(self).unwrap())
    }
}

impl ResponseError for AuthError {
    fn render_response(&self) -> HttpResponse {
        let err_json = json!({ "data": {
            "type": "error",
            "context": self.context,
            "message": self.client_message
        } });
        HttpResponse::build(StatusCode::from_u16(self.status).unwrap()).json(err_json)
    }
}

impl std::error::Error for AuthError {
    fn description(&self) -> &str { &self.client_message }
}