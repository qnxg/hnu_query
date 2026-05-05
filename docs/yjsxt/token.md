# 研究生系统令牌

研究生系统的令牌（`YjsxtToken`）与教务系统的令牌不同，额外包含了一个 `id` 字段。

## 获取 id

登录研究生系统时，CAS 重定向的目标路径中包含一段标识符：

```
http://yjsxt.hnu.edu.cn/gmis/{id}/student/...
```

这里的 `{id}` 会从 Location 响应头中提取，后续的所有 API 请求都需要将其拼接在路径中。

## 缓存

与教务系统令牌一样，`YjsxtToken` 也支持缓存：

```rust
// 缓存时保存 headers 和 id
let headers = yjsxt_token.headers().clone();
let id = yjsxt_token.id().to_string();

// 恢复时
let yjsxt_token = YjsxtToken::from_unchecked(headers, id);
```

确保缓存的 headers 是有效的，否则会导致未定义行为。
