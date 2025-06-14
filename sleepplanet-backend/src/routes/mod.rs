use salvo::handler;
use salvo::logging::*;
use salvo::prelude::*;

// 定义一个简单的路由处理函数

// 创建基础路由
pub fn root() -> Router {
    // 为了满足 salvo::Handler 特征约束，使用 HandlerFn 将函数包装成处理器
    #[handler]
    async fn hello_world(res: &mut Response) {
        res.render(Text::Plain("Hello, Salvo!"));
    }

    Router::new().hoop(Logger::new()).get(hello_world)
}
