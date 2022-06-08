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
            Ok((key, value)) => Some(InviteToken {
                token: base64::encode(key),
                used: false,
            }),
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
    db.insert(&key, b"some content")
        .or(Err("Database error".to_string()))?;
    Ok(InviteToken {
        token: base64::encode(key),
        used: false,
    })
}

pub fn delete(db: sled::Tree, username: String, invite: &str) -> Result<(), String> {
    let bkey = match base64::decode(invite) {
        Err(_) => return Err("Invalid key".to_string()),
        Ok(a) => a,
    };
    let key = std::str::from_utf8(&bkey).or(Err("Invalid key".to_string()))?;

    if !key.starts_with(&format!("{}:", username)) {
        return Err("Invalid key".to_string());
    }

    // /!\ infinit recursion
    db.remove(&key)
        .or(Err("Database error".to_string()))?
        .ok_or("Invalid token".to_string())?;
    Ok(())
}

pub fn uze(db: &sled::Tree, invite: &str, _username: &str) -> Result<(), String> {
    let invite = base64::decode(invite).map_err(|_| "Invalid invite".to_string())?;
    let res = db
        .get(invite)
        .map_err(|_| "database error".to_string())?
        .ok_or("Invalid invite".to_string())?;
    // TODO
    Ok(())
}
