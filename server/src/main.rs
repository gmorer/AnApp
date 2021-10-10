// use std::sync::Arc;
use proto::server::auth::auth_server::AuthServer;
use tonic::transport::Server;

mod jwt;

mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let jwt = jwt::Jwt::new();

    let db: sled::Db = sled::open("my_db").expect("cannot open the database");

    // let invites_db = Arc::new(db.open_tree("invites").expect("cannot open the invite database"));
    let users_db = db
        .open_tree("invites")
        .expect("cannot open the invite database");

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
    let auth_svc = AuthServer::new(services::auth::Service::new(users_db, jwt));
    //let users_svc = HelloServer::with_interceptor(users::Service::new(users_db), check_auth);

    Server::builder()
        .accept_http1(true)
        .add_service(tweb_config.enable(auth_svc))
        // .add_service(echo_svc)
        .serve("127.0.0.1:5051".parse().unwrap())
        .await?;

    Ok(())
}
