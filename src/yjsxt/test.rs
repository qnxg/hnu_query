use crate::{
    cas::{self},
    yjsxt::login::YjsxtToken,
};

pub async fn get_yjsxt_token() -> YjsxtToken {
    let cas_token = cas::test::get_cas_token().await.unwrap();
    YjsxtToken::acquire_by_cas_login(&cas_token)
        .await
        .unwrap()
}
