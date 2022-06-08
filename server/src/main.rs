// use std::sync::Arc;
use proto::server::{auth::auth_server::AuthServer, user::user_server::UserServer};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tonic::transport::Server;

mod jwt;
mod refresh_token;
use refresh_token::RefreshToken;
mod invite;

mod services;

const SALT: &str = "randomsalt";

pub fn get_now_plus(exp: u32) -> usize {
    SystemTime::now()
        .checked_add(Duration::from_secs(exp as u64))
        .expect("Error during timestamp manipulation")
        .duration_since(UNIX_EPOCH)
        .expect("Error during timestamp manipulation")
        .as_secs() as usize
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let jwt = jwt::Jwt::new();

    let db: sled::Db = sled::open("my_db").expect("cannot open the database");

    let invites_db = db
        .open_tree("invites")
        .expect("cannot open the invite database");

    let users_db = db
        .open_tree("users")
        .expect("cannot open the users database");

    let refresh_token_db = db
        .open_tree("users")
        .expect("cannot open the refresh_token_db database");
    let refresh_token = RefreshToken::new(refresh_token_db);

    let tweb_config = tonic_web::config()
        .allow_all_origins()
        .allow_credentials(true)
        .expose_headers(vec![
            "x-request-id",
            "content-type",
            "x-grpc-web",
            "x-user-agent",
        ]);

    // Server::builder()
    //     .accept_http1(true)
    //     .add_service(tonic_web::enable(echo_svc.clone()))
    // 	.add_service(echo_svc)
    //     .serve(addr)
    //     .await?;
    let auth_svc = AuthServer::new(services::auth::Service::new(
        users_db.clone(),
        jwt.clone(),
        refresh_token.clone(),
        invites_db.clone(),
    ));
    let user_svc = UserServer::with_interceptor(
        services::user::Service::new(refresh_token, users_db, invites_db),
        jwt,
    );
    //let users_svc = HelloServer::with_interceptor(users::Service::new(users_db), check_auth);

    Server::builder()
        .accept_http1(true)
        .add_service(tweb_config.enable(auth_svc))
        .add_service(tweb_config.enable(user_svc))
        // .add_service(echo_svc)
        .serve("127.0.0.1:5051".parse().unwrap())
        .await?;

    Ok(())
}
