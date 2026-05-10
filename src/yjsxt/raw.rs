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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graduate_decrypt() {
        let encoded = "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";
        let res = graduate_decrypt(encoded).unwrap();
        println!("{}", res);
    }
}
