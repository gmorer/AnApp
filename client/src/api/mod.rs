use proto::client::auth::{
    get_access_token_res, get_refresh_token_res, signup_res, GetAccessTokenReq, GetRefreshTokenReq,
    SignupReq,
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

#[derive(Debug, Clone)]
pub enum Error {
    InternalError(String),
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
    auth_client: Arc<Mutex<AuthClient>>,
}

#[derive(Debug, Clone)]
struct Creds {
    access_token: String,
    refresh_token: String,
    username: String,
}

#[derive(Debug, Clone)]
pub struct Api {
    clients: Clients,
    creds: Arc<Mutex<Option<Creds>>>,
    _as_creds: Arc<AtomicBool>,
}

fn get_now() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Error during timestamp manipulation")
        .as_secs() as usize
}

impl Api {
    pub async fn connect() -> Result<Self, String> {
        let (auth, _) = futures::try_join!(AuthClient::connect(ADDR), async { Ok(1) })
            .map_err(|e| e.to_string())?;
        let clients = Clients {
            auth_client: Arc::new(Mutex::new(auth)),
        };
        Ok(Self {
            creds: Arc::new(Mutex::new(None)),
            _as_creds: Arc::new(AtomicBool::new(false)),
            clients,
        })
    }

    pub async fn logout(&mut self) {
        let mut creds = self.creds.lock().await;
        *creds = None;
        self._as_creds.store(false, Ordering::Relaxed);
    }

    pub fn as_creds(&self) -> bool {
        self._as_creds.load(Ordering::Relaxed)
    }

    async fn get_access_token(
        &self,
        username: String,
        refresh_token: Option<String>,
    ) -> Result<String, Error> {
        let refresh_token = if let Some(_refresh_token) = refresh_token {
            _refresh_token
        } else {
            // Look for the local token
            let creds = match &*(self.creds.lock().await) {
                Some(creds) => creds.clone(),
                None => return Err(Error::CredentialsError("No credentials".to_string())),
            };
            let claims = match jsonwebtoken::dangerous_insecure_decode::<AccessTokenClaims>(
                &creds.access_token,
            ) {
                Ok(claims) => claims,
                Err(e) => return Err(Error::InternalError(e.to_string())),
            };
            // +5 second
            if claims.claims.exp > get_now() + 5 {
                return Ok(creds.access_token.clone());
            }
            // Local token outdated, ask for a new one
            let claims = match jsonwebtoken::dangerous_insecure_decode::<AccessTokenClaims>(
                &creds.refresh_token,
            ) {
                Ok(claims) => claims,
                Err(e) => return Err(Error::InternalError(e.to_string())),
            };
            if claims.claims.exp < get_now() + 1 {
                return Err(Error::DeAuth("Refresh token outdated".to_string()));
            }
            creds.refresh_token.clone()
        };
        let req = tonic::Request::new(GetAccessTokenReq {
            refresh_token,
            username,
        });
        let res = match self
            .clients
            .auth_client
            .lock()
            .await
            .get_access_token(req)
            .await
        {
            Ok(res) => res,
            Err(e) => return Err(Error::InternalError(e.to_string())),
        };
        let access_token = match res.into_inner().payload {
            Some(get_access_token_res::Payload::Ok(bdy)) => bdy.access_token,
            Some(get_access_token_res::Payload::Error(e)) => {
                return Err(Error::ServerError(format!("{:?}", e)))
            }
            None => return Err(Error::InternalError("Empty payload".to_string())),
        };
        Ok(access_token)
    }

    pub async fn login(&mut self, username: String, password: String) -> Result<(), Error> {
        let req = tonic::Request::new(GetRefreshTokenReq {
            username: username.clone(),
            password,
        });
        let res = match self
            .clients
            .auth_client
            .lock()
            .await
            .get_refresh_token(req)
            .await
        {
            Ok(res) => res,
            Err(e) => return Err(Error::InternalError(e.to_string())),
        };
        let refresh_token = match res.into_inner().payload {
            Some(get_refresh_token_res::Payload::Ok(bdy)) => bdy.refresh_token,
            Some(get_refresh_token_res::Payload::Error(e)) => {
                return Err(Error::ServerError(format!("{:?}", e)))
            }
            None => return Err(Error::InternalError("Empty payload".to_string())),
        };
        let access_token = self
            .get_access_token(username, Some(refresh_token.clone()))
            .await?;
        let username =
            match jsonwebtoken::dangerous_insecure_decode::<AccessTokenClaims>(&access_token) {
                Ok(claims) => claims.claims.sub,
                Err(e) => return Err(Error::InternalError(e.to_string())),
            };
        *(self.creds.lock().await) = Some(Creds {
            username,
            refresh_token,
            access_token,
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
        let res = match self.clients.auth_client.lock().await.signup(req).await {
            Ok(res) => res,
            Err(e) => return Err(Error::InternalError(e.to_string())),
        };

        let refresh_token = match res.into_inner().payload {
            Some(signup_res::Payload::Ok(bdy)) => bdy.refresh_token,
            Some(signup_res::Payload::Error(e)) => {
                return Err(Error::ServerError(format!("{:?}", e)))
            }
            None => return Err(Error::InternalError("Empty Payload".to_string())),
        };
        let access_token = self
            .get_access_token(username.clone(), Some(refresh_token.clone()))
            .await?;
        let username =
            match jsonwebtoken::dangerous_insecure_decode::<AccessTokenClaims>(&access_token) {
                Ok(claims) => claims.claims.sub,
                Err(e) => return Err(Error::InternalError(e.to_string())),
            };
        *(self.creds.lock().await) = Some(Creds {
            username,
            refresh_token,
            access_token,
        });
        self._as_creds.store(true, Ordering::Relaxed);
        Ok(())
    }
}
