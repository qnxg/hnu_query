use crate::{
    cas::{self},
    test::test_ok,
    xgxt::login::XgxtToken,
};

pub async fn get_xgxt_token() -> XgxtToken {
    let cas_token = test_ok(cas::test::get_cas_token().await, "get CAS token");

    test_ok(
        XgxtToken::acquire_by_cas_login(&cas_token).await,
        "acquire XGXT token",
    )
}
