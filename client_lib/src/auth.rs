use crate::Error;
use crate::{Creds, Error};
use proto::client::auth::{get_refresh_token_res, signup_res, GetRefreshTokenReq, SignupReq};
use std::sync::Arc;
use tonic::transport::Channel;

pub type AuthClient = proto::client::auth::auth_client::AuthClient<Channel>;

pub struct AuthApi {
    client: AuthClient,
}

impl AuthApi {
    pub async fn login(
        channel: Channel,
        username: String,
        password: String,
    ) -> Result<Creds, Error> {
        let client = AuthClient::connect(channel).map_err(Error::Transport)?;
        let req = tonic::Request::new(GetRefreshTokenReq {
            username: username.clone(),
            password,
        });
        let res = client
            .get_refresh_token(req)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        let res = match res.into_inner().payload {
            Some(get_refresh_token_res::Payload::Ok(bdy)) => bdy,
            Some(get_refresh_token_res::Payload::Error(e)) => {
                return Err(Error::ServerError(format!("{:?}", e)))
            }
            None => return Err(Error::Internal("Empty payload".to_string())),
        };
        Ok(Creds {
            auth_client: client,
            username,
            refresh_token: res.refresh_token,
            access_token: res.access_token,
            access_exp: res.access_exp,
        })
    }

    pub async fn signup(
        channel: Channel,
        username: String,
        password: String,
        invite_code: String,
    ) -> Result<Creds, Error> {
        let client = AuthClient::connect(channel).map_err(Error::Transport)?;
        let req = tonic::Request::new(SignupReq {
            username: username.clone(),
            password,
            invite_code,
        });
        let res = client
            .signup(req)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;

        let res = match res.into_inner().payload {
            Some(signup_res::Payload::Ok(bdy)) => bdy,
            Some(signup_res::Payload::Error(e)) => {
                return Err(Error::ServerError(format!("{:?}", e)))
            }
            None => return Err(Error::Internal("Empty Payload".to_string())),
        };
        Ok(Creds {
            auth_client: client,
            username,
            refresh_token: res.refresh_token,
            access_token: res.access_token,
            access_exp: res.access_exp,
        })
    }
}
