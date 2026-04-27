//! 使用此库通过 use crate::utils::client; 即可，在mod.rs中已pub use导出

use reqwest::{
    Client,
    header::{GetAll, HeaderValue},
    redirect::Policy,
};
use std::{sync::LazyLock, time::Duration};

pub static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        // 需要忽略无效证书，因为图书馆的https证书是寄的，而采用http请求的话cas下发的ticket不被图书系统接受
        .danger_accept_invalid_certs(true)
        .connection_verbose(false)
        // 设置超时是重要的，避免超时中间件触发后任务仍在进行
        .timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(2))
        .pool_idle_timeout(Duration::from_secs(20))
        .pool_max_idle_per_host(2000)    // 部署到生产环境一定要适当调整，Linux系统默认TCP上限是1024，本程序大致访问6个host左右，设置一个合理值不要超过上限，如系统没有网络调优情况下可设置为100
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36")  // 必须加才会通过cas的验证
        // .user_agent("reqwest/0.12.8")
        .no_proxy() // 禁止代理，防止调试时候系统代理的影响
        .redirect(Policy::none()) // 禁止自动重定向方便操作，目前有几个接口依赖于禁止重定向，因此不能直接允许重定向
        // .http1_title_case_headers()
        .build()
        .expect("构建client失败")
});

/// 项目的cookie_parser，旨在只保留key=value的字符串形式
#[inline]
pub fn cookie_parser(cookie: GetAll<HeaderValue>) -> Vec<String> {
    cookie
        .iter()
        .filter_map(cookie_parser_inner)
        .collect::<Vec<String>>()
}

#[inline]
fn cookie_parser_inner(cookie: &HeaderValue) -> Option<String> {
    let cookie = cookie
        .to_str()
        .expect("异常cookie")
        .split(';')
        .collect::<Vec<&str>>();
    let pair: Vec<&str> = cookie[0].split('=').collect();
    if pair[1].is_empty() {
        return None;
    }
    Some(format!("{}={}", pair[0], pair[1]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_request() {
        let res = CLIENT.get("https://www.baidu.com").send().await.unwrap();
        assert!(res.status().is_success());
    }
}
