use proto::prost::Message;
use proto::server::user::Invite;
use rand::distributions::Alphanumeric;
use rand::Rng;

// key : username:randomstring
// out -> base64(key)

pub fn get(db: sled::Tree, username: String) -> Result<String, String> {
    let r = db.scan_prefix(format!("{}:", username));
    while let Some(Ok((key, val))) = r.next() {
        // for (key, val) in r {
        let key = base64::encode(key);
        println!(
            "key: {}, val: {}",
            String::from_utf8_lossy(key.as_ref()),
            String::from_utf8_lossy(val.as_ref())
        );
    }
    Ok("olelele".to_string())
}

pub fn create(db: sled::Tree, username: String) -> Result<String, String> {
    let username = base64::encode(username);
    assert!(username.len() < 10);
    let salt: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20 - username.len())
        .map(char::from)
        .collect();
    let key = format!("{}:{}", username, salt);

    // /!\ infinit recursion
    if db.contains_key(key).or(Err("Database Error"))? {
        return create(db, username);
    }
    db.insert(key, b"some content")
        .or(Err("Database error".to_string()))
        .and(Ok("Invite Created".to_string()))
}

pub fn delete(db: sled::Tree, username: String, key: String) -> Result<String, String> {
    let username = base64::encode(username);
    if !key.starts_with(&username) {
        return Err("This is not your invite".to_string());
    }

    db.remove(key)
        .map(|res| match res {
            Some(_) => "Done".to_string(),
            None => "This invite does not exist".to_string(),
        })
        .or(Err("Database error".to_string()))
}

pub fn uze(db: sled::Tree, invite: &str, username: &str) -> Result<(), String> {
    let invite = base64::decode(invite).map_err(|_| "Invalid invite".to_string())?;
    let res = db
        .get(invite)
        .map_err(|_| "database error".to_string())?
        .ok_or("Invalid inite".to_string())?;
    let db_content = Invite::decode(res.as_ref()).expect("DB DECODE ERROR");
    if !db_content.username.is_empty() {}
    // Serialize res into proto obj
    // Verify not used
    // Update it
    Ok(())
}
