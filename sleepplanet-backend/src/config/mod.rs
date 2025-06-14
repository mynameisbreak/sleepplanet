//! 应用配置模块，包含日志配置和数据库配置

use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
// use salvo::logging;
use serde::Deserialize;

mod bd_config;
mod log_config;

pub use bd_config::DbConfig;
pub use log_config::LogConfig;
use tokio::sync::OnceCell;

pub static SERVER_CONFIG: OnceCell<ServerConfig> = OnceCell::<ServerConfig>::const_new();

/// 初始化应用配置
/// 从配置文件和环境变量加载配置，并进行基本验证
pub fn init() {
    // 创建配置加载器，合并配置文件和环境变量
    // 优先使用 APP_CONFIG 环境变量指定的配置文件，默认使用 config.toml
    let raw_config = Figment::new()
        .merge(Toml::file(
            Env::var("APP_CONFIG").as_deref().unwrap_or("appconfig.toml"),
        ))
        .merge(Env::prefixed("APP_").global());

    // 解析配置到 ServerConfig 结构体
    let mut config = match raw_config.extract::<ServerConfig>() {
        Ok(config) => config,
        Err(_e) => {
            eprintln!("配置文件无效，请检查您的配置文件。");
            std::process::exit(1);
        }
    };

    // 从环境变量获取数据库URL（如果配置文件中未设置）
    if config.database.url.is_empty() {
        config.database.url = Env::var("DATABASE_URL").unwrap_or_default();
    }

    // 验证数据库URL是否已设置
    if config.database.url.is_empty() {
        eprintln!("DATABASE_URL 未设置，请在环境变量中设置 DATABASE_URL。");
        std::process::exit(1);
    }

    // 将配置存储到全局变量
    SERVER_CONFIG.set(config).expect("服务器配置初始化失败");
}


/// 获取全局服务器配置
///
/// 此函数会返回已初始化的 `ServerConfig` 实例。
/// 如果配置尚未初始化，程序会 panic。
pub  fn get_config() -> &'static ServerConfig {
    SERVER_CONFIG.get().expect("服务器配置尚未初始化，请先调用 init 函数")
}


#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct JwtConfig {
    /// JWT签名密钥
    pub secret: String,
    /// 令牌过期时间（秒）
    pub expires_in: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
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
