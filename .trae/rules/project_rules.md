使用中文交流
以下是为项目优化的简洁中文开发规范：
Salvo框架概述
Salvo是基于Rust的Web框架，聚焦简洁性、高效性和易用性。核心概念包括路由（Router）、处理器（Handler）、中间件（Middleware）、请求（Request）、响应（Response）和数据仓库（Depot）。
核心概念：
	1. 路由（Router）：
	• 通过Router::new()创建路由实例。
	• 使用path()或with_path()定义路径规则。
	• 支持get()、post()、patch()、delete()等HTTP方法绑定。
	• 支持路径参数（如{id}、<id:num>格式）。
	• 可添加filters::path()、filters::get()等过滤器。
	• 使用hoop()方法添加中间件。
	2. 处理器（Handler）：
	• 使用#[handler]宏简化处理器定义。
	• 可选参数包含Request（请求对象）、Depot（数据仓库）、FlowCtrl（流程控制器）。
	• 返回类型需实现Writer特征（如&str、String、Result<T, E>）。
	3. 中间件（Middleware）：
	• 需实现Handler特征。
	• 通过hoop()方法将中间件添加到路由或服务中。
	• 通过FlowCtrl控制执行流程（如ctrl.skip_rest()跳过后续处理）。
	4. 请求（Request）：
	• 使用req.param::<T>("param_name")获取路径参数。
	• 使用req.query::<T>("query_name")获取查询参数。
	• 通过req.form::<T>("name").await或req.parse_json::<T>().await解析表单或JSON数据。
	• 使用req.extract()将数据提取到结构体中。
	5. 响应（Response）：
	• 使用res.render()渲染响应内容。
	• 支持Text::Plain()（纯文本）、Text::Html()（HTML）、Json()（JSON）等响应类型。
	• 通过res.status_code()或StatusError设置状态码。
	• 使用Redirect::found()实现重定向。
	6. 数据仓库（Depot）：
	• 通过depot.insert()和depot.obtain::<T>()等方法在中间件与处理器间存储临时数据。
	7. 提取器（Extractors）：
	• 使用#[salvo(extract(...))]注解将请求数据映射到结构体。

核心功能：
	• 静态文件服务：支持StaticDir（目录映射）或StaticEmbed（嵌入编译）方式。
	• OpenAPI支持：通过#[endpoint]宏自动生成接口文档。

路由设计：
	• 支持扁平式或树状路由结构。

中间件机制：
	• 中间件本质是Handler，可添加到路由（Router）、服务（Service）或异常捕获器（Catcher）。
	• FlowCtrl支持跳过后续处理器或中间件。

错误处理：
	• 处理器返回Result<T, E>类型，其中T和E需实现Writer特征。
	• 自定义错误通过Writer特征处理，默认使用anyhow::Error类型。

部署规范：
	• Salvo应用可编译为单一可执行文件，方便部署。

Project Structure:

sleepplanet-backend/
├── src/
│   ├── main.rs       # 入口文件，启动服务
│   ├── router/
│   │   ├── audio.rs  # 音频相关路由（含推荐/管理接口）
│   │   ├── admin.rs  # 管理员相关路由（含用户/审计接口）
│   │   └── middleware/
│   │       ├── auth.rs    # JWT认证中间件（校验请求权限）
│   │       └── rate_limit.rs # 限流中间件（防止恶意请求）
│   ├── service/
│   │   ├── audio.rs  # 音频服务（元数据操作/推荐算法）
│   │   ├── category.rs # 分类服务（分类树构建/校验）
│   │   ├── tag.rs    # 标签服务（标签关联/去重）
│   │   └── user.rs   # 用户服务（权限校验/审计日志）
│   ├── model/
│   │   ├── audio.rs  # 音频元数据结构体（含标题/时长/格式）
│   │   ├── category.rs # 分类结构体（含父级ID/层级）
│   │   └── tag.rs    # 标签结构体（含关联音频计数）
│   ├── error/
│   │   ├── mod.rs    # 自定义错误类型定义（如ServiceError、DbError）
│   │   └── handler.rs# HTTP错误响应格式化
│   ├── common/
│   │   ├── utils.rs  # 通用工具函数（时间格式化/文件指纹生成）
│   │   └── config.rs # 配置解析（读取.env文件，含COS密钥）
│   └── adapter/
│       ├── db.rs     # 数据库操作封装（sqlx执行/连接池管理）
│       └── cos.rs    # COS存储适配器（文件上传/下载签名）
├── migrations/
└── assets/
    ├── js/
    └── css/

