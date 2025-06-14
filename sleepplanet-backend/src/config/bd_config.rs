use serde::{Deserialize, Serialize};

use super::default_false;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DbConfig {
    /// 主数据库设置。通常可写，但在某些配置中为只读。
    /// 可选的从数据库，始终为只读。
    #[serde(alias = "database_url")]
    pub url: String,
    #[serde(default = "default_db_pool_size")]
    pub pool_size: u32,
    pub min_idle: Option<u32>,

    /// 等待未确认TCP数据包的秒数，超时后视为连接中断。
    /// 此值决定应用与数据库之间完全丢包时的不可用时长：
    /// 设置过高会导致不必要的长时间中断（在数据库异常逻辑触发前），
    /// 设置过低可能导致健康连接被误判为中断。
    #[serde(default = "default_tcp_timeout")]
    pub tcp_timeout: u64,
    /// 从连接池获取可用连接的等待时间，超时后返回错误。
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,
    /// 等待查询响应的时间，超时后取消查询并返回错误。
    #[serde(default = "default_statement_timeout")]
    pub statement_timeout: u64,
    /// 用于异步操作（如连接创建）的线程数。
    #[serde(default = "default_helper_threads")]
    pub helper_threads: usize,
    /// 是否强制所有数据库连接使用TLS加密。
    #[serde(default = "default_false")]
    pub enforce_tls: bool,
}

fn default_helper_threads() -> usize {
    10
}
fn default_db_pool_size() -> u32 {
    10
}
fn default_tcp_timeout() -> u64 {
    10000
}
fn default_connection_timeout() -> u64 {
    30000
}
fn default_statement_timeout() -> u64 {
    30000
}