use num_bigint::BigUint;

pub fn rsa_encrypt(password: &str, exponent: &str, modulus: &str) -> String {
    let password_bytes = password.as_bytes();
    let password_int = BigUint::from_bytes_be(password_bytes);
    let e_int = BigUint::parse_bytes(exponent.as_bytes(), 16).unwrap();
    let m_int = BigUint::parse_bytes(modulus.as_bytes(), 16).unwrap();

    let result_int = password_int.modpow(&e_int, &m_int);
    format!("{:0>128}", result_int.to_str_radix(16))
}
