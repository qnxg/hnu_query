# 测试

## 静态数据解析测试

TODO

## 实际请求测试

由于发起实际的请求需要网络 IO，不适合在 CI 中运行，因此相关的测试均使用 `#[ignore]` 标记。使用 `cargo test` 测试时需要提供 `--ignored` 参数。

由于测试时共享了一个全局的 `reqwest` 的 client，所以测试时还需要传递 `--test-threads 1` 避免奇怪的报错。

请在校园网环境下或是登录湖大 VPN 后进行测试。测试时请注意你不要开启代理，并且确保 DNS 使用了校园网的 DNS（一般情况下这一点是满足的）。

发起实际的请求需要提供真实的用户信息。进行测试前，你需要在 `.env` 文件中填写相关的信息。

> **测试不通过？**
>
> 如果你发现你的账号无法通过某些测试，那么可能意味着当前的库没能正确处理一些特别的情况，请通过提交 Issue 向我们反馈。

### 一般平台

一般来说你需要填写如下的信息

- `TEST_STU_ID`：要用来测试的学号
- `TEST_PASSWORD`：要用来测试的学号的个人门户密码
- `TEST_XN`：测试时提供的学年
- `TEST_XQ`：测试时提供的学期
- `TEST_YEAR`：测试时提供的年份信息
- `TEST_MONTH`：测试时提供的月份信息
- `TEST_DAY`：测试时提供的日期信息

然后就可以跑如下的测试了

```bash
cargo test ca:: -- --ignored --test-threads 1 # 测试可信电子凭证平台相关
cargo test gym:: -- --ignored --test-threads 1 # 测试体测系统相关
cargo test netflow:: -- --ignored --test-threads 1 # 测试校园网流量相关
cargo test pt:: -- --ignored --test-threads 1 # 测试个人门户相关
cargo test xgxt:: -- --ignored --test-threads 1 # 测试学工系统相关
```

### 宿舍相关

如果要测试宿舍电量相关，则还需要在 `.env` 文件中配置如下信息：

* `TEST_DORMITORY_PARK`：宿舍园区信息
* `TEST_DORMITORY_BUILD`：宿舍楼栋信息
* `TEST_DORMITORY_ROOM`：宿舍的房间信息

上述信息需要满足 `xgxt::personal_info::Dormitory::from_parsed_value` 的参数要求。上述信息可以通过 `xgxt::personal_info` 获取。

然后就可以跑如下的测试了：

```bash
cargo test wxpay:: -- --ignored --test-threads 1
```

### 大物实验平台

如果要测试大物实验平台相关，首先你需要确保你提供的学号是有大物实验平台的权限的（只有当前学期有修读大学物理实验课程的学生才有该权限），然后你需要根据 `lab::test::captcha` 的注释配置好验证码解析器。


然后你需要在 `.env` 文件中配置如下信息：

* `TEST_LAB_PASSWORD`：大物实验平台的登陆密码
* `TEST_LAB_MAX_TRIED`：登录大物实验平台时最大重试次数，一般设置为 `5` 应该就足够了

然后就可以进行如下的测试：

```bash
cargo test lab::login -- --ignored --test-threads 1
cargo test lab::schedule -- --ignored --test-threads 1
cargo test lab::semester -- --ignored --test-threads 1
```

可以从 `lab::semester` 的输出信息中得到学期 id，将其配置到 `TEST_LAB_SEMESTER_ID` 中，就可以继续测试：

```bash
cargo test lab::course -- --ignored --test-threads 1
```

`lab::course` 的输出中可以得到课程 id，将其配置到 `TEST_LAB_COURSE_ID` 中，就可以继续测试：

```bash
cargo test lab::grade -- --ignored --test-threads 1
```

### 教务系统

如果要测试教务系统相关，则还需要在 `.env` 文件中配置如下信息：

* `TEST_HDJW_WEEK`：学期的第几周
* `TEST_HDJW_DAY_OF_WEEK`：一周的第几天，可参考项目文档的字段约定部分

* `TEST_HDJW_BUILDING_ID`：用来测试空教室查询时提供的楼栋 id，取值见 `get_empty_classroom` 的参数说明
* `TEST_HDJW_TIME`：用来测试空教室查询时提供的节次信息，多个节次用 `,` 分割，取值见 `hdjw::get_empty_classroom` 的参数说明

* `TEST_HDJW_JX0404ID`：用来测试成绩详情信息时提供的课程的 `jx0404id`，该取值需要先通过测试 `hdjw::get_grade` 的输出来获得

然后就可以跑如下的测试了：

```bash
cargo test hdjw:: -- --ignored --test-threads 1
```

