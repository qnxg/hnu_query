# 代码规范

## 1. 可序列化与 Trait

- 一般情况下，所有公开出去的结构体应满足下面条件之一，以确保大多数结构体都可以序列化成文本然后缓存（详见 [`cache.md`](./cache.md)）：
  - 实现了 `Serialize` 和 `Deserialize`
  - 提供了类似 `from_xxx` / `from_xxx_unchecked` 的方法，可以根据该方法构造结构体；该方法需要的参数应能通过结构体上公开的方法获取
- 所有公开出去的结构体 / 枚举都应该实现 `Debug`。一般情况下所有公开出去的结构体 / 枚举都应该实现 `Clone`
- 业务结果类型通常同时 derive：`Serialize, Deserialize, Debug, Clone`（顺序随便无所谓）
- Token 类型一般**不**实现 `Serialize` 和 `Deserialize`，而是提供 `from_*_unchecked`

## 2. 错误处理

- 统一使用 `crate::Error<E>`
- `src/error.rs` 中提供了一些错误转换的辅助函数：
  - `.network_err()`：网络请求错误。可以作用在 `reqwest` 库中可能返回 `reqwest::Error` 的地方，将其转为 `NetworkError`
  - `.unexpected_err()`：难以归类或理论上不应失败的错误。可以作用在任意的错误类型上，将其转为 `Unexpected`
  - `.parse_err(data)?` / `.parse_err_with_reason(data, reason)?`：解析失败的错误。可以作用在任意的错误类型上，将其转为 `ParseError`
  - 有时解析失败可能并非抛出了什么错误导致，而是某些条件不满足，对于这种情况我们可以手动调用函数 `parse_err` / `parse_err_with_reason` 来直接构造解析错误
- 域内特有的错误放在该系统的 `error.rs`（如 `TokenExpired`）。某个接口的特定错误放在功能子模块内（如 `LoginIssue`、`AccountIssue`）
- 没有特定错误时，`E` 使用 `std::convert::Infallible`
- 不应该使用 `unwrap`，已经配置 `clippy` 规则进行禁止。会 panic 的地方（比如 `assert` 和 `expect`），仅用于全局单例初始化（如正则表达式构建）或是文档明确声明会 panic 的前置条件。如果根据代码逻辑，某个代码写了看似会 panic 的地方实际不会执行，那么就无需写到函数的文档注释。

## 3. 公开性

- 所有文件遵循只有提供给外部使用的东西才设置为 `pub`
- 对于 `parse.rs`，如果解析过程需要先转为一个中间结构体，那么这个中间的结构体不应该 `pub`，里面的字段也不能被 `pub`，除非解析过程只能将数据解析一部分，需要 `mod.rs` 再次请求其他的数据来进一步解析（比如 `crate::gym::get_appointment`）
- 对于各个系统的 `mod.rs`，需要把各个功能子模块给 `pub` 出去，同时 `pub use` 这些子模块提供的函数，不 `pub use` 子模块的结构体
- 测试函数（非辅助类函数，而是带 `#[test]` 或 `#[tokio::test]` 的函数）不应该 `pub`

## 4. 命名

- 对于 `fetch.rs` 和 `parse.rs`，`pub` 出去的函数名称不应该带有和请求/解析有相同或类似含义的动词，比如 `pub async fn get_xxx` 就是不太合适的，应该写为 `pub async fn xxx`。对于不 `pub` 出去的函数则没有此要求。在功能子模块的 `mod.rs` 中进行调用的时候，应使用 `fetch::xxx` 和 `parse::xxx` 的写法
- 对于 `parse.rs`，如果解析过程需要先转为一个中间结构体，那么这个中间的结构体的名字建议带 `Raw` 前缀，不要和 `mod.rs` 中的结构体名字一致
- `crate::Error` 应该写全，比如一些 `Result` 类型应写为 `Result<T, crate::Error<E>>` 而不是 `Result<T, Error<E>>`

## 5. 注释

