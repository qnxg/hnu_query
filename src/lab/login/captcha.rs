/// 大物实验平台的验证码解析器
pub trait CaptchaResolver {
    /// 解析验证码
    ///
    /// # Arguments
    ///
    /// - `data`: 验证码图片的二进制数据，应该是 `jpg` 格式的图片
    ///
    /// # Returns
    ///
    /// 解析后的验证码
    ///
    /// # Errors
    ///
    /// 如果解析失败，则返回错误信息
    fn resolve(
        &self,
        data: &[u8],
    ) -> impl Future<Output = Result<String, Box<dyn std::error::Error + Send + Sync>>>;
}
