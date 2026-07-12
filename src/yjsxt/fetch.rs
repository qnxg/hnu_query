use crate::yjsxt::error::TokenExpired;

pub trait YjsxtResponse {
    fn check_token_expired(self) -> Result<Self, crate::Error<TokenExpired>>
    where
        Self: Sized;
}

impl YjsxtResponse for reqwest::Response {
    fn check_token_expired(self) -> Result<Self, crate::Error<TokenExpired>> {
        if self.status() == reqwest::StatusCode::FOUND {
            return Err(crate::Error::Other(TokenExpired));
        }
        Ok(self)
    }
}
