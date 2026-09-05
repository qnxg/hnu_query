use std::{fmt, str::FromStr};

/// 时间段
/// 应为`14:00~16:00`的形式
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct TimeRange {
    start_time: String,
    end_time: String,
}

impl TimeRange {
    pub fn new(start_time: &str, end_time: &str) -> TimeRange {
        TimeRange {
            start_time: start_time.to_string(),
            end_time: end_time.to_string(),
        }
    }
}

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}~{}", self.start_time, self.end_time)
    }
}

/// 从形如`14:00~16:00`的字符串构建时间段
impl FromStr for TimeRange {
    type Err = std::fmt::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.splitn(2, ['~', '-']).collect::<Vec<_>>();
        let (start_time, end_time) = match parts[..] {
            [s, e] => (s, e),
            _ => return Err(std::fmt::Error),
        };

        Ok(TimeRange::new(start_time, end_time))
    }
}

impl PartialEq for TimeRange {
    fn eq(&self, other: &Self) -> bool {
        self.start_time == other.start_time && self.end_time == other.end_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(
            "14:00~16:00".parse::<TimeRange>(),
            Ok(TimeRange::new("14:00", "16:00"))
        );
        assert_eq!(
            "14:00-16:00".parse::<TimeRange>(),
            Ok(TimeRange::new("14:00", "16:00"))
        );
    }

    #[test]
    fn test_from_str_multiple_separators() {
        // `splitn(2, ['~', '-'])` 只按第一个分隔符切分一次
        assert_eq!(
            "14:00~16:00-18:00".parse::<TimeRange>(),
            Ok(TimeRange::new("14:00", "16:00-18:00"))
        );
        assert_eq!(
            "14:00-16:00~18:00".parse::<TimeRange>(),
            Ok(TimeRange::new("14:00", "16:00~18:00"))
        );
    }

    #[test]
    fn test_from_str_separator_at_edges() {
        // 分隔符在开头/结尾时，空字符串会作为 start/end 的一部分
        assert_eq!(
            "~16:00".parse::<TimeRange>(),
            Ok(TimeRange::new("", "16:00"))
        );
        assert_eq!(
            "14:00~".parse::<TimeRange>(),
            Ok(TimeRange::new("14:00", ""))
        );
    }

    #[test]
    fn test_from_str_invalid() {
        assert!("14:00".parse::<TimeRange>().is_err());
        assert!("".parse::<TimeRange>().is_err());
        // 不含分隔符的输入同样无法解析
        assert!("14:00,16:00".parse::<TimeRange>().is_err());
        assert!("1416".parse::<TimeRange>().is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(TimeRange::new("14:00", "16:00").to_string(), "14:00~16:00");
    }

    #[test]
    fn test_parse_display_round_trip() {
        let range = "14:00~16:00".parse::<TimeRange>().expect("valid range");
        assert_eq!(range.to_string(), "14:00~16:00");
        assert_eq!(range.to_string().parse::<TimeRange>(), Ok(range));
    }

    #[test]
    fn test_partial_eq() {
        assert_eq!(
            TimeRange::new("14:00", "16:00"),
            TimeRange::new("14:00", "16:00")
        );
        assert_ne!(
            TimeRange::new("14:00", "16:00"),
            TimeRange::new("14:00", "17:00")
        );
        assert_ne!(
            TimeRange::new("14:00", "16:00"),
            TimeRange::new("13:00", "16:00")
        );
    }

    #[test]
    fn test_serde_round_trip() {
        let range = TimeRange::new("14:00", "16:00");
        let json = serde_json::to_string(&range).expect("can not transform a TimeRange");
        assert_eq!(json, r#"{"start_time":"14:00","end_time":"16:00"}"#);
        assert_eq!(
            serde_json::from_str::<TimeRange>(&json).expect("should be a TimeRange"),
            range
        );
    }

    #[test]
    fn test_deserialize_error() {
        // 字段缺失 / 字段类型错误 / 非法 JSON 都应反序列化失败
        assert!(serde_json::from_str::<TimeRange>("{}").is_err());
        assert!(serde_json::from_str::<TimeRange>(r#"{"start_time":"14:00"}"#).is_err());
        assert!(
            serde_json::from_str::<TimeRange>(r#"{"start_time":"14:00","end_time":42}"#).is_err()
        );
        assert!(serde_json::from_str::<TimeRange>("not json").is_err());
    }
}
