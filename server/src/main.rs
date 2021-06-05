// use std::sync::Arc;
use tonic::{ transport::Server, Response, Request, Status };
use proto::server::hello::{ hello_server::{ HelloServer, Hello}, HelloReq, HelloRes };

// mod jwt;
// mod invite;

// const STATIC_FOLDER: &str = "./static";

type TonicResult<T> = Result<Response<T>, Status>;

#[derive(Default)]
struct HelloService();


#[tonic::async_trait]
impl Hello for HelloService {
	async fn say_hello(&self, _request: Request<HelloReq>) -> TonicResult<HelloRes> {
		Ok(Response::new(HelloRes { message: "yolo".to_string() }))
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // GET /hello/warp => 200 OK with body "Hello, warp!"

	// let jwt = Arc::new(jwt::Jwt::new());

	// let db: sled::Db = sled::open("my_db").expect("cannot open the database");

	// let invites_db = Arc::new(db.open_tree("invites").expect("cannot open the invite database"));

	let tweb_config = tonic_web::config()
     .allow_all_origins()
     .allow_credentials(true)
     .expose_headers(vec!["x-request-id", "content-type","x-grpc-web","x-user-agent"]);

    // Server::builder()
    //     .accept_http1(true)
    //     .add_service(tonic_web::enable(echo_svc.clone()))
	// 	.add_service(echo_svc)
    //     .serve(addr)
    //     .await?;
	let echo_svc = HelloServer::new(HelloService::default());

	Server::builder()
        .accept_http1(true)
        .add_service(tweb_config.enable(echo_svc.clone()))
        // .add_service(echo_svc)
        .serve("127.0.0.1:5051".parse().unwrap())
        .await?;

    Ok(())

	// .and(with_db(db));
	// let public_api =
	// 	warp::path("register").and(warp::post()).and_then(register)
	// 		.or(warp::path("login").and(warp::post()).and_then(register))
	// 		.or(warp::path("refresh").and(warp::get()).and_then(register));
	
	// let private_api =
		// warp::path("invite").and(
		// 	warp::get().and_then(invite::get)
		// 	.or(warp::post().and_then(invite::create))
		// 	.or(warp::delete().and(warp::path::param::<String>()).and_then(invite::delete))
		// );
	// 	.or(warp::path("me").and(
	// 		warp::path("password").and(warp::put()).and_then(register)
	// 		.or(warp::path("username").and(warp::put()).and_then(register))
	// 	));


	// let api = warp::path("api").and(auth_wrapper(jwt)).map(|aa| format!("Hello, api! {} ", aa));
	// let examples = warp::path("/").and(warp::fs::dir("./static/"));

}