JSON Response Format:

#[derive(Serialize)]
pub struct JsonResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: T,
}

示例响应（音频推荐接口）:
```json
{
  "code": 200,
  "data": [
    {"id": "aud_789", "title": "雨夜助眠"},
    {"id": "aud_012", "title": "溪流白噪音"}
  ]
}
```

前端指南：
	1.	Tailwind CSS：
	•	使用 flex、grid、space-x、space-y、bg-{颜色}、text-{颜色}、rounded-{尺寸}、shadow-{尺寸}。
	2.	Alpine.js：
	•	使用 x-data、x-model、@click、x-show、x-if。
	3.	片段架构：
	•	通过 X-Fragment-Header 和 x-html 实现部分页面更新。

错误处理：
	•	AppError 处理各种错误类型：公共错误、内部错误、HTTP 状态错误、Sqlx 错误、验证错误。
	•	使用 tracing 记录错误并返回适当的 HTTP 状态码。

数据库操作：
	•	使用 SQLx 进行异步数据库操作（例如 query!、query_as!）。
	•	使用 LIMIT 和 OFFSET 进行分页。

密码处理：
	•	使用 bcrypt/Argon2 进行密码哈希处理。

输入验证：
	•	使用 validator 验证和清理输入。

SQLx 指南：

1. 数据库连接设置：
   • PostgreSQL：
     - URL 格式：postgres://用户:密码@主机:端口/数据库名
     - 启用功能：sqlx/postgres
     - 类型：支持 Timestamp、Uuid、Json/Jsonb。

2. 查询宏：
   ```rust
   // 编译时进行类型检查
   sqlx::query!("SELECT * FROM users WHERE id = $1", id)
   
   // 命名参数
   sqlx::query!("SELECT * FROM users WHERE id = $1 AND active = $2", id, active)
   
   // 插入并返回数据
   sqlx::query!("INSERT INTO users (name) VALUES ($1) RETURNING id", name)
   ```

3. 模型集成：
   ```rust
   #[derive(sqlx::FromRow)]
   struct User {
       id: i64,
       username: String,
       created_at: chrono::DateTime<chrono::Utc>
   }
   
   // 与 query_as! 一起使用
   sqlx::query_as!(User, "SELECT * FROM users")
   ```

4. 常见操作：
   • 查询：
     ```rust
     let user = sqlx::query_as!(User, 
         "SELECT * FROM users WHERE id = $1", id
     ).fetch_one(&pool).await?;
     ```
   • 插入：
     ```rust
     sqlx::query!(
         "INSERT INTO users (username) VALUES ($1)",
         username
     ).execute(&pool).await?;
     ```
   • 更新：
     ```rust
     sqlx::query!(
         "UPDATE users SET username = $1 WHERE id = $2",
         new_username, id
     ).execute(&pool).await?;
     ```
   • 删除：
     ```rust
     sqlx::query!(
         "DELETE FROM users WHERE id = $1", id
     ).execute(&pool).await?;
     ```

5. 迁移：
   • CLI 设置：cargo install sqlx-cli
   • 创建：sqlx migrate add create_users
   • 运行：sqlx migrate run
   • 回滚：sqlx migrate revert

6. 最佳实践：
   • 使用连接池（sqlx::Pool）
   • 使用 query! 宏启用编译时检查
   • 使用事务进行原子操作
   • 为模型实现 FromRow
   • 在 CI 中使用 sqlx prepare 启用离线模式

7. 高级功能：
   • Postgres：
     - 数组：$1::text[]
     - JSON：jsonb、json 类型
     - 支持 LISTEN/NOTIFY

8. 错误处理：
   • 使用 anyhow 或 thiserror
   • 处理特定于数据库的错误
   • 实现自定义错误类型
   • 使用 Result<T, sqlx::Error>
