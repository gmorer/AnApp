use std::future::Future;
use tonic::metadata::MetadataValue;
use tonic::transport::{Channel, Endpoint};

use proto::client::auth::{
    get_access_token_res, get_refresh_token_res, signup_res, GetAccessTokenReq, GetRefreshTokenReq,
    SignupReq,
};
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
use tokio::sync::Mutex;

const ADDR: &str = "http://127.0.0.1:5051";

type AuthClient = proto::client::auth::auth_client::AuthClient<tonic::transport::Channel>;
type UserClient = proto::client::user::user_client::UserClient<tonic::transport::Channel>;
type TonicRes<T> = Result<tonic::Response<T>, tonic::Status>;

#[derive(Debug, Clone)]
pub enum Error {
    Internal(String),
    ServerError(String),
    CredentialsError(String),
    DeAuth(String),
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

#[derive(Debug, Clone)]
struct Clients {
    user_client: Arc<Mutex<UserClient>>,
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

impl Clients {
    pub fn new(channel: Channel) -> Self {
        Self {
            user_client: Arc::new(Mutex::new(UserClient::new(channel))),
        }
    }
}

#[derive(Debug, Clone)]
struct Creds {
    access_token: String,
    access_exp: u32,
    refresh_token: String,
    username: String,
    clients: Clients, // Connected clients
}

#[derive(Debug, Clone)]
pub struct Api {
    creds: Arc<Mutex<Option<Creds>>>,
    _as_creds: Arc<AtomicBool>,
    auth_client: Arc<Mutex<AuthClient>>, // THe only not connected client
    channel: Channel,
}

fn get_now() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Error during timestamp manipulation")
        .as_secs() as u32
}

impl Api {
    pub async fn connect() -> Result<Self, String> {
        let channel = Endpoint::from_static(ADDR)
            .connect()
            .await
            .map_err(|e| e.to_string())?;
        let auth_client = Arc::new(Mutex::new(AuthClient::new(channel.clone())));
        Ok(Self {
            creds: Arc::new(Mutex::new(None)),
            _as_creds: Arc::new(AtomicBool::new(false)),
            auth_client,
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

    pub async fn logout(&mut self) {
        let mut creds = self.creds.lock().await;
        *creds = None;
        self._as_creds.store(false, Ordering::Relaxed);
    }

    pub fn as_creds(&self) -> bool {
        self._as_creds.load(Ordering::Relaxed)
    }

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

    pub async fn login(&mut self, username: String, password: String) -> Result<(), Error> {
        let req = tonic::Request::new(GetRefreshTokenReq {
            username: username.clone(),
            password,
        });
        let res = self
            .auth_client
            .lock()
            .await
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
        *(self.creds.lock().await) = Some(Creds {
            clients: Clients::new(self.channel.clone()),
            username,
            refresh_token: res.refresh_token,
            access_token: res.access_token,
            access_exp: res.access_exp,
        });
        self._as_creds.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub async fn signup(
        &mut self,
        username: String,
        password: String,
        invite_code: String,
    ) -> Result<(), Error> {
        let req = tonic::Request::new(SignupReq {
            username: username.clone(),
            password,
            invite_code,
        });
        let res = self
            .auth_client
            .lock()
            .await
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
        *(self.creds.lock().await) = Some(Creds {
            clients: Clients::new(self.channel.clone()),
            username,
            refresh_token: res.refresh_token,
            access_token: res.access_token,
            access_exp: res.access_exp,
        });
        self._as_creds.store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn priv_call<'a, F: 'a, Fut, A: 'a, C, R>(
        &self,
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
            let mut creds_out = self.creds.lock().await;
            let mut creds = creds_out
                .as_ref()
                .ok_or(Error::CredentialsError("Not conected".to_string()))?
                .clone();
            if creds.access_exp < get_now() + 5 {
                let req = tonic::Request::new(GetAccessTokenReq {
                    refresh_token: creds.refresh_token.clone(),
                    username: creds.username.clone(),
                });
                let res = match self.auth_client.lock().await.get_access_token(req).await {
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
                creds.access_token = access_token.clone();
                creds.access_exp = access_exp;
                *creds_out = Some(creds);
                MetadataValue::from_str(&access_token)
            } else {
                MetadataValue::from_str(&creds.access_token)
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

    async fn get_user_client(&self) -> Result<UserClient, Error> {
        Ok(self
            .creds
            .lock()
            .await
            .as_ref()
            .ok_or(Error::CredentialsError("Not conected".to_string()))?
            .clients
            .user_client
            .lock()
            .await
            .clone())
    }

    pub async fn get_refresh_tokens(&mut self) -> Result<Vec<RefreshToken>, Error> {
        let req = tonic::Request::new(GetRefreshTokensReq {});
        let mut user_client = self.get_user_client().await?;
        let res = self
            .priv_call(&mut user_client, &UserClient::get_refresh_tokens, req)
            .await?
            .map_err(|e| Error::Internal(e.to_string()))?;
        let tokens = match res.into_inner().payload {
            Some(get_refresh_tokens_res::Payload::Ok(bdy)) => bdy.refresh_tokens,
            _ => return Err(Error::Internal("aaa".to_string())),
        };
        Ok(tokens)
    }

    pub async fn change_password(
        &mut self,
        old_password: String,
        new_password: String,
    ) -> Result<(), Error> {
        let req = tonic::Request::new(ChangePasswordReq {
            new_password,
            old_password,
        });
        let mut user_client = self.get_user_client().await?;
        match self
            .priv_call(&mut user_client, &UserClient::change_password, req)
            .await?
            .map_err(|e| Error::Internal(e.to_string()))?
            .into_inner()
            .payload
        {
            Some(change_password_res::Payload::Ok(_)) => Ok(()),
            _ => Err(Error::Internal("aaa".to_string())),
        }
    }

    pub async fn delete_refresh_token(&mut self, refresh_token: String) -> Result<(), Error> {
        let req = tonic::Request::new(DeleteRefreshTokenReq { refresh_token });
        let mut user_client = self.get_user_client().await?;
        match self
            .priv_call(&mut user_client, &UserClient::delete_refresh_token, req)
            .await?
            .map_err(|e| Error::Internal(e.to_string()))?
            .into_inner()
            .payload
        {
            Some(delete_refresh_token_res::Payload::Ok(_)) => Ok(()),
            _ => Err(Error::Internal("aaa".to_string())),
        }
    }

    pub async fn create_invite(&mut self) -> Result<InviteToken, Error> {
        let req = tonic::Request::new(CreateInviteTokenReq {});
        let mut user_client = self.get_user_client().await?;
        match self
            .priv_call(&mut user_client, &UserClient::create_invite_token, req)
            .await?
            .map_err(|e| Error::Internal(e.to_string()))?
            .into_inner()
            .payload
        {
            Some(create_invite_token_res::Payload::Ok(invite)) => match invite.token {
                Some(invite) => Ok(invite),
				_ => Err(Error::Internal("aaa".to_string())),
            },
            _ => Err(Error::Internal("aaa".to_string())),
        }
    }

    pub async fn get_invites(&mut self) -> Result<Vec<InviteToken>, Error> {
        let req = tonic::Request::new(GetInviteTokensReq {});
        let mut user_client = self.get_user_client().await?;
        match self
            .priv_call(&mut user_client, &UserClient::get_invite_tokens, req)
            .await?
            .map_err(|e| Error::Internal(e.to_string()))?
            .into_inner()
            .payload
        {
            Some(get_invite_tokens_res::Payload::Ok(invite)) => Ok(invite.tokens),
            _ => Err(Error::Internal("aaa".to_string())),
        }
    }
}
