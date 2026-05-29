use hnu_query::{
    cas::login::CasToken,
    hdjw::{self, login::HdjwToken},
};

#[tokio::main]
async fn main() {
    // 学号
    let stu_id = "";
    // 个人门户密码
    let password = "";
    // 创建统一身份认证系统的令牌
    let Ok(cas_token) = CasToken::acquire_by_login(stu_id, password).await else {
        eprintln!("CAS login failed");
        return;
    };
    // 通过统一身份认证系统登录来获得教务系统的令牌
    let Ok(hdjw_token) = HdjwToken::acquire_by_cas_login(&cas_token).await else {
        eprintln!("HDJW login failed");
        return;
    };
    // 获取 2025 - 2026 学年秋季学期的课程成绩
    let grade = hdjw::get_grade(&hdjw_token, 2025, 1).await;
    println!("{:#?}", grade);
}
