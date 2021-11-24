use proto::server::auth::{
    auth_server::Auth, get_access_token_res, get_refresh_token_res, signup_res, GetAccessTokenReq,
    GetAccessTokenRes, GetRefreshTokenReq, GetRefreshTokenRes, SignupReq, SignupRes,
};
use tonic::{Code, Request, Response, Status};

use crate::jwt::Jwt;
use crate::refresh_token::RefreshToken;

type TonicResult<T> = Result<Response<T>, Status>;

const SALT: &str = "randomsalt";

pub struct Service {
    users: sled::Tree,
    jwt: Jwt,
    refresh_token: RefreshToken,
}

impl Service {
    pub fn new(users: sled::Tree, jwt: Jwt, refresh_token: RefreshToken) -> Self {
        Self {
            users,
            jwt,
            refresh_token,
        }
    }
}

#[tonic::async_trait]
impl Auth for Service {
    async fn get_refresh_token(
        &self,
        request: Request<GetRefreshTokenReq>,
    ) -> TonicResult<GetRefreshTokenRes> {
        let request = request.into_inner();
        let password = request.password;
        let username = request.username;
        if password.len() < 3 || username.len() < 3 {
            return Err(Status::new(
                Code::InvalidArgument,
                "Username or password invalid.",
            ));
        }
        let hash = match self.users.get(&username) {
            Ok(Some(users)) => users,
            _ => {
                return Err(Status::new(
                    Code::InvalidArgument,
                    "Username or password invalid.",
                ))
            }
        };

        if argon2::verify_encoded(&std::str::from_utf8(&hash).unwrap(), password.as_bytes())
            != Ok(true)
        {
            return Err(Status::new(
                Code::InvalidArgument,
                "Username or password invalid.",
            ));
        };

        let refresh_token = self.refresh_token.new_token(&username);

        Ok(Response::new(GetRefreshTokenRes {
            payload: Some(get_refresh_token_res::Payload::Ok(
                get_refresh_token_res::Ok {
                    refresh_token,
                    access_token: self.jwt.create_token(&username),
                    access_exp: Jwt::get_exp(),
                },
            )),
        }))
    }

    async fn get_access_token(
        &self,
        request: Request<GetAccessTokenReq>,
    ) -> TonicResult<GetAccessTokenRes> {
        let request = request.into_inner();
        let refresh_token = request.refresh_token;
        let username = request.username;

        if !self.refresh_token.verify(&username, &refresh_token) {
            return Err(Status::new(Code::InvalidArgument, "Invalid token"));
        }

        Ok(Response::new(GetAccessTokenRes {
            payload: Some(get_access_token_res::Payload::Ok(
                get_access_token_res::Ok {
                    access_token: self.jwt.create_token(&username),
                    exp: Jwt::get_exp(),
                },
            )),
        }))
    }

    async fn signup(&self, request: Request<SignupReq>) -> TonicResult<SignupRes> {
        let request = request.into_inner();
        let password = request.password;
        let username = request.username;
        let _invite = request.invite_code;

        if username.as_str().contains(":") {
            return Err(Status::new(
                Code::InvalidArgument,
                "Invalid username format",
            ));
        }

        match self.users.get(username.as_bytes()) {
            Ok(Some(_)) => {
                return Err(Status::new(Code::InvalidArgument, "Username already exist"))
            }
            Ok(None) => {}
            Err(e) => {
                return Err(Status::new(
                    Code::InvalidArgument,
                    format!("database error {}", e),
                ))
            }
        }
        let hash = match argon2::hash_encoded(
            password.as_bytes(),
            SALT.as_bytes(),
            &(argon2::Config::default()),
        ) {
            Ok(hash) => hash,
            _ => {
                return Err(Status::new(
                    Code::Unknown,
                    "Unknown error when hashing your password",
                ))
            }
        };
        self.users.insert(&username, hash.as_bytes());
        let refresh_token = self.refresh_token.new_token(&username);

        // TODO: create a new one

        Ok(Response::new(SignupRes {
            payload: Some(signup_res::Payload::Ok(signup_res::Ok {
                refresh_token,
                access_token: self.jwt.create_token(&username),
                access_exp: Jwt::get_exp(),
            })),
        }))
    }
}
