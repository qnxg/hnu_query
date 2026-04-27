use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
/// 宿舍信息
///
/// [`crate::xgxt::get_person_info`] 会从学工系统拿到原始的宿舍信息，该原始宿舍信息包含园区和楼栋，需要进一步解析
pub struct Dormitory {
    park: Option<String>,
    build: Option<String>,
    /// 房间号
    room: String,
    /// 原始宿舍信息
    raw_dormitory: String,
}

impl Dormitory {
    /// 从已经解析好的园区和楼栋信息构造 Dormitory
    ///
    /// # Parameters
    ///
    /// - `park`: 园区
    /// - `build`: 楼栋
    /// - `room`: 房间号
    ///
    /// # Preconditions
    ///
    /// `park` 和 `build` 的值必须是 [`Dormitory::park`] 和 [`Dormitory::build`] 能够返回的值，否则可能会出现意料之外的后果
    pub fn from_parsed_value(park: &str, build: &str, room: &str) -> Self {
        Self {
            park: Some(park.to_string()),
            build: Some(build.to_string()),
            room: room.to_string(),
            raw_dormitory: format!("{}{}", park, build),
        }
    }
    /// 园区
    ///
    /// 解析成功后有且仅有如下取值：
    ///
    /// * `南校区`
    /// * `财院校区`
    /// * `天马园区`
    /// * `德智园区`
    /// * `德智留学生公寓`
    /// * `望麓桥学生公寓`
    /// * `牛头山学生公寓`
    ///
    /// 如果解析失败，则返回 None
    pub fn park(&self) -> Option<&str> {
        self.park.as_deref()
    }
    /// 楼栋
    ///
    /// 解析成功后有且仅有如下取值：
    ///
    /// * 当园区为`南校区`时：
    ///     * `7舍` 到 `8舍`
    ///     * `10舍` 到 `15舍`
    ///     * `17舍` 到 `19舍`
    ///     * `南楼`
    ///     * `培训小楼`
    /// * 当园区为`财院校区`时：
    ///     * `1` 到 `3`
    ///     * `5` 到 `7`
    ///     * `12`
    ///     * `A` 到 `B`
    /// * 当园区为`天马园区`时：
    ///     * `一区1栋` 到 `一区4栋`
    ///     * `二区1栋` 到 `二区7栋`
    ///     * `三区9栋` 到 `三区13栋`
    ///     * `三区16栋` 到 `三区20栋`
    ///     * `四区1栋` 到 `四区4栋`
    /// * 当园区为`德智园区`时：
    ///     * `2栋`
    ///     * `5栋` 到 `11栋`
    ///     * `13栋`
    ///     * `15栋` 到 `16栋`
    /// * 当园区为`德智留学生公寓`时：TODO 尚不明确
    /// * 当园区为`望麓桥学生公寓`时：
    ///     * `1栋` 到 `4栋`
    /// * 当园区为`牛头山学生公寓`时：
    ///     * `2栋` 到 `7栋`
    ///
    /// 如果解析失败，则返回 None
    pub fn build(&self) -> Option<&str> {
        self.build.as_deref()
    }
    /// 房间号
    pub fn room(&self) -> &str {
        &self.room
    }
    /// 原始宿舍信息
    pub fn raw_dormitory(&self) -> &str {
        &self.raw_dormitory
    }
    /// 是否解析成功，即园区和楼栋都解析成功
    pub fn successfully_parsed(&self) -> bool {
        self.park.is_some() && self.build.is_some()
    }
}

/// 从学工系统的宿舍信息中解析出 Dormitory
pub fn parse_dormitory(dormitory: String, room: String) -> Dormitory {
    let mut park = None;
    let mut build = None;
    if dormitory.contains("德智") {
        park = Some("德智园区");
        let re = Regex::new(r"\d+栋").expect("构建正则表达式失败");
        build = re.find_iter(&dormitory).map(|mat| mat.as_str()).next();
    }
    if dormitory.contains("天马") {
        park = Some("天马园区");
        let re = Regex::new(r"[一二三四]区\d+栋").expect("构建正则表达式失败");
        build = re.find_iter(&dormitory).map(|mat| mat.as_str()).next();
    }
    if dormitory.contains("望麓桥") {
        park = Some("望麓桥学生公寓");
        let re = Regex::new(r"\d+栋").expect("构建正则表达式失败");
        build = re.find_iter(&dormitory).map(|mat| mat.as_str()).next();
    }
    if dormitory.contains("牛头山") {
        let re = Regex::new(r"\d+栋").expect("构建正则表达式失败");
        build = re.find_iter(&dormitory).map(|mat| mat.as_str()).next();
    }
    if dormitory.contains("财院校区") {
        park = Some("财院校区");
        let re = Regex::new(r"[1-9AB]").expect("构建正则表达式失败");
        build = re.find_iter(&dormitory).map(|mat| mat.as_str()).next();
        // TODO 研楼目前还没有样本，不知道怎么搞
    }
    if dormitory.contains("南校区") {
        park = Some("南校区");
        let re = Regex::new(r"[1-9]+舍").expect("构建正则表达式失败");
        build = re.find_iter(&dormitory).map(|mat| mat.as_str()).next();
    }
    Dormitory {
        park: park.map(|s| s.to_string()),
        build: build.map(|s| s.to_string()),
        room,
        raw_dormitory: dormitory,
    }
}
