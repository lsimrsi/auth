use crate::error::AuthError;
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

    pub fn send_forgot_email(&self, to: &str, token: &str, username: &str) -> Result<(), AuthError> {
        // let msg = format!(
        //     "Hi, please use the following link to reset your password:
        //     \n\rhttp://localhost:3000/reset-password?token={}
        //     \n\rIf you did not initiate this request, you can safely ignore this email.
        //     \n\rThanks,
        //     \n\rAuth App Support",
        //     token
        // );

        let href = format!("http://localhost:3000/reset-password?token={}", token);
        let html_msg = format!(r#"
        <div style="display: flex;">
        <table style="font-family: sans-serif; color: #555; padding: 20px; margin: auto; border: 3px solid #ccc; border-radius: 20px;">
            <tr>
            <td>Hi {0}, please use the following link to reset your password:</td>
            </tr>
            <tr>
            <td><a href="{1}"><h3>{1}</h3></a></td>
            </tr>
            <tr>
            <td style="padding-bottom: 20px;">If you did not initiate this request, you can safely ignore this email.</td>
            </tr>
            <tr>
            <td>Thanks,</td>
            </tr>
            <tr>
            <td>Auth App Support</td>
            </tr>
        </table>
        </div>
        "#, username, href);

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
              "type": "text/html",
              "value": html_msg
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
            Err(err) => Err(AuthError::internal_error(&err.to_string())),
        }
    }
}
