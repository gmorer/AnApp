use std::future::Future;
use tonic::metadata::MetadataValue;
use tonic::transport::{Channel, Endpoint};

use futures::lock::Mutex;
use proto::client::auth::{get_access_token_res, GetAccessTokenReq};
use proto::client::user::{
    change_password_res, create_invite_token_res, delete_refresh_token_res, get_invite_tokens_res,
    get_refresh_tokens_res, ChangePasswordReq, CreateInviteTokenReq, DeleteRefreshTokenReq,
    GetInviteTokensReq, GetRefreshTokensReq, InviteToken, RefreshToken,
};
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{SystemTime, UNIX_EPOCH};

mod auth;
use auth::{AuthApi, AuthClient};

mod user;
use user::UserApi;

const ADDR: &str = "http://127.0.0.1:5051";

type TonicRes<T> = Result<tonic::Response<T>, tonic::Status>;

#[derive(Debug, Clone)]
pub enum Error {
    Internal(String),
    ServerError(String),
    CredentialsError(String),
    DeAuth(String),
    Transport(tonic::transport::Error),
}

#[derive(Debug, Serialize, Deserialize)]
struct AccessTokenClaims {
    pub sub: String, /*  Username  */
    exp: usize,      /* expiration */
    iss: String,     /*   access   */
}

#[derive(Debug, Serialize, Deserialize)]
struct RefreshTokenClaims {
    pub sub: String, /*  Username  */
    exp: usize,      /* expiration */
    iss: String,     /*  refresh   */
}

// struct AuthIntercept(String);
// impl Interceptor for AuthInterceptor {
//     fn call(&mut self, request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
//         let token = self
//             .request
//             .get_mut()
//             .metadata_mut()
//             .insert("Authorization", self.0.parse.unwrap());
//     }
// }

#[derive(Debug, Clone)]
struct Creds {
    access_token: String,
    access_exp: u32,
    refresh_token: String,
    username: String,
    auth_client: AuthClient,
}

impl Creds {
    pub async fn priv_call<'a, F: 'a, Fut, A: 'a, C, R>(
        &mut self,
        client: &'a mut C,
        func: F,
        mut args: tonic::Request<A>,
    ) -> Result<TonicRes<R>, Error>
    where
        F: Fn(&'a mut C, tonic::Request<A>) -> Fut + Send + Sync,
        Fut: Future<Output = TonicRes<R>> + Send + 'a,
        A: Send,
        C: Send,
    {
        let access_token = {
            if self.access_exp < get_now() + 5 {
                let req = tonic::Request::new(GetAccessTokenReq {
                    refresh_token: creds.refresh_token.clone(),
                    username: creds.username.clone(),
                });
                let res = match self.auth_client.await.get_access_token(req).await {
                    Ok(res) => res,
                    Err(e) => return Err(Error::Internal(e.to_string())),
                };
                let (access_token, access_exp) = match res.into_inner().payload {
                    Some(get_access_token_res::Payload::Ok(bdy)) => (bdy.access_token, bdy.exp),
                    Some(get_access_token_res::Payload::Error(e)) => {
                        return Err(Error::ServerError(format!("{:?}", e)))
                    }
                    None => return Err(Error::Internal("Empty Payload".to_string())),
                };
                self.access_token = access_token.clone();
                self.access_exp = access_exp;
                MetadataValue::from_str(&access_token)
            } else {
                MetadataValue::from_str(&self.access_token)
            }
        };
        let access_token = access_token.map_err(|e| {
            Error::Internal(format!(
                "Cannot create grpc metadata from access token: {}",
                e
            ))
        })?;
        args.metadata_mut().insert("authorization", access_token);
        Ok(func(client, args).await)
    }
}

#[derive(Debug, Clone)]
pub struct Api {
    creds: Arc<Mutex<Creds>>,
    // _as_creds: Arc<AtomicBool>,
    // auth_client: AuthClient, // THe only not connected client
    channel: Channel,
    user_api: UserApi,

    // Could be removed
    auth_api: AuthApi,
}

fn get_now() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Error during timestamp manipulation")
        .as_secs() as u32
}

impl Api {
    async fn get_chan() -> Result<Channel, Error> {
        Endpoint::from_static(ADDR)
            .connect()
            .await
            .map_err(|e| e.to_string())?;
    }

    // 3 differents constructor: connecting, login, sign up

    pub async fn connect() -> Result<Self, Error> {
        unimplemented!();
    }

    pub async fn login(username: &str, password: &str) -> Result<Self, Error> {
        let channel = Self::get_chan()?;
        let creds = auth_api.login(channel, username, password)?;
        let creds = Arc::new(Mutex::new(creds));
        let user_api = UserApi::new(channel)?;
        Ok(Self {
            creds,
            auth_api,
            user_api,
            channel,
        })
    }

    pub async fn signup(username: &str, password: &str, invite: &str) -> Result<Self, Error> {
        let channel = Self::get_chan()?;
        let creds = auth_api.signup(channel, username, password, invite)?;
        let creds = Arc::new(Mutex::new(creds));
        let user_api = UserApi::new(channel)?;
        Ok(Self {
            creds,
            auth_api,
            user_api,
            channel,
        })
    }

    pub async fn username(&self) -> String {
        self.creds
            .lock()
            .await
            .as_ref()
            .map_or("Unknown".to_string(), |c| c.username.clone())
    }

    pub async fn logout(self) {
        let mut creds = self.creds.lock().await;
        *creds = None;
        // TODO: delete refresh token
    }

    // pub fn as_creds(&self) -> bool {
    //     self._as_creds.load(Ordering::Relaxed)
    // }

    // async fn get_access_token(
    //     &self,
    //     username: String,
    //     refresh_token: Option<String>,
    // ) -> Result<String, Error> {
    //     let refresh_token = if let Some(_refresh_token) = refresh_token {
    //         _refresh_token
    //     } else {
    //         // Look for the local token
    //         let (access, refresh) = match &*(self.creds.lock().await) {
    //             Some(creds) => (creds.access_token.clone(), creds.refresh_token.clone()),
    //             None => return Err(Error::CredentialsError("No credentials".to_string())),
    //         };
    //         let claims = match jsonwebtoken::dangerous_insecure_decode::<AccessTokenClaims>(&access)
    //         {
    //             Ok(claims) => claims,
    //             Err(e) => return Err(Error::Internal(e.to_string())),
    //         };
    //         // +5 second
    //         if claims.claims.exp > get_now() as usize + 5 {
    //             return Ok(access);
    //         }
    //         // Local token outdated, ask for a new one
    //         refresh
    //     };
    //     let req = tonic::Request::new(GetAccessTokenReq {
    //         refresh_token,
    //         username,
    //     });
    //     let res = match self.auth_client.lock().await.get_access_token(req).await {
    //         Ok(res) => res,
    //         Err(e) => return Err(Error::Internal(e.to_string())),
    //     };
    //     let access_token = match res.into_inner().payload {
    //         Some(get_access_token_res::Payload::Ok(bdy)) => bdy.access_token,
    //         Some(get_access_token_res::Payload::Error(e)) => {
    //             return Err(Error::ServerError(format!("{:?}", e)))
    //         }
    //         None => return Err(Error::Internal("Empty payload".to_string())),
    //     };
    //     Ok(access_token)
    // }

    // async fn get_user_client(&self) -> Result<UserClient, Error> {
    //     Ok(self
    //         .creds
    //         .lock()
    //         .await
    //         .as_ref()
    //         .ok_or(Error::CredentialsError("Not conected".to_string()))?
    //         .clients
    //         .user_client
    //         .lock()
    //         .await
    //         .clone())
    // }
}
