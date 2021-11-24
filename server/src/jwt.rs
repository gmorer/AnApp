use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tonic::service::Interceptor;
/*
    Access token are used to access api endpoints it live only 10 minutes
    sub: username
    exp: timestamp of the date generated plus 10 minutes
*/

/*
    Refresh token are used to generate new Access token
    sub: username
    exp: timestamp of the date generated plus 1 month
    iss: ID of the token ( can be blacklisted ) if token not in database < ID >
*/

// TODO: .env file
const SECRET_KEY: &str = "super secret";

const TOKEN_DURATION: u32 = 60 * 10; /* 10 minutes in seconds */

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: String, /*  Username  */
    pub exp: usize,  /* expiration */
    pub iss: String, /*   access   */
}

fn get_now_plus(exp: u32) -> usize {
    SystemTime::now()
        .checked_add(Duration::from_secs(exp as u64))
        .expect("Error during timestamp manipulation")
        .duration_since(UNIX_EPOCH)
        .expect("Error during timestamp manipulation")
        .as_secs() as usize
}

#[derive(Clone)]
pub struct Jwt {
    decode_key: DecodingKey<'static>, // Will be fix in next version of jwt
    encode_key: EncodingKey,
    validation: Validation,
    header: Header,
}

impl Jwt {
    pub fn new() -> Self {
        Self {
            decode_key: DecodingKey::from_secret(SECRET_KEY.as_ref()),
            encode_key: EncodingKey::from_secret(SECRET_KEY.as_ref()),
            validation: Validation::default(),
            header: Header::default(),
        }
    }

    pub fn get_exp() -> u32 {
        get_now_plus(TOKEN_DURATION) as u32
    }

    pub fn create_token(&self, username: &str) -> String {
        encode(
            &self.header,
            &AccessTokenClaims {
                sub: username.to_string(),
                exp: get_now_plus(TOKEN_DURATION),
                iss: "access".to_string(),
            },
            &self.encode_key,
        )
        .unwrap()
    }
}

impl Interceptor for Jwt {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        let token = match request.metadata().get("Authorization") {
            Some(token) => token.to_str().unwrap(),
            None => {
                return Err(tonic::Status::new(
                    tonic::Code::PermissionDenied,
                    "Missing credentials",
                ));
            }
        };
        let token = match decode::<AccessTokenClaims>(token, &self.decode_key, &self.validation) {
            Ok(token) => token,
            Err(_) => {
                return Err(tonic::Status::new(
                    tonic::Code::PermissionDenied,
                    "Invalid token",
                ))
            }
        };
        if token.claims.exp < get_now_plus(0) {
            return Err(tonic::Status::new(
                tonic::Code::PermissionDenied,
                "Expired credentials",
            ));
        }
        if token.claims.iss != "access" {
            return Err(tonic::Status::new(
                tonic::Code::PermissionDenied,
                "Invalid token",
            ));
        }
        request.extensions_mut().insert(token.claims);
        Ok(request)
    }
}
