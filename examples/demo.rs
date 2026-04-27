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
    let mut cas_token = CasToken::new(stu_id, password);
    // 通过统一身份认证系统登录来获得教务系统的令牌
    let hdjw_token = HdjwToken::acquire_by_cas_login(&mut cas_token)
        .await
        .unwrap();
    // 获取 2025 - 2026 学年秋季学期的课程成绩
    let grade = hdjw::get_grade(&hdjw_token, 2025, 1).await;
    println!("{:#?}", grade);
}
