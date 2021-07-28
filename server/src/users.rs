use tonic::{ Code, Request, Response, Status};
use proto::server::users::{
    login_res, signup_res,
    users_server::Users,
    HelloReq, HelloRes, LoginReq, LoginRes, SignupReq, SignupRes,
};

type TonicResult<T> = Result<Response<T>, Status>;

const SALT: &str = "randomsalt";

pub struct Service {
    users: sled::Tree
}

impl Service {
    pub fn new(users: sled::Tree) -> Self {
        Self {
            users
        }
    }

}
#[tonic::async_trait]
impl Users for Service {
    async fn say_hello(&self, _request: Request<HelloReq>) -> TonicResult<HelloRes> {
        Ok(Response::new(HelloRes {
            message: "yolo".to_string(),
        }))
    }

    async fn login(&self, request: Request<LoginReq>) -> TonicResult<LoginRes> {
        let request = request.into_inner();
        let password = request.password;
        let username = request.username;
        let hash = match self.users.get(username) {
            Ok(Some(users)) => users,
            _ => return Err(Status::new(Code::InvalidArgument, "Username or password invalid."))
        };

        if argon2::verify_encoded(&std::str::from_utf8(&hash).unwrap(), password.as_bytes()) != Ok(true) {
            return Err(Status::new(Code::InvalidArgument, "Username or password invalid."));
        };

        Ok(Response::new(LoginRes {
            payload: Some(login_res::Payload::Ok(login_res::Ok{
                access_token: "blabla".to_string(),
                refresh_token: "blabla".to_string(),
            }))
        }))
    }
    async fn signup(&self, request: Request<SignupReq>) -> TonicResult<SignupRes> {
        let request = request.into_inner();
        let password = request.password;
        let username = request.username.as_bytes();
        let _invite = request.invite_code;

        match self.users.get(username) {
            Ok(Some(_)) => {},
            _ => return Err(Status::new(Code::InvalidArgument, "Username already exist")),
        }
        let hash = match argon2::hash_encoded(password.as_bytes(), SALT.as_bytes(), &(argon2::Config::default())) {
            Ok(hash) => hash,
            _ => return Err(Status::new(Code::Unknown, "Unknown error when hashing your password")),
        };
        self.users.insert(username, hash.as_bytes());
        Ok(Response::new(SignupRes {
            payload: Some(signup_res::Payload::Ok(signup_res::Ok{
                access_token: "blabla".to_string(),
                refresh_token: "blabla".to_string(),
            }))
        }))
    }
}
