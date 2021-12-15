use proto::server::user::{
    get_refresh_tokens_res, user_server::User, GetRefreshTokensReq, GetRefreshTokensRes,
    RefreshToken as RefreshTokenPb,
};
use tonic::{Request, Response, Status};

use crate::jwt::AccessTokenClaims;
use crate::refresh_token::RefreshToken;

type TonicResult<T> = Result<Response<T>, Status>;

pub struct Service {
    refresh_token: RefreshToken,
}

impl Service {
    pub fn new(refresh_token: RefreshToken) -> Self {
        Self { refresh_token }
    }
}

#[tonic::async_trait]
impl User for Service {
    async fn get_refresh_tokens(
        &self,
        request: Request<GetRefreshTokensReq>,
    ) -> TonicResult<GetRefreshTokensRes> {
        let claims = request.extensions().get::<AccessTokenClaims>().unwrap();
        let tokens = self.refresh_token.get_all(&claims.sub);
        Ok(Response::new(GetRefreshTokensRes {
            payload: Some(get_refresh_tokens_res::Payload::Ok(
                get_refresh_tokens_res::Ok {
                    refresh_tokens: tokens,
                },
            )),
        }))
    }
}
