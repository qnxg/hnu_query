use crate::{
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    hdjw::{error::TokenExpired, login::HdjwToken, raw::HdjwResponseExtractor},
    utils::client,
};
use serde::Deserialize;

// 课程成绩查询接口
// 该 URL 缺少学期的参数，需要后续再用 format 拼接
const GRADE_URL: &str = "http://hdjw.hnu.edu.cn/jsxsd/kscj/cjcx_list?pageNum=1&pageSize=50&kcxz=&kcsx=&kcmc=&xsfs=all&sfxsbcxq=1";

// 该 URL 缺少 jx0404id 的参数，需要后续再用 format 拼接
const GRADE_DETAIL_URL: &str = "http://hdjw.hnu.edu.cn/jsxsd/kscj/pscj_list.do?zcj=";

/// 教务 `考试成绩 > 课程成绩` 返回数据单项
#[derive(Deserialize, Debug)]
pub struct GradeInfoRes {
    // 未知字段
    // pub cj0708id: String,
    // 学年学期信息（暂时不用）
    // pub xnxqid: String,
    /// 课程代码
    pub kch: String,
    /// 课程名称
    pub kc_mc: String,
    // 开课学院（暂时不用）
    // pub ksdw: String,
    //  似乎和 xnxqid 重复
    // pub xqmc: String,
    /// 学分
    pub xf: f32,
    // 总学时（暂时不用）
    // pub zxs: u32,
    // 考试方式（暂时不用）
    // pub ksfs: String,
    /// 课程属性（必修/选修等）
    pub kcsx: Option<String>,
    // 似乎又和 xnxqid 重复
    // pub xqstr: String,
    /// 总成绩
    pub zcj: u8,
    // 总成绩字符串形式（暂时不用）
    // pub zcjstr: String,
    // 未知字段
    // pub kz: u8,
    ///  课程性质（通识必修/专业核心等）
    pub kcxzmc: String,
    // 未知字段
    // pub xs0101id: String,
    /// 用于课程成绩详情查询，部分成绩没有该字段
    pub jx0404id: Option<String>,
    /// 绩点
    pub jd: f32,
    // 考试性质（暂时不用）
    // pub ksxz: String,
    /// 主修还是辅修
    pub falb: String,
    /// 成绩标识（缓考/重修等，注意这个标识是挂在为 0 分的那个成绩 item 上）
    pub cjbs: Option<String>,
}

pub async fn raw_grade_data(
    hdjw_token: &HdjwToken,
    xn: u16,
    xq: u8,
) -> Result<Vec<GradeInfoRes>, crate::Error<TokenExpired>> {
    let headers = hdjw_token.headers().clone();
    let mut raw_res = client
        .get(format!("{}&kksj={}-{}-{}", GRADE_URL, xn, xn + 1, xq))
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .extract_data()
        .await?;
    let raw_res_str = raw_res.to_string();
    let res: Vec<GradeInfoRes> =
        serde_json::from_value(raw_res["data"].take()).parse_err(&raw_res_str)?;
    Ok(res)
}

/// 返回的原始数据是 html 格式
pub async fn raw_grade_detail_data(
    hdjw_token: &HdjwToken,
    jx0404id: &str,
) -> Result<String, crate::Error<TokenExpired>> {
    let headers = hdjw_token.headers().clone();
    let res = client
        .get(format!("{}&jx0404id={}", GRADE_DETAIL_URL, jx0404id))
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .extract_data()
        .await?;
    Ok(res.as_str().ok_or(parse_err(&res.to_string()))?.to_string())
}
