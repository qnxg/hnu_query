mod raw;
mod utils;

use crate::gym::{
    error::TokenExpired,
    grade::{
        raw::{raw_grade_detail_data, raw_grade_summary_data},
        utils::{item_class_into_color, item_grade_into_color},
    },
    login::GymToken,
};
use serde::{Deserialize, Serialize};
use tokio::try_join;

/// 体测成绩
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Grade {
    /// 姓名
    pub name: String,
    /// 学号
    pub stu_id: String,
    /// 总成绩的文字描述，如 `不及格`
    pub grade: String,
    /// 总成绩的分数
    pub score: f64,
    /// 体测成绩描述，如 `暂无`
    pub report_desc: String,
    /// 体测成绩状态，如 `部分体测值异常`
    pub report_status: String,
    /// 体测成绩类型，如 `正常`
    pub report_type: String,
    /// 视力成绩
    pub eye: EyeGrade,
    /// 50m 成绩
    pub short_run: GradeItem,
    /// BMI 成绩
    pub bmi: GradeItem,
    /// 跳远成绩
    pub jump: GradeItem,
    /// 引体向上/仰卧起坐成绩
    ///
    /// 进一步区分是引体向上还是仰卧起坐，可以调用 `xgxt::personal_info` 来获取性别。
    pub pull_and_sit: GradeItem,
    /// 长跑成绩
    pub run: GradeItem,
    /// 坐位体前屈成绩
    pub sit_and_reach: GradeItem,
    /// 肺活量成绩
    pub vc: GradeItem,
}

/// 视力成绩
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EyeGrade {
    /// 右眼裸视力
    pub eyesight_right: String,
    /// 左眼裸视力
    pub eyesight_left: String,
    /// 右眼裸视力描述
    pub eyesight_right_detail: String,
    /// 左眼裸视力描述
    pub eyesight_left_detail: String,
    /// 右眼串镜视力
    pub eye_mirror_right: String,
    /// 右眼串镜视力描述
    pub eye_mirror_right_detail: String,
    /// 左眼串镜视力
    pub eye_mirror_left: String,
    /// 左眼串镜视力描述
    pub eye_mirror_left_detail: String,
    /// 右眼屈光不正视力
    pub eye_ametropia_right: String,
    /// 右眼屈光不正视力描述
    pub eye_ametropia_right_detail: String,
    /// 左眼屈光不正视力
    pub eye_ametropia_left: String,
    /// 左眼屈光不正视力描述
    pub eye_ametropia_left_detail: String,
}

/// 体测某个项目的成绩的具体描述
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GradeItem {
    /// 等级颜色
    pub color: GradeItemColor,
    /// 成绩的等级，
    /// 如 `优秀`、`良好`、`及格`、`不及格`、`缺项`、`肥胖`、`超重` 等
    pub rank: String,
    /// 取得的成绩数据
    ///
    /// 不同的项目，该字段的格式不同：
    /// - 50m: `{小数}秒`，如 `10.5秒`
    /// - BMI: `{小数}厘米/{小数}千克`，如 `178.5厘米/70.5千克`
    /// - 跳远: `{小数}厘米`，如 `192.0厘米`
    /// - 引体向上/仰卧起坐: `{整数}次`，如 `10次`
    /// - 1000m/800m: `{整数}'{整数}"`，如 `4'30"`
    /// - 坐位体前屈: `{小数}厘米`，如 `10.5厘米`
    /// - 肺活量: `{整数}毫升`，如 `3000毫升`
    pub grade: String,
    /// 该项目的得分
    ///
    /// 注意是整个项目的得分，一般来说会是一个 [0, 100] 的整数。
    /// 不是该项目占总成绩的分数。
    pub score: i32,
}

/// 体测某个项目的成绩的等级颜色
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Copy)]
pub enum GradeItemColor {
    /// 绿色
    Green,
    /// 红色
    Red,
}

