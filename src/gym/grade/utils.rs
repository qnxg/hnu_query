use crate::gym::grade::GradeItemColor;
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

pub fn item_grade_into_color(grade: &str) -> GradeItemColor {
    if ["不及格", "缺项", "肥胖", "超重"].contains(&grade) {
        GradeItemColor::Red
    } else {
        GradeItemColor::Green
    }
}

pub fn item_class_into_color(class: &str) -> GradeItemColor {
    if class == "red" {
        GradeItemColor::Red
    } else {
        GradeItemColor::Green
    }
}
