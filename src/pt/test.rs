use crate::{
    cas::{self},
    pt::login::PtToken,
    test::test_ok,
};

pub async fn get_pt_token() -> PtToken {
    let cas_token = test_ok(cas::test::get_cas_token().await, "get CAS token");

    test_ok(
        PtToken::acquire_by_cas_login(&cas_token).await,
        "acquire PT token",
    )
}
