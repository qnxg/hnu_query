pub(crate) mod request;

// 发送请求的全局请求池，设置请求上限为1000个
pub(crate) use request::CLIENT as client;
