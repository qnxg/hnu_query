use crate::{
    ca::login::CaToken,
    cas::{self},
    test::test_ok,
};

pub async fn get_ca_token() -> CaToken {
    let cas_token = test_ok(cas::test::get_cas_token().await, "get CAS token");

    test_ok(
        CaToken::acquire_by_cas_login(&cas_token).await,
        "acquire CA token",
    )
}
