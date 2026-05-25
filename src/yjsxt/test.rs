use crate::{
    cas::{self},
    yjsxt::login::YjsxtToken,
};

pub async fn get_yjsxt_token() -> YjsxtToken {
    let mut cas_token = cas::test::get_cas_token().await.unwrap();
    YjsxtToken::acquire_by_cas_login(&mut cas_token)
        .await
        .unwrap()
}
