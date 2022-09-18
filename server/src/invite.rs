use crate::get_now_plus;
use proto::prost::Message;
use proto::server::user::InviteToken;
use rand::distributions::Alphanumeric;
use rand::Rng;

// key : username:randomstring
// out -> base64(key)

pub fn get(db: &sled::Tree, username: &str) -> Result<Vec<InviteToken>, String> {
    Ok(db
        .scan_prefix(format!("{}:", username))
        .filter_map(|entry| match entry {
            Ok((key, value)) => {
                let mut token = InviteToken::decode(value.as_ref()).expect("Invalid invite token.");
                token.token = base64::encode(key);
                Some(token)
            }
            Err(_) => None,
        })
        .collect())
}

pub fn create(db: &sled::Tree, user: &str) -> Result<InviteToken, String> {
    // let username = base64::encode(user);
    assert!(user.len() < 10);
    let salt: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(15 - user.len())
        .map(char::from)
        .collect();
    let key = format!("{}:{}", user, salt);

    // /!\ infinit recursion
    if db.contains_key(&key).or(Err("Database Error"))? {
        return create(db, user);
    }
    let mut token = InviteToken {
        token: String::new(),
        created: get_now_plus(0) as u32,
        used: 0,
        used_by: String::new(),
    };
    db.insert(&key, token.encode_to_vec())
        .or(Err("Database error".to_string()))?;
    token.token = base64::encode(key);
    Ok(token)
}

// Can only delete a not used one
pub fn delete(db: sled::Tree, username: String, invite: &str) -> Result<(), String> {
    let bkey = match base64::decode(invite) {
        Err(_) => return Err("Invalid key".to_string()),
        Ok(a) => a,
    };
    let key = std::str::from_utf8(&bkey).or(Err("Invalid key".to_string()))?;

    if !key.starts_with(&format!("{}:", username)) {
        return Err("Invalid key".to_string());
    }

    match db
        .update_and_fetch(&key, |a| match a {
            Some(data) => {
                let token = InviteToken::decode(data).expect("Invalid invide token.");
                if token.used == 0 {
                    None
                } else {
                    Some(token.encode_to_vec())
                }
            }
            None => None,
        })
        .map_err(|e| format!("Database error: {}", e))?
    {
        None => Ok(()),
        Some(_) => Err("Token already used".to_string()),
    }
}

pub fn uze(db: &sled::Tree, invite: &str, username: &str) -> Result<(), String> {
    let invite = base64::decode(invite).map_err(|_| "Invalid invite".to_string())?;
    // Return the previous token
    match db
        .fetch_and_update(invite, |a| match a {
            Some(data) => {
                let mut token = InviteToken::decode(data.as_ref()).expect("Invalid invite token");
                if token.used != 0 {
                    Some(token.encode_to_vec())
                } else {
                    token.used = get_now_plus(0) as u32;
                    token.used_by = username.to_string();
                    Some(token.encode_to_vec())
                }
            }
            None => None,
        })
        .map_err(|e| format!("database error: {}", e))?
    {
        None => Err("Invalid token".to_string()),
        Some(data) => {
            let token = InviteToken::decode(data.as_ref()).expect("Invalid invite token");
            if token.used != 0 {
                Err("Already used.".to_string())
            } else {
                Ok(())
            }
        }
    }
}
