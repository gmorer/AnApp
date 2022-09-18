use crate::{Creds, Error};
use proto::client::auth::{get_refresh_token_res, signup_res, GetRefreshTokenReq, SignupReq};
use tonic::transport::Channel;

pub type AuthClient = proto::client::auth::auth_client::AuthClient<Channel>;

pub(crate) async fn login(
    channel: Channel,
    username: &str,
    password: &str,
) -> Result<Creds, Error> {
    let mut client = AuthClient::new(channel);
    let req = tonic::Request::new(GetRefreshTokenReq {
        username: username.to_string(),
        password: password.to_string(),
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
        username: username.to_string(),
        refresh_token: res.refresh_token,
        access_token: res.access_token,
        access_exp: res.access_exp,
    })
}

pub(crate) async fn signup(
    channel: Channel,
    username: &str,
    password: &str,
    invite_code: &str,
) -> Result<Creds, Error> {
    let mut client = AuthClient::new(channel);
    let req = tonic::Request::new(SignupReq {
        username: username.to_string(),
        password: password.to_string(),
        invite_code: invite_code.to_string(),
    });
    let res = client
        .signup(req)
        .await
        .map_err(|e| Error::Internal(e.to_string()))?;

    let res = match res.into_inner().payload {
        Some(signup_res::Payload::Ok(bdy)) => bdy,
        Some(signup_res::Payload::Error(e)) => return Err(Error::ServerError(format!("{:?}", e))),
        None => return Err(Error::Internal("Empty Payload".to_string())),
    };
    Ok(Creds {
        auth_client: client,
        username: username.to_string(),
        refresh_token: res.refresh_token,
        access_token: res.access_token,
        access_exp: res.access_exp,
    })
}
