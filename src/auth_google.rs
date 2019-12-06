use failure;
use jsonwebtoken as jwt;
use jwt::{Algorithm, Validation};
use reqwest;
use std::sync::Arc;

#[derive(Clone)]
pub struct GoogleSignin {
    client: Arc<reqwest::Client>,
    secret: String,
}

#[derive(Deserialize, Debug)]
struct JWK {
    alg: String,
    n: String,
    kid: String,
    e: String,
    kty: String,
    r#use: String,
}

#[derive(Deserialize, Debug)]
struct Certs {
    keys: Vec<JWK>,
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

impl GoogleSignin {
    pub fn new(secret: &str) -> GoogleSignin {
        GoogleSignin {
            client: Arc::new(reqwest::Client::new()),
            secret: secret.to_owned(),
        }
    }

    fn get_certs(&self) -> reqwest::Result<Certs> {
        let url = "https://www.googleapis.com/oauth2/v3/certs";
        let json = self.client.get(url).send()?.json();
        // println!("get_certs json: {:?}", json);
        json
    }

    pub fn decode_token(&self, token: &str) -> Result<GooglePayload, failure::Error> {
        let certs: Certs = self.get_certs()?;

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
