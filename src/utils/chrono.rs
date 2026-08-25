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
            [_] => return Err(std::fmt::Error),
            _ => unreachable!(),
        };

        Ok(TimeRange::new(start_time, end_time))
    }
}

impl PartialEq for TimeRange {
    fn eq(&self, other: &Self) -> bool {
        self.start_time == other.start_time && self.end_time == other.end_time
    }
}
