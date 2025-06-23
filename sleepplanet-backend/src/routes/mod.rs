use anyhow::Error;
use salvo::handler;
use salvo::logging::*;
use salvo::prelude::*;
use sqlx::{FromRow, PgPool};

use crate::db;
pub mod sys_admin;

// 定义一个简单的路由处理函数

// 创建基础路由

// 为了满足 salvo::Handler 特征约束，使用 HandlerFn 将函数包装成处理器
#[handler]
pub async fn hello_world(res: &mut Response) {
    res.render(Text::Plain("Hello, Salvo!"));
}

#[handler]
pub async fn table_count(depot: &Depot, res: &mut Response) {
    // 从Depot获取数据库连接池
    let pool = db::get_pool();

    // 查询public模式下的表数量
    // 由于查询结果是一个整数，这里显式指定查询结果的类型为 i64
    let count = sqlx::query_scalar::<_, i64>(
        r#"
            SELECT COUNT(*) 
            FROM information_schema.tables 
            WHERE table_schema = 'public'
            "#,
    )
    .fetch_one(pool)
    .await
    .unwrap();
    res.render(Json(count));
}
pub fn root() -> Router {
    // 构建并返回Router
    Router::new()
        .get(hello_world)
        .push(Router::with_path("sys").push(Router::with_path("login").post(sys_admin::sys_login)))
        
}


