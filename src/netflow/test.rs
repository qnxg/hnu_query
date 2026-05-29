use crate::{
    cas::{self},
    netflow::login::NetflowToken,
    test::test_ok,
};

pub async fn get_netflow_token() -> NetflowToken {
    let cas_token = test_ok(cas::test::get_cas_token().await, "get CAS token");

    test_ok(
        NetflowToken::acquire_by_cas_login(&cas_token).await,
        "acquire netflow token",
    )
}
