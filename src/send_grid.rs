use crate::auth_error::AuthError;
use reqwest;
use reqwest::header::*;
use serde_json::json;
use std::sync::Arc;

#[derive(Clone)]
pub struct SendGrid {
    key: String,
    client: Arc<reqwest::Client>,
}

impl SendGrid {
    pub fn new(key: &str) -> SendGrid {
        SendGrid {
            key: key.to_owned(),
            client: Arc::new(reqwest::Client::new()),
        }
    }

    pub fn send_forgot_email(&self, to: &str) -> Result<(), AuthError> {
        let data = json!({
          "personalizations": [
            {
              "to": [
                {
                  "email": to
                }
              ],
              "subject": "Auth App: Password Reset"
            }
          ],
          "from": {
            "email": "support@authapp.com"
          },
          "content": [
            {
              "type": "text/plain",
              "value": "Hi, please use the following link to reset your password:\n\rtodo: add link\n\r\n\rIf you did not initiate this request, you can safely ignore this email.\n\r\n\rThanks,\n\rAuth App Support"
            }
          ]
        });
  
        let mut res = match self
            .client
            .post("https://api.sendgrid.com/v3/mail/send")
            .header(AUTHORIZATION, format!("Bearer {}", self.key))
            .header(CONTENT_TYPE, format!("application/json"))
            .json(&data)
            .send()
        {
            Ok(r) => r,
            Err(err) => return Err(AuthError::internal_error(&err.to_string())),
        };

        match res.text() {
            Ok(_) => Ok(()),
            Err(err) => Err(AuthError::internal_error(&err.to_string()))
        }
    }
}
