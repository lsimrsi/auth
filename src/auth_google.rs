use failure;
use jsonwebtoken as jwt;
use jwt::{Algorithm, Validation};
use reqwest;
use std::sync::{Mutex, Arc};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct GoogleSignin {
    client: Arc<reqwest::Client>,
    certs: Arc<Mutex<Certs>>,
    cert_expiration: Arc<Mutex<i64>>,
}

impl GoogleSignin {
    pub fn new() -> GoogleSignin {
        GoogleSignin {
            client: Arc::new(reqwest::Client::new()),
            certs: Arc::new(Mutex::new(Certs::new())),
            cert_expiration: Arc::new(Mutex::new(0)),
        }
    }

    fn get_new_certs(&self) -> reqwest::Result<(i64, Certs)> {
        let url = "https://www.googleapis.com/oauth2/v3/certs";
        let mut res = self.client.get(url).send()?;
        let time = match res.headers().get("expires") {
            Some(expires) => expires.to_str().unwrap_or(""),
            None => ""
        };

        let new_expiration = match DateTime::parse_from_rfc2822(time) {
            Ok(dt) => dt.timestamp(),
            Err(_) => 0,
        };

        Ok((new_expiration, res.json()?))
    }

    fn get_cached_certs(&self) -> Result<Certs, failure::Error> {
        let mut current_certs = match self.certs.lock() {
            Ok(cc) => cc,
            Err(err) => return Err(failure::err_msg(err.to_string())),
        };

        let mut current_expiration = match self.cert_expiration.lock() {
            Ok(ce) => ce,
            Err(err) => return Err(failure::err_msg(err.to_string())),
        };

        // if certs are expired or empty, get new ones
        if *current_expiration < Utc::now().timestamp() || current_certs.keys.is_empty() {
            let (new_expiration, new_certs) = self.get_new_certs()?;
            *current_expiration = new_expiration;
            *current_certs = new_certs;
        }

        Ok(current_certs.clone())
    }

    pub fn decode_token(&self, token: &str) -> Result<GooglePayload, failure::Error> {
        let certs: Certs = self.get_cached_certs()?;

        let claimed_kid = jwt::decode_header(&token)?.kid.unwrap_or_default();
        let mut e = "";
        let mut n = "";

        let mut b_match = false;
        for key in &certs.keys {
            if key.kid == claimed_kid {
                b_match = true;
                n = &key.n;
                e = &key.e;
                break;
            }
        }

        if !b_match {
            return Err(failure::err_msg(
                "google decode_token: kid does not match any google kid".to_string(),
            ));
        }

        let mut validation = Validation {
            leeway: 10,
            iss: Some("accounts.google.com".to_owned()),
            algorithms: vec![Algorithm::RS256],
            ..Validation::default()
        };
        validation.set_audience(&[
            "709178405751-3gehnuuoka3ccht41qs4uo175vc6vg3f.apps.googleusercontent.com",
        ]);

        let token_data = jwt::decode_rsa_components::<GooglePayload>(&token, &n, &e, &validation)?;
        Ok(token_data.claims)
    }
}

#[derive(Serialize, Deserialize)]
pub struct GoogleToken {
    pub id_token: String,
}

#[derive(Deserialize, Debug, Clone)]
struct JWK {
    alg: String,
    n: String,
    kid: String,
    e: String,
    kty: String,
    r#use: String,
}

#[derive(Deserialize, Debug, Clone)]
struct Certs {
    keys: Vec<JWK>,
}

impl Certs {
    fn new() -> Certs {
        Certs {
            keys: Vec::new()
        }
    }
}

#[derive(Deserialize, Debug)]
struct Header {
    alg: String,
    kid: String,
    typ: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GooglePayload {
    pub iss: String,
    pub azp: String,
    pub aud: String,
    pub sub: String,
    pub email: String,
    pub email_verified: bool,
    pub at_hash: String,
    pub name: String,
    pub picture: String,
    pub given_name: String,
    pub family_name: String,
    pub locale: String,
    pub iat: i32,
    pub exp: i32,
    pub jti: String,
}