/// 获取体测成绩
///
/// # Parameters
///
/// - `gym_token`: 体测系统的令牌，可以通过 [GymToken::acquire_by_cas_login] 或 [GymToken::acquire_by_direct_login] 获取
/// - `xn`: 学年，如 `2025`
///
/// # Returns
///
/// 返回体测成绩
///
/// # Errors
///
/// 如果提供的 `gym_token` 过期了，那么会返回 [TokenExpired] 错误，需要重新获取一个新的 [GymToken]
#[expect(clippy::too_many_lines, reason = "REFACTOR ME")]
pub async fn get_grade(gym_token: &GymToken, xn: u16) -> Result<Grade, crate::Error<TokenExpired>> {
    let (grade_summary, grade_detail) = try_join!(
        raw_grade_summary_data(gym_token, xn),
        raw_grade_detail_data(gym_token, xn),
    )?;
    let eye = EyeGrade {
        eyesight_right: grade_detail.eyesight_right,
        eyesight_left: grade_detail.eyesight_left,
        eyesight_right_detail: grade_detail.eyesight_right_detail,
        eyesight_left_detail: grade_detail.eyesight_left_detail,
        eye_mirror_right: grade_detail.eye_mirror_right,
        eye_mirror_right_detail: grade_detail.eye_mirror_right_detail,
        eye_mirror_left: grade_detail.eye_mirror_left,
        eye_mirror_left_detail: grade_detail.eye_mirror_left_detail,
        eye_ametropia_right: grade_detail.eye_ametropia_right,
        eye_ametropia_right_detail: grade_detail.eye_ametropia_right_detail,
        eye_ametropia_left: grade_detail.eye_ametropia_left,
        eye_ametropia_left_detail: grade_detail.eye_ametropia_left_detail,
    };
    // grade_summary 和 grade_detail 中
    // grade 是形如 `不及格` 的评级
    // class 形如是 `green` 的颜色信息（仅 grade_summary 中有）
    // score 在 grade_summary 中别是形如 `10.5秒` 的带单位数据
    //       在 grade_detail 中是该项目得分
    let short_run = GradeItem {
        color: grade_summary
            .short_run_class
            .map(|class| item_class_into_color(&class))
            .unwrap_or(item_grade_into_color(&grade_detail.short_run_grade)),
        rank: grade_detail.short_run_grade,
        grade: grade_summary
            .short_run_score
            .unwrap_or(grade_detail.short_run + "秒"),
        score: grade_detail.short_run_score,
    };
    let bmi = GradeItem {
        color: grade_summary
            .bmi_class
            .map(|class| item_class_into_color(&class))
            .unwrap_or(item_grade_into_color(&grade_detail.bmi_grade)),
        rank: grade_detail.bmi_grade,
        grade: grade_summary.bmi_score.unwrap_or(format!(
            "{}厘米/{}千克",
            grade_detail.height, grade_detail.weight
        )),
        score: grade_detail.bmi_score,
    };
    let jump = GradeItem {
        color: grade_summary
            .jump_class
            .map(|class| item_class_into_color(&class))
            .unwrap_or(item_grade_into_color(&grade_detail.jump_grade)),
        rank: grade_detail.jump_grade,
        grade: grade_summary
            .jump_score
            .unwrap_or(grade_detail.jump + "厘米"),
        score: grade_detail.jump_score,
    };
    let pull_and_sit = GradeItem {
        color: grade_summary
            .pull_and_sit_class
            .map(|class| item_class_into_color(&class))
            .unwrap_or(item_grade_into_color(&grade_detail.pull_and_sit_grade)),
        rank: grade_detail.pull_and_sit_grade,
        grade: grade_summary
            .pull_and_sit_score
            .unwrap_or(format!("{}次", grade_detail.pull_and_sit)),
        score: grade_detail.pull_and_sit_score + grade_detail.extra_score_pull_or_sit_up,
    };
    let run = GradeItem {
        color: grade_summary
            .run_class
            .map(|class| item_class_into_color(&class))
            .unwrap_or(item_grade_into_color(&grade_detail.run_grade)),
        rank: grade_detail.run_grade,
        grade: grade_summary.run_score.unwrap_or({
            let total_seconds: u32 = grade_detail.run.parse().unwrap_or(0);
            let minutes = total_seconds / 60;
            let seconds = total_seconds - minutes * 60;
            if seconds != 0 {
                format!("{}'{}\"", minutes, seconds)
            } else {
                format!("{}'", minutes)
            }
        }),
        score: grade_detail.run_score + grade_detail.extra_score_run,
    };
    let sit_and_reach = GradeItem {
        color: grade_summary
            .sit_and_reach_class
            .map(|class| item_class_into_color(&class))
            .unwrap_or(item_grade_into_color(&grade_detail.sit_and_reach_grade)),
        rank: grade_detail.sit_and_reach_grade,
        grade: grade_summary
            .sit_and_reach_score
            .unwrap_or(grade_detail.sit_and_reach + "厘米"),
        score: grade_detail.sit_and_reach_score,
    };
    let vc = GradeItem {
        color: grade_summary
            .vc_class
            .map(|class| item_class_into_color(&class))
            .unwrap_or(item_grade_into_color(&grade_detail.vc_grade)),
        rank: grade_detail.vc_grade,
        grade: grade_summary
            .vc_score
            .unwrap_or(format!("{}毫升", grade_detail.vc)),
        score: grade_detail.vc_score,
    };
    let res = Grade {
        name: grade_detail.student_name,
        stu_id: grade_detail.student_num,
        grade: grade_detail.total_grade,
        score: grade_detail.total_score,
        report_desc: grade_summary.report_desc.unwrap_or("暂无".to_string()),
        report_status: grade_summary.report_status.unwrap_or("暂无".to_string()),
        report_type: grade_summary.report_type.unwrap_or("暂无".to_string()),
        eye,
        short_run,
        bmi,
        jump,
        pull_and_sit,
        run,
        sit_and_reach,
        vc,
    };
    Ok(res)
}

#[cfg(test)]
mod test {
    use super::get_grade;
    use crate::{gym::test::get_gym_token, test::TEST_XN};

    #[tokio::test]
    #[ignore]
    pub async fn test_get_grade() {
        let gym_token = get_gym_token().await;
        let grade = get_grade(&gym_token, *TEST_XN).await.unwrap();
        println!("{:#?}", grade);
    }
}
