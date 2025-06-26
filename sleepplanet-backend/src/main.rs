use salvo::{prelude::*, server::ServerHandle};
use tokio::signal;
use tracing::{debug, info};

use crate::utils::error::AppError;
mod config;
mod controller;
mod db;
mod routes;
mod utils;

pub struct EmptyObject {}
pub type AppResult<T> = Result<T, AppError>;
pub type JsonResult<T> = Result<Json<T>, AppError>;
pub type EmptyResult = Result<Json<EmptyObject>, AppError>;
/// 🚀 应用程序入口点
/// 负责初始化配置、数据库连接、日志系统，并启动Web服务器
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化配置系统
    crate::config::init();
    let config = crate::config::get_config();

    // 初始化日志系统
    let _guard = config.log.guard();
    info!("📊 日志级别设置为: {}", &config.log.filter_level);

    // 初始化数据库连接池
    db::init_db(&config.database).await;
    info!("✅ 数据库连接池初始化成功");

    // 创建路由服务
    let service = Service::new(routes::root());

    // 绑定服务器地址
    let acceptor = TcpListener::new(&config.server.addr).bind().await;

    // 创建并启动服务器
    let server = Server::new(acceptor);
    tokio::spawn(shutdown_signal(server.handle()));

    info!("🌐 服务器已启动，监听地址: {}", config.server.addr);
    server.serve(service).await;

    Ok(())
}

/// 🛑 优雅关闭信号处理
async fn shutdown_signal(handle: ServerHandle) {
    // 监听Ctrl+C信号
    let ctrl_c = async {
        signal::ctrl_c().await.expect("无法安装Ctrl+C信号处理器");
    };

    // 监听Unix终止信号
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("无法安装终止信号处理器")
            .recv()
            .await;
    };

    // 非Unix系统不监听终止信号
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // 等待任一信号
    tokio::select! {
        _ = ctrl_c => info!("⏹️ 接收到Ctrl+C信号，开始优雅关闭..."),
        _ = terminate => info!("⏹️ 接收到终止信号，开始优雅关闭..."),
    }

    // 60秒内优雅关闭服务器
    handle.stop_graceful(std::time::Duration::from_secs(60));
}
