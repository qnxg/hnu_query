use aes::cipher::KeyInit;
use aes::cipher::{BlockEncryptMut, block_padding::Pkcs7};
use base64::engine::{Engine as _, general_purpose::STANDARD as base64};

type Aes128EcbEnc = ecb::Encryptor<aes::Aes128>;

const LAB_KEY: &str = "1234567891234567";

pub fn lab_encrypt(e: &str) -> String {
    let cipher = Aes128EcbEnc::new_from_slice(LAB_KEY.as_bytes()).unwrap();
    let plaintext = e.as_bytes();
    let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(plaintext);
    let p1 = base64.encode(&ciphertext);
    base64.encode(p1.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lab_encrypt() {
        let data = "202402050201";
        let encrypted = lab_encrypt(data);
        assert_eq!(encrypted, "M1laOXBibVZLeS9Qd0FvUGlnNnlmZz09");
    }
}
