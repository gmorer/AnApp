use crate::get_now_plus;
use proto::prost::Message;
use proto::server::user::RefreshToken as RefreshTokenPb;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::iter;
use std::vec::Vec;

// token in the db: "[username]:[token]" // cons: double source of truth, no use of the valu field

// custom token : "[username.base64][randomstring?]" // hard to secure (or remove : in username)

#[derive(Clone)]
pub struct RefreshToken {
    db: sled::Tree,
}

impl RefreshToken {
    pub fn new(db: sled::Tree) -> Self {
        Self { db }
    }
    pub fn new_token(&self, username: &str) -> String {
        let mut rng = thread_rng();
        let token: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .take(15)
            .collect();
        let entry = format!("{}:{}", username, token);
        let now = get_now_plus(0);
        // TODO add informations to the token
        let token_pb = RefreshTokenPb {
            token: "".to_string(), // Not use again
            from: "somewhere".to_string(),
            creation_date: now as u32,
            expiration_date: 0, // TODO
            last_use: now as u32,
        };
        let _res = self.db.insert(entry.as_bytes(), token_pb.encode_to_vec());
        token
    }
    pub fn verify(&self, username: &str, token: &str) -> bool {
        let entry = format!("{}:{}", username, token);
        let now = get_now_plus(0);
        let _pl = match self.db.update_and_fetch(entry.as_bytes(), |token| {
            if let Some(token) = token {
                let mut token =
                    RefreshTokenPb::decode(token).expect("malformated RefreshToken in the db");
                token.last_use = now as u32;
                Some(token.encode_to_vec())
            } else {
                None
            }
        }) {
            Ok(Some(pl)) => pl,
            Ok(None) => return false,
            Err(e) => {
                println!("Error: {}", e);
                return false; // TODO: handle errros
            }
        };
        true
    }
    pub fn delete(&self, username: &str, token: &str) {
        let entry = format!("{}:{}", username, token);
        self.db.remove(entry.as_bytes());
    }
    pub fn get_all(&self, username: &str) -> Vec<RefreshTokenPb> {
        let username = format!("{}:", username);
        let tokens = self
            .db
            .scan_prefix(username)
            .map(|entry| {
                let entry = entry.unwrap();
                let key = entry.0.as_ref();
                let key = std::str::from_utf8(key).unwrap();
                let key = key[key.find(":").unwrap() + 1..].to_string();
                let mut token = RefreshTokenPb::decode(entry.1.as_ref())
                    .expect("malformated RefreshToken inthe db");
                token.token = key;
                token
            })
            .collect();
        tokens
    }
}
