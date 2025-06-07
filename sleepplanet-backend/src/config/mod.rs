use salvo::logging;
use serde::Deserialize;
mod log_config;

pub use log_config::LogConfig;
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub server: ListenConfig,
    pub database: DbConfig,
    pub log: LogConfig,
    pub jwt: JwtConfig,
    pub ttl: TtlConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListenConfig {
    /// 服务监听端口
    #[serde(default = "default_listen_addr")]
    pub addr: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DbConfig {
    /// 数据库连接URL
    pub url: String,
    /// 连接池大小
    pub pool_size: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    /// JWT签名密钥
    pub secret: String,
    /// 令牌过期时间（秒）
    pub expires_in: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TtlConfig {
    /// 会话存活时间（秒）
    pub session: u64,
    /// 缓存存活时间（秒）
    pub cache: u64,
}

#[allow(dead_code)]
pub fn default_false() -> bool {
    false
}
#[allow(dead_code)]
pub fn default_true() -> bool {
    true
}

fn default_listen_addr() -> String {
    "127.0.0.1:8008".into()
}
