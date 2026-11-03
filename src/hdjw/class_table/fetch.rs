use crate::{
    error::{CheckStatusCodeErr, MapNetworkErr, MapUnexpectedErr},
    hdjw::{error::TokenExpired, login::HdjwToken},
    utils::client,
};

// 课表这里的课程信息接口是分页的，我这里设置了一页 50 条，应该没有人一学期超过 50 门课吧（）
// 教务系统有点诡异，这个 pageSize 最好不要设置太大。我们发现，如果设置 200 这个特殊数字就会返回 html 的格式，其他数字都会返回 json 格式。具体原因不明，但是不建议太大，适量就好
// 该 URL 缺少学期的参数，需要后续再用 format 拼接
const CLASS_TABLE_URL: &str = "http://hdjw.hnu.edu.cn/jsxsd/xskb/xskb_list.do?viweType=1&needData=1&pageNum=1&pageSize=50&viweType=1&demoStr=&needData=1&baseUrl=%2Fjsxsd&sfykb=2&xsflMapListJsonStr=%E8%AE%B2%E8%AF%BE%E5%AD%A6%E6%97%B6%2C%E6%8C%87%E5%AF%BC%E5%AD%A6%E6%97%B6%2C%E5%AE%9E%E9%AA%8C%E5%AD%A6%E6%97%B6%2C%E5%85%B6%E4%BB%96%2C&zc=&kbjcmsid=1";

/// 获取课表信息
pub async fn class_table(
    hdjw_token: &HdjwToken,
    xn: u16,
    xq: u8,
) -> Result<String, crate::Error<TokenExpired>> {
    client
        .get(format!(
            "{}&xnxq01id={}-{}-{}",
            CLASS_TABLE_URL,
            xn,
            xn + 1,
            xq
        ))
        .headers(hdjw_token.headers().clone())
        .send()
        .await
        .network_err()?
        .status_code_err()
        .await?
        .text()
        .await
        .unexpected_err()
}
