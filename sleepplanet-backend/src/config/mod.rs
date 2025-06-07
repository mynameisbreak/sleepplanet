use serde::Deserialize;

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
    pub addr:String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DbConfig {
    /// 数据库连接URL
    pub url: String,
    /// 连接池大小
    pub pool_size: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogConfig {
    /// 日志级别（如"debug", "info"）
    pub level: String,
    /// 日志文件路径
    pub path: String,
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
