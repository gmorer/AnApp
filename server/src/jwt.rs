use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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

const ACCESS_TOKEN_DURATION: u64 = 60 * 10; /* 10 minutes in seconds */
const REFRESH_TOKEN_DURATION: u64 = 60 * 60 * 24 * 30; /* 1 month in seconds */

#[derive(Debug, Serialize, Deserialize)]
struct AccessTokenClaims {
    sub: String, /*  Username  */
    exp: usize,  /* expiration */
    iss: String, /*   access   */
}

#[derive(Debug, Serialize, Deserialize)]
struct RefreshTokenClaims {
    pub sub: String, /*  Username  */
    exp: usize,      /* expiration */
    iss: String,     /*  refresh   */
}

#[derive(PartialEq)]
pub enum TokenType {
    AccessToken,
    RefreshToken,
}

fn get_now_plus(exp: u64) -> usize {
    SystemTime::now()
        .checked_add(Duration::from_secs(exp))
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

    pub fn create_token(&self, username: &str, token: TokenType) -> String {
        match token {
            TokenType::AccessToken => encode(
                &self.header,
                &AccessTokenClaims {
                    sub: username.to_string(),
                    exp: get_now_plus(ACCESS_TOKEN_DURATION),
                    iss: "access".to_string(),
                },
                &self.encode_key,
            ),
            TokenType::RefreshToken => encode(
                &self.header,
                &RefreshTokenClaims {
                    sub: username.to_string(),
                    exp: get_now_plus(REFRESH_TOKEN_DURATION),
                    iss: "refresh".to_string(),
                },
                &self.encode_key,
            ),
        }
        .expect("Error during token creation")
    }

    pub fn is_token_valid(&self, token: &str, typ: TokenType) -> bool {
        let token = match decode::<AccessTokenClaims>(&token, &self.decode_key, &self.validation) {
            Ok(token) => token,
            Err(e) => {
                eprintln!("decode error: {} token: {}", e, token);
                return false;
            }
        };
        if typ == TokenType::RefreshToken && token.claims.iss != "refresh" {
            eprintln!("wrong type ");
            // Not a refresh one
            false
        } else if typ == TokenType::AccessToken && token.claims.iss != "access" {
            eprintln!("wrong type ");
            // Not a access one
            false
        } else if token.claims.exp < get_now_plus(0) {
            eprintln!("outdateed");
            // Expired one
            false
        } else {
            true
        }
    }

    pub fn get_username(&self, token: &str) -> Result<String, String> {
        // if !header.starts_with("bearer ") {
        //     return Err("no bearer in the Authorization header".to_string());
        // }
        // let header = &header["bearer ".len()..];
        let token = decode::<AccessTokenClaims>(token, &self.decode_key, &self.validation)
            .map_err(|e| format!("Invalid Token: {}", e))?;
        Ok(token.claims.sub)
    }
}
/* pub fn validate token */

/*
    ? Refresh token in the midleware ?
    Midleware Data: Validation, decodingKey
*/
