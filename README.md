<div align="center">

# HNU Query

湖南大学校内系统查询库

<p>
  <img alt="language" src="https://img.shields.io/badge/language-Rust-dea584" />
  <img alt="edition" src="https://img.shields.io/badge/edition-2024-1f425f" />
</p>
</div>

## 简介

本项目将湖南大学的校内系统（如个人门户，教务系统等）的接口进行包装，提供一套获取结构化数据的能力。

同时本项目也是 “湖南大学微生活” 小程序的数据抓取部分。如果你在使用湖南大学微生活小程序时遇到了和数据抓取/解析有关的错误，可以向本项目提交 Issue 或是 PR，所有的代码贡献都将会同步应用到湖南大学微生活小程序的线上版本。具体可以参考 CONTRIBUTING.md。

## 目前支持的功能

* 可信电子凭证平台
  * 获取本科生主修课程的可信电子凭证排名信息

* 体测系统
  * 获取体测预约信息
  * 获取体测成绩
* 教务系统
  * 获取课表
  * 获取无课表课程
  * 获取空教室
  * 获取考试安排
  * 获取课程成绩
  * 获取课程成绩详情分数
  * 获取排名

* 大物实验平台
  * 获取课程列表
  * 获取实验安排
  * 获取实验成绩

* 校园网流量系统
  * 获取校园网流量明细
  * 获取校园网流量账单
  * 获取校园网欠费金额
  * 获取当月校园网流量的使用情况
  * 获取校园网流量锁定状态

* 个人门户
  * 获取校园卡信息
  * 获取校园卡消费历史
  * 获取学校邮箱的未读邮件数

* 学工系统
  * 获取个人信息

* 其他
  * 获取宿舍电量


## 快速开始

你需要在湖南大学校园网内或是登录湖南大学 VPN 使用本项目。

本项目本质上为 [Rust](https://rust-lang.org/) [library crate](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html)，使用本项目需要你有一定的 Rust 基础。

TODO 安装

```rust
// 完整代码在 examples/demo.rs 中
// 学号
let stu_id = "";
// 个人门户密码
let password = "";
// 创建统一身份认证系统的令牌
let mut cas_token = CasToken::new(stu_id, password);
// 通过统一身份认证系统登录来获得教务系统的令牌
let hdjw_token = HdjwToken::acquire_by_cas_login(&mut cas_token).await.unwrap();
// 获取 2025 - 2026 学年秋季学期的课程成绩
let grade = hdjw::get_grade(&hdjw_token, 2025, 1).await;
println!("{:#?}", grade);
```

```text
Ok(
    [
        Grade {
            course_id: "TB001TY24Ⅲ",
            course_name: "体育Ⅲ",
            credit: 1.0,
            course_type1: Some(
                "必修",
            ),
            course_type2: "通识必修",
...
```

更详细的使用说明请参考项目的文档：

```bash
cargo doc --open
```

和本项目 `docs` 目录下的文档。

## 贡献

我们欢迎任何形式，或大或小的贡献。同时，你的贡献将有助于湖南大学微生活小程序变得更好。

我们在 CONTRIBUTING.md 中对贡献方法进行了说明，你可以参考。

> 我们欢迎对 Rust 和软件开发感兴趣的同学和我们一同交流：
>
> TODO 群二维码

## License

本项目基于 `AGPL-3.0` 协议。所有基于本项目的代码必须开源。