- `parse.rs` 中所有 `pub` 出去的函数的参数应该在文档注释中注明被解析的参数的来源，例如：`` `json_str` 为 [`super::fetch::xxx`] 返回的数据 ``
- 对于所有的公开到 crate 外部的函数/结构体/枚举，应当具有清晰的文档注释。对于一些名称本身即可足够表示无歧义含义的地方可以不加文档注释
- 文档注释应该是用于给调用者说明的。对于一些无需调用者了解的细节、步骤、临时 workaround 等，应该使用 `//` 注释，且注释独占一行
- 文档注释使用 `# Arguments`、`# Returns`、`# Panics`、`# Errors`、`# Performence` 等来划分文档注释的部分。注意对参数的说明部分使用 `# Arguments` 而不是 `# Parameters`
- 文档注释中的列表使用 `-` 而不是 `*`

## 6. 测试

- 所有涉及到产生实际请求（网络 IO）的测试，应该标注为 `#[ignore]`。测试用到的参数，应该可以通过 `.env` 配置（见 [`test.md`](./test.md)）
- 测试函数应该返回 `TestResult`
- 测试数据应放到 `test_data` 文件夹下
- 一些可以共用的测试辅助函数和测试用的环境变量的加载放在 `test.rs` 中，测试用的环境变量使用 `crate::test::test_env_parse` 来加载，并定义为一个使用 `LazyLock` 包裹的 `static` 变量。如果测试用的环境变量只会在一个地方使用，那么不必设为 `static` 变量，如果不涉及到将字符串转换为其他类型，那么也不必使用 `test_env_parse`，直接使用 `env!(...)` 即可。
- 测试函数一般放在被测试函数所在的同一个代码文件的末尾定义的 `mod tests` 中。`mod tests`
- 和测试相关的模块/函数都应设置 `#[cfg(test)]`
- 如果新增了一些接口，需要在对应的功能子模块的 `mod.rs` 中添加测试函数，该测试函数应该可以根据环境变量提供的参数发送实际请求，同时输出你新增加的接口的运行结果。同时还需要在 `test_data` 文件夹下放置学校对应系统响应的原始数据，并在 `parse.rs` 中添加测试函数，解析你添加的测试数据，并对解析结果进行 `assert`。同时不要忘记更新 [`test.md`](./test.md)
- 如果修复了一些接口，如果之前的接口是因为数据解析而导致的需要修复，则你需要在 `test_data` 中放置原来不能正常解析的数据，并在 `parse.rs` 中添加测试函数

## 7. 可观测性

- 项目中的一些关键函数（主要是一些公开的 API）需要添加形如如下的代码

  ```rust
  #[cfg_attr(
      feature = "tracing",
      tracing::instrument(skip(password), fields(subsystem = "cas"), err)
  )]
  ```

  其中的 `subsystem` 需要为函数所在的学校系统的名称。同时需要使用 `skip(...)` 来跳过一些涉及到登录凭证（如密码，Token）的敏感参数。

- 在一些逻辑比较复杂的地方需要开启 span 并记录一些日志

## 8. 其他

- 模块的头部，各个部分的顺序应为：`mod` -> `use` -> `pub use`，三个部分用空白行分割，每个部分内部不要有空行分割。`use` 部分内部的顺序应该为 `use super::...` -> `use crate::...` -> `use ...`
- fetch.rs
  - `fetch.rs` 的主要职责是请求数据，里面的函数应尽可能返回原始数据，尽可能少做解析。常见的返回结果是 `String` 或是 `Bytes`
  - 在模块头部定义要请求的 URL 常量，如： `const XXX_URL: &str`
  - 使用 `crate::utils::client` 发起请求，HNU Query 在普通的 `reqwest::Client` 的基础上增加了一些配置（如不开启 HTTPS 证书校验，关闭自动重定向等），具体见 `src/utils/request.rs`

    一个经典的发送请求的代码：

    ```rust
    client
        .get(URL)
        .headers(token.headers().clone())
        .send()
        .await
        // send 出错是 NetworkError
        .network_err()?
        .error_for_status()
        // 响应的状态码不对是 UnexpectedError
        .unexpected_err()?
        .text()
        .await
        // 没有拿到响应的文本数据也是 Unexpected Error
        .unexpected_err()
    ```

- parse.rs
  - `parse.rs` 的主要职责是解析数据，里面的函数应只做数据解析而不发送请求，因此函数一般是非 `async` 的

---

更多：

- [`fields.md`](./fields.md)：字段约定，你的代码也应该遵循这些字段约定
