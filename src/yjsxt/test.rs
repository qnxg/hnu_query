use crate::{
    cas::{self},
    test::test_ok,
    yjsxt::login::YjsxtToken,
};

pub async fn get_yjsxt_token() -> YjsxtToken {
    let cas_token = test_ok(cas::test::get_cas_token().await, "get CAS token");

    test_ok(
        YjsxtToken::acquire_by_cas_login(&cas_token).await,
        "acquire YJSXT token",
    )
}
