use crate::{
    error::{MapNetworkErr, MapParseErr, MapUnexpectedErr, parse_err},
    hdjw::{error::TokenExpired, login::HdjwToken, raw::HdjwResponseExtractor},
    utils::client,
};
use serde::Deserialize;

// 课表这里的课程信息接口是分页的，我这里设置了一页 50 条，应该没有人一学期超过 50 门课吧（）
// 教务系统有点诡异，这个 pageSize 最好不要设置太大。我们发现，如果设置 200 这个特殊数字就会返回 html 的格式，其他数字都会返回 json 格式。具体原因不明，但是不建议太大，适量就好
// 该 URL 缺少学期的参数，需要后续再用 format 拼接
const CLASS_TABLE_URL: &str = "http://hdjw.hnu.edu.cn/jsxsd/xskb/xskb_list.do?viweType=1&needData=1&pageNum=1&pageSize=50&viweType=1&demoStr=&needData=1&baseUrl=%2Fjsxsd&sfykb=2&xsflMapListJsonStr=%E8%AE%B2%E8%AF%BE%E5%AD%A6%E6%97%B6%2C%E6%8C%87%E5%AF%BC%E5%AD%A6%E6%97%B6%2C%E5%AE%9E%E9%AA%8C%E5%AD%A6%E6%97%B6%2C%E5%85%B6%E4%BB%96%2C&zc=&kbjcmsid=1";

// 无课表课程
// 该 URL 缺少学期的参数，需要后续再用 format 拼接
const CLASS_TABLE_EXTRA: &str = "http://hdjw.hnu.edu.cn/jsxsd/xskb/xskb_list.do?viweType=2&needData=1&pageNum=1&pageSize=20&viweType=2&demoStr=&needData=1&baseUrl=%2Fjsxsd&sfykb=1&xsflMapListJsonStr=%E8%AE%B2%E8%AF%BE%E5%AD%A6%E6%97%B6%2C%E6%8C%87%E5%AF%BC%E5%AD%A6%E6%97%B6%2C%E5%AE%9E%E9%AA%8C%E5%AD%A6%E6%97%B6%2C%E5%85%B6%E4%BB%96%2C&zc=&kbjcmsid=1";

/// 教务 `教学运行 > 我的课表 > 有课表课程` 返回数据单项
/// 还有其他一些具体学时信息的字段，懒得搞了
#[derive(Deserialize, Debug)]
#[expect(unused)]
pub struct CourseInfo {
    /// 课程代码
    pub kch: String,
    /// 课程名称
    pub kc_mc: String,
    /// 教师名称
    pub jg0101mc: Option<String>,
    /// 教师工号（暂时不用）
    pub jsgh: Option<String>,
    pub kt_mc: String, // 上课班级
    /// 课堂容量（暂时不用）
    pub pkrs: u16,
    /// 上课人数
    pub xkrs: u16,
    /// 课程性质（通识必修/专业核心等）
    pub kcxz: String,
    /// 课程类别（必修/选修等）
    pub kclb: String,
    /// 通知单编号（暂时不用）
    pub jx0404id: String,
    /// 分组名称，这里当作课程的备注信息
    pub fzmc: Option<String>,
    /// 上课时间
    pub sktime: String,
    /// 上课地点
    pub skddmc: String,
    /// 上课校区
    pub skxqmc: String,
    /// 开课院系（暂时不用）
    pub kkyx: String,
    /// 周学时（暂时不用）
    pub zhouxs: String,
    /// 学分
    pub xf: f32,
    /// 总学时（暂时不用）
    pub zxs: u16,
    /// 考核方式（暂时不用）
    pub khfs: String,
}

/// 教务 `教学运行 > 我的课表 > 无课表课程` 返回数据单项
#[derive(Deserialize, Debug)]
pub struct ExtraCourseInfo {
    /// 课程代码
    pub kch: String,
    /// 课程名称
    pub kc_mc: String,
    /// 教师名称
    pub jg0101mc: String,
    /// 分组名称
    pub fzmc: Option<String>,
    /// 课程性质（通识必修/专业核心等）
    pub kcxz: String,
    /// 上课班级
    pub kt_mc: String,
    /// 上课人数
    pub xkrs: u16,
    /// 上课校区
    pub skxqmc: String,
    /// 学分
    pub xf: f32,
}

/// 获取课表信息
pub async fn raw_class_table_data(
    hdjw_token: &HdjwToken,
    xn: u16,
    xq: u8,
) -> Result<Vec<CourseInfo>, crate::Error<TokenExpired>> {
    let headers = hdjw_token.headers().clone();
    let res = client
        .get(format!(
            "{}&xnxq01id={}-{}-{}",
            CLASS_TABLE_URL,
            xn,
            xn + 1,
            xq
        ))
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .extract_data()
        .await?;
    match res.get("count").and_then(|c| c.as_u64()) {
        None => Err(parse_err(&res.to_string())),
        Some(0) => Ok(vec![]), // 有可能 count 是 0 但是不带 data 字段
        Some(_) => {
            // 取 data 字段返回
            let res: Vec<CourseInfo> =
                serde_json::from_value(res["data"].clone()).parse_err(&res.to_string())?;

            Ok(res)
        }
    }
}

/// 获取无课表课程
pub async fn raw_class_table_extra_data(
    hdjw_token: &HdjwToken,
    xn: u16,
    xq: u8,
) -> Result<Vec<ExtraCourseInfo>, crate::Error<TokenExpired>> {
    let headers = hdjw_token.headers().clone();
    let res = client
        .get(format!(
            "{}&xnxq01id={}-{}-{}",
            CLASS_TABLE_EXTRA,
            xn,
            xn + 1,
            xq
        ))
        .headers(headers)
        .send()
        .await
        .network_err()?
        .error_for_status()
        .unexpected_err()?
        .extract_data()
        .await?;
    match res.get("count").and_then(|c| c.as_u64()) {
        None => Err(parse_err(&res.to_string())),
        Some(0) => Ok(vec![]), // 有可能 count 是 0 但是不带 data 字段
        Some(_) => {
            // 取 data 字段返回
            let res: Vec<ExtraCourseInfo> =
                serde_json::from_value(res["data"].clone()).parse_err(&res.to_string())?;
            Ok(res)
        }
    }
}
