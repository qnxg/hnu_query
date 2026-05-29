use crate::{
    cas::{self},
    pt::login::PtToken,
};

pub async fn get_pt_token() -> PtToken {
    let cas_token = cas::test::get_cas_token().await.unwrap();
    PtToken::acquire_by_cas_login(&cas_token).await.unwrap()
}
