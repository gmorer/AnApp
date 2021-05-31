use jsonwebtoken::{ encode, EncodingKey, Header, Validation, DecodingKey, decode };
use serde::{ Deserialize, Serialize };
use std::time::{ Duration, SystemTime, UNIX_EPOCH };
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
const ACCESS_SECRET: &str = "super secret";
const REFRESH_SECRET: &str = "another secret";

const ACCESS_TOKEN_DURATION: u64 = 60 * 10; /* 10 minutes in seconds */
const REFRESH_TOKEN_DURATION: u64 = 60 * 60 * 24 * 30; /* 1 month in seconds */

#[derive(Debug, Serialize, Deserialize)]
struct AccessTokenClaims {
	sub: String, /*  Username  */
	exp: usize,  /* expiration */
}

#[derive(Debug, Serialize, Deserialize)]
struct RefreshTokenClaims {
	pub sub: String, /*  Username  */
	exp: usize,  /* expiration */
	iss: String, /*  Token id  */
}

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
pub struct Jwt<'a> {
	decode_key: DecodingKey<'a>,
	encode_key: EncodingKey,
	validation: Validation,
	header: Header
}
impl<'a> Jwt<'a> {
	pub fn new() -> Self {
		Self {
			decode_key: DecodingKey::from_secret(ACCESS_SECRET.as_ref()),
			encode_key: EncodingKey::from_secret(REFRESH_SECRET.as_ref()),
			validation: Validation::default(),
			header: Header::default()
		}
	}

	pub fn create_token(&self, username: String, token: TokenType) -> String {
		match token {
			TokenType::AccessToken => encode(
				&self.header,
				&AccessTokenClaims {
					sub: username,
					exp: get_now_plus(ACCESS_TOKEN_DURATION),
				},
				&self.encode_key
			),
			TokenType::RefreshToken => encode(
				&self.header,
				&RefreshTokenClaims {
					sub: username,
					exp: get_now_plus(REFRESH_TOKEN_DURATION),
					iss: "RandomID".to_string(),
				},
				&self.encode_key
			),
		}
		.expect("Error during token creation")
	}

	pub fn get_username(&self, header: &str) -> Result<String, String> {
		if !header.starts_with("bearer ") {
			return Err("no bearer in the Authorization header".to_string());
		}
		let header = &header["bearer ".len()..];
		let token = decode::<AccessTokenClaims>(&header, &self.decode_key, &self.validation).map_err(|e| format!("Invalid Token: {}", e))?;
		Ok(token.claims.sub)
	}
}
/* pub fn validate token */

/*
	? Refresh token in the midleware ?
	Midleware Data: Validation, decodingKey
*/