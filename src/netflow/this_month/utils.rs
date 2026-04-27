/// 给流量数据加上单位，原始的数据是没有单位的
///
/// 流量字符串可能返回"小于0.01GB"，此时不重复添加单位
pub fn try_add_gb_suffix(str: &mut String) {
    if !str.ends_with("GB") {
        *str += "GB";
    }
}
