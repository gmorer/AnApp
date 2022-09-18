use crate::{ClientCreds, Error};
use proto::client::user::{
    change_password_res, create_invite_token_res, delete_refresh_token_res, get_invite_tokens_res,
    get_refresh_tokens_res, ChangePasswordReq, CreateInviteTokenReq, DeleteRefreshTokenReq,
    GetInviteTokensReq, GetRefreshTokensReq, InviteToken, RefreshToken,
};
use tonic::transport::Channel;
type UserClient = proto::client::user::user_client::UserClient<Channel>;

#[derive(Debug, Clone)]
pub struct UserApi {
    client: UserClient,
    creds: ClientCreds,
}

impl UserApi {
    pub(crate) async fn new(channel: Channel, creds: ClientCreds) -> Result<Self, Error> {
        let client = UserClient::new(channel);
        Ok(Self { client, creds })
    }

    pub async fn get_refresh_tokens(&mut self) -> Result<Vec<RefreshToken>, Error> {
        let req = tonic::Request::new(GetRefreshTokensReq {});
        let res = self
            .creds
            .lock()
            .await
            .priv_call(&mut self.client, &UserClient::get_refresh_tokens, req)
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
        match self
            .creds
            .lock()
            .await
            .priv_call(&mut self.client, &UserClient::change_password, req)
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
        match self
            .creds
            .lock()
            .await
            .priv_call(&mut self.client, &UserClient::delete_refresh_token, req)
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
        match self
            .creds
            .lock()
            .await
            .priv_call(&mut self.client, &UserClient::create_invite_token, req)
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
        match self
            .creds
            .lock()
            .await
            .priv_call(&mut self.client, &UserClient::get_invite_tokens, req)
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
