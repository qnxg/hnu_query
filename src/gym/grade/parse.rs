use super::{EyeGrade, Grade, GradeItem};
use crate::gym::{
    error::TokenExpired,
    grade::{
        GradeItemColor,
        raw::{RawGradeDetail, RawGradeSummary},
    },
};
use serde::{Deserialize, Deserializer};

/// If the value is None, return "0" instead.
pub fn none_to_zero<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer);
    if opt.is_err() {
        Ok(Some("0".to_string()))
    } else {
        Ok(opt?)
    }
}

fn item_grade_into_color(grade: &str) -> GradeItemColor {
    if ["不及格", "缺项", "肥胖", "超重"].contains(&grade) {
        GradeItemColor::Red
    } else {
        GradeItemColor::Green
    }
}

fn item_class_into_color(class: &str) -> GradeItemColor {
    if class == "red" {
        GradeItemColor::Red
    } else {
        GradeItemColor::Green
    }
}

#[expect(clippy::too_many_lines, reason = "REFACTOR ME")]
pub fn grade(
    grade_summary: RawGradeSummary,
    grade_detail: RawGradeDetail,
) -> Result<Grade, crate::Error<TokenExpired>> {
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
        color: grade_summary.short_run_class.map_or(
            item_grade_into_color(&grade_detail.short_run_grade),
            |class| item_class_into_color(&class),
        ),
        rank: grade_detail.short_run_grade,
        grade: grade_summary
            .short_run_score
            .unwrap_or(grade_detail.short_run + "秒"),
        score: grade_detail.short_run_score,
    };
    let bmi = GradeItem {
        color: grade_summary
            .bmi_class
            .map_or(item_grade_into_color(&grade_detail.bmi_grade), |class| {
                item_class_into_color(&class)
            }),
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
            .map_or(item_grade_into_color(&grade_detail.jump_grade), |class| {
                item_class_into_color(&class)
            }),
        rank: grade_detail.jump_grade,
        grade: grade_summary
            .jump_score
            .unwrap_or(grade_detail.jump + "厘米"),
        score: grade_detail.jump_score,
    };
    let pull_and_sit = GradeItem {
        color: grade_summary.pull_and_sit_class.map_or(
            item_grade_into_color(&grade_detail.pull_and_sit_grade),
            |class| item_class_into_color(&class),
        ),
        rank: grade_detail.pull_and_sit_grade,
        grade: grade_summary
            .pull_and_sit_score
            .unwrap_or(format!("{}次", grade_detail.pull_and_sit)),
        score: grade_detail.pull_and_sit_score + grade_detail.extra_score_pull_or_sit_up,
    };
    let run = GradeItem {
        color: grade_summary
            .run_class
            .map_or(item_grade_into_color(&grade_detail.run_grade), |class| {
                item_class_into_color(&class)
            }),
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
        color: grade_summary.sit_and_reach_class.map_or(
            item_grade_into_color(&grade_detail.sit_and_reach_grade),
            |class| item_class_into_color(&class),
        ),
        rank: grade_detail.sit_and_reach_grade,
        grade: grade_summary
            .sit_and_reach_score
            .unwrap_or(grade_detail.sit_and_reach + "厘米"),
        score: grade_detail.sit_and_reach_score,
    };
    let vc = GradeItem {
        color: grade_summary
            .vc_class
            .map_or(item_grade_into_color(&grade_detail.vc_grade), |class| {
                item_class_into_color(&class)
            }),
        rank: grade_detail.vc_grade,
        grade: grade_summary
            .vc_score
            .unwrap_or(format!("{}毫升", grade_detail.vc)),
        score: grade_detail.vc_score,
    };
    Ok(Grade {
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
    })
}

#[cfg(test)]
mod tests {
    use crate::test::TestResult;

    use super::*;

    #[test]
    fn test_grade() -> TestResult<()> {
        let raw_grade_summary: RawGradeSummary =
            serde_json::from_str(include_str!("test_data/getStudentScore.json"))?;
        let raw_grade_details: RawGradeDetail =
            serde_json::from_str(include_str!("test_data/getEyeDetails.json"))?;

        let grade = grade(raw_grade_summary, raw_grade_details)?;

        assert_eq!(grade.name, "林政和");
        assert_eq!(grade.stu_id, "202506050175");
        assert_eq!(grade.grade, "不及格");
        assert_eq!(grade.score, 56.2);
        assert_eq!(grade.report_desc, "暂无");
        assert_eq!(grade.report_status, "部分体测值异常");
        assert_eq!(grade.report_type, "正常");

        let eye = &grade.eye;
        assert_eq!(eye.eyesight_right, "--");
        assert_eq!(eye.eyesight_left, "--");
        assert_eq!(eye.eyesight_right_detail, "未测");
        assert_eq!(eye.eyesight_left_detail, "未测");
        assert_eq!(eye.eye_mirror_right, "9");
        assert_eq!(eye.eye_mirror_left, "9");
        assert_eq!(eye.eye_mirror_right_detail, "未测");
        assert_eq!(eye.eye_mirror_left_detail, "未测");
        assert_eq!(eye.eye_ametropia_right, "9");
        assert_eq!(eye.eye_ametropia_left, "9");
        assert_eq!(eye.eye_ametropia_right_detail, "未测");
        assert_eq!(eye.eye_ametropia_left_detail, "未测");

        assert_eq!(grade.short_run.color, GradeItemColor::Red);
        assert_eq!(grade.short_run.rank, "不及格");
        assert_eq!(grade.short_run.grade, "9.3秒");
        assert_eq!(grade.short_run.score, 50);

        assert_eq!(grade.bmi.color, GradeItemColor::Red);
        assert_eq!(grade.bmi.rank, "超重");
        assert_eq!(grade.bmi.grade, "178.6厘米/76.5千克");
        assert_eq!(grade.bmi.score, 80);

        assert_eq!(grade.jump.color, GradeItemColor::Red);
        assert_eq!(grade.jump.rank, "不及格");
        assert_eq!(grade.jump.grade, "205.0厘米");
        assert_eq!(grade.jump.score, 50);

        assert_eq!(grade.pull_and_sit.color, GradeItemColor::Red);
        assert_eq!(grade.pull_and_sit.rank, "不及格");
        assert_eq!(grade.pull_and_sit.grade, "0次");
        assert_eq!(grade.pull_and_sit.score, 0);

        assert_eq!(grade.run.color, GradeItemColor::Red);
        assert_eq!(grade.run.rank, "不及格");
        assert_eq!(grade.run.grade, "4'33''");
        assert_eq!(grade.run.score, 50);

        assert_eq!(grade.sit_and_reach.color, GradeItemColor::Green);
        assert_eq!(grade.sit_and_reach.rank, "及格");
        assert_eq!(grade.sit_and_reach.grade, "12.5厘米");
        assert_eq!(grade.sit_and_reach.score, 72);

        assert_eq!(grade.vc.color, GradeItemColor::Green);
        assert_eq!(grade.vc.rank, "良好");
        assert_eq!(grade.vc.grade, "4460毫升");
        assert_eq!(grade.vc.score, 80);

        Ok(())
    }
}
