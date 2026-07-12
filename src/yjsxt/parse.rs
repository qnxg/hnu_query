use crate::{error::MapParseErr, yjsxt::error::TokenExpired};
use aes::cipher::{BlockDecryptMut, KeyInit, block_padding::Pkcs7};
use base64::engine::{Engine, general_purpose::STANDARD as base64};

const GRADUATE_KEY: &str = "southsoft12345!#";

type Aes128EcbDec = ecb::Decryptor<aes::Aes128>;

// 研究生系统的响应存在加密，需要解密
pub fn decrypt_response(data: &str) -> Result<String, crate::Error<TokenExpired>> {
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
