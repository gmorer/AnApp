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
        // TODO add informations to the token
        let res = self.db.insert(entry.as_bytes(), "TODO");
        token
    }
    pub fn verify(&self, username: &str, token: &str) -> bool {
        let entry = format!("{}:{}", username, token);
        let _pl = match self.db.get(entry.as_bytes()) {
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
    pub fn get_all(&self, username: &str) -> Vec<String> {
        let username = format!("{}:", username);
        let tokens = self
            .db
            .scan_prefix(username)
            .map(|entry| {
                let entry = entry.unwrap();
                let key = entry.0.as_ref();
                let key = std::str::from_utf8(key).unwrap();
                key[key.find(":").unwrap()..].to_string()
            }) // TODO: collect claims
            .collect();
        tokens
    }
}
