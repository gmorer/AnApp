use proto::server::user as userpb;

use tonic::{Code, Request, Response, Status};

use crate::jwt::AccessTokenClaims;
use crate::refresh_token::RefreshToken;

type TonicResult<T> = Result<Response<T>, Status>;

pub struct Service {
    refresh_token: RefreshToken,
    users: sled::Tree,
}

impl Service {
    pub fn new(refresh_token: RefreshToken, users: sled::Tree) -> Self {
        Self {
            refresh_token,
            users,
        }
    }
}

#[tonic::async_trait]
impl userpb::user_server::User for Service {
    async fn get_refresh_tokens(
        &self,
        request: Request<userpb::GetRefreshTokensReq>,
    ) -> TonicResult<userpb::GetRefreshTokensRes> {
        let claims = request.extensions().get::<AccessTokenClaims>().unwrap();
        let tokens = self.refresh_token.get_all(&claims.sub);
        Ok(Response::new(userpb::GetRefreshTokensRes {
            payload: Some(userpb::get_refresh_tokens_res::Payload::Ok(
                userpb::get_refresh_tokens_res::Ok {
                    refresh_tokens: tokens,
                },
            )),
        }))
    }

    async fn delete_refresh_token(
        &self,
        request: Request<userpb::DeleteRefreshTokenReq>,
    ) -> TonicResult<userpb::DeleteRefreshTokenRes> {
        let username = &request
            .extensions()
            .get::<AccessTokenClaims>()
            .unwrap()
            .sub
            .clone();
        let request = request.into_inner();
        let token = request.refresh_token;
        self.refresh_token.delete(username, &token);
        Ok(Response::new(userpb::DeleteRefreshTokenRes {
            payload: Some(userpb::delete_refresh_token_res::Payload::Ok(
                userpb::delete_refresh_token_res::Ok {},
            )),
        }))
    }

    async fn change_password(
        &self,
        request: Request<userpb::ChangePasswordReq>,
    ) -> TonicResult<userpb::ChangePasswordRes> {
        let username = &request
            .extensions()
            .get::<AccessTokenClaims>()
            .unwrap()
            .sub
            .clone();
        let request = request.into_inner();
        let old_password = request.old_password;
        let new_password = request.new_password;
        if new_password.len() < 3 {
            return Err(Status::new(Code::InvalidArgument, "Username invalid."));
        }
        let hash = match self.users.get(&username) {
            Ok(Some(users)) => users,
            _ => Err(Status::new(Code::InvalidArgument, "User does not exist"))?,
        };
        if argon2::verify_encoded(
            &std::str::from_utf8(&hash).unwrap(),
            old_password.as_bytes(),
        ) != Ok(true)
        {
            Err(Status::new(Code::InvalidArgument, "Invalid new password"))?;
        };
        let new_hash = match argon2::hash_encoded(
            new_password.as_bytes(),
            crate::SALT.as_bytes(),
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
        self.users
            .insert(&username, new_hash.as_bytes())
            .map_err(|_| Status::new(Code::Unknown, "Cannot insert new password"))?;
        Ok(Response::new(userpb::ChangePasswordRes {
            payload: Some(userpb::change_password_res::Payload::Ok(
                userpb::change_password_res::Ok {},
            )),
        }))
    }
}
