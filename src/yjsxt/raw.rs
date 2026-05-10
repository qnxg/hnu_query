use crate::{
    error::{MapParseErr, MapUnexpectedErr},
    yjsxt::error::TokenExpired,
};
use aes::cipher::{BlockDecryptMut, KeyInit, block_padding::Pkcs7};
use base64::engine::{Engine, general_purpose::STANDARD as base64};
use serde::de::DeserializeOwned;

const GRADUATE_KEY: &str = "southsoft12345!#";

type Aes128EcbDec = ecb::Decryptor<aes::Aes128>;

pub fn graduate_decrypt(data: &str) -> Result<String, crate::Error<TokenExpired>> {
    let decode = base64
        .decode(data)
        .parse_err_with_reason(data, "base64 解码失败")?;
    let key = <aes::cipher::generic_array::GenericArray<u8, _>>::from_slice(
        &GRADUATE_KEY.as_bytes()[..16],
    );
    let res = Aes128EcbDec::new(key)
        .decrypt_padded_vec_mut::<Pkcs7>(&decode)
        .parse_err_with_reason(data, "AES 解密失败")?;
    String::from_utf8(res).parse_err_with_reason(data, "UTF-8 转换失败")
}

pub trait YjsxtResponseExtractor {
    async fn extract_data<T: DeserializeOwned>(
        self,
        decrypt: bool,
    ) -> Result<T, crate::Error<TokenExpired>>;
}

impl YjsxtResponseExtractor for reqwest::Response {
    async fn extract_data<T: DeserializeOwned>(
        self,
        decrypt: bool,
    ) -> Result<T, crate::Error<TokenExpired>> {
        if self.status() == reqwest::StatusCode::FOUND {
            return Err(crate::Error::Other(TokenExpired));
        }
        let body = self
            .error_for_status()
            .unexpected_err()?
            .text()
            .await
            .unexpected_err()?;
        let body = if decrypt {
            graduate_decrypt(&body)?
        } else {
            body
        };
        serde_json::from_str::<T>(&body).parse_err(&body)
    }
}
