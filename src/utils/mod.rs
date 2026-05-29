pub mod request;

// 发送请求的全局请求池，设置请求上限为1000个
pub use request::CLIENT as client;
