use crate::{
    ca::login::CaToken,
    cas::{self},
};

pub async fn get_ca_token() -> CaToken {
    let mut cas_token = cas::test::get_cas_token().await.unwrap();
    CaToken::acquire_by_cas_login(&mut cas_token).await.unwrap()
}
