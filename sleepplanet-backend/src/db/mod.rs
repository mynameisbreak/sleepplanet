use crate::config::DbConfig;
use sqlx::postgres::PgPool;
use tokio::sync::OnceCell;

pub static SQLX_POOL: OnceCell<PgPool> = OnceCell::const_new();

pub async fn init_db(db: &DbConfig) {
    // 建立数据库连接池
    let pool = match PgPool::connect(&db.url).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("数据库url错误: {}", &db.url);
            tracing::error!("数据库连接失败: {}", e);
            std::process::exit(1);
        }
    };

    // 将连接池设置到全局变量
    // 设置全局数据库连接池
    if let Err(e) = SQLX_POOL.set(pool) {
        tracing::error!("设置全局数据库连接池失败: {}", e);
        std::process::exit(1);
    }
}

// 获取数据库连接池
#[inline]
pub fn get_pool() -> &'static PgPool {
    SQLX_POOL.get().unwrap()
}