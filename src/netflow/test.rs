use crate::{
    cas::{self},
    netflow::login::NetflowToken,
};

pub async fn get_netflow_token() -> NetflowToken {
    let cas_token = cas::test::get_cas_token().await.unwrap();
    NetflowToken::acquire_by_cas_login(&cas_token)
        .await
        .unwrap()
}
