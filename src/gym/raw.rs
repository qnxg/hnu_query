use crate::{
    error::{MapParseErr, MapUnexpectedErr, parse_err},
    gym::error::TokenExpired,
};
use reqwest::Response;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::error::Error as StdError;

#[derive(Deserialize, Debug, Serialize)]
pub struct BadGymResponse {
    pub data: Value,
    pub info: String,
    pub status: i64,
}

pub trait GymResponseExtractor {
    async fn extract_data<T: DeserializeOwned, E: StdError>(
        self,
    ) -> Result<Result<T, BadGymResponse>, crate::Error<E>>;
}

impl GymResponseExtractor for Response {
    async fn extract_data<T: DeserializeOwned, E: StdError>(
        self,
    ) -> Result<Result<T, BadGymResponse>, crate::Error<E>> {
        let body = self.text().await.unexpected_err()?;
        let res: BadGymResponse = serde_json::from_str(&body).parse_err(&body)?;
        if res.status == 1 {
            let data = serde_json::from_value::<T>(res.data).parse_err(&body)?;
            return Ok(Ok(data));
        }
        Ok(Err(res))
    }
}

pub trait GymResponse<T> {
    fn check_cache(self) -> Result<Self, crate::Error<TokenExpired>>
    where
        Self: Sized;
    fn into_result<E: StdError>(self) -> Result<T, crate::Error<E>>;
}

impl<T> GymResponse<T> for Result<T, BadGymResponse> {
    /// 检查该响应是否表明 cookie 过期，如果是的话则将 cookie 缓存清除
    fn check_cache(self) -> Result<Self, crate::Error<TokenExpired>> {
        // 典型的异常response body：
        // {"data":[],"info":"登录失效","status":-1}
        if let Err(ref bad_resp) = self
            && bad_resp.info.contains("登录失效")
        {
            return Err(crate::Error::Other(TokenExpired));
        }
        Ok(self)
    }
    fn into_result<E: StdError>(self) -> Result<T, crate::Error<E>> {
        match self {
            Ok(value) => Ok(value),
            Err(bad_resp) => {
                let data = serde_json::to_string(&bad_resp).expect("序列化失败");
                Err(parse_err(&data))
            }
        }
    }
}
