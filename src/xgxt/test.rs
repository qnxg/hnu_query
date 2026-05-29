use crate::{
    cas::{self},
    xgxt::login::XgxtToken,
};

pub async fn get_xgxt_token() -> XgxtToken {
    let cas_token = cas::test::get_cas_token().await.unwrap();
    XgxtToken::acquire_by_cas_login(&cas_token)
        .await
        .unwrap()
}
