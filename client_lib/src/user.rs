use tonic::transport::Channel;
use crate::Error;

type UserClient = proto::client::user::user_client::UserClient<Channel>;

struct UserApi {
	client: UserClient,
}

impl UserApi {
	pub async fn new(channel: Channel) -> Result<Self, Error> {
		let client = UserClient::connect(channel).await.map_err(Error::Transport)?;
		OK(Self { client })
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
