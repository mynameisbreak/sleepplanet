use crate::JsonResult;
use crate::config::get_config;
use crate::controller::sys_admin::*;
use crate::utils::error::AppError;
use crate::utils::jwt::{Claims, auth_hoop, generate_token};

use regex::Regex;
use salvo::http::cookie::Cookie;
use salvo::oapi::extract::JsonBody;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use time::{Duration, OffsetDateTime};
use tracing::{error, info, warn};
use validator::Validate;

/// 登录请求数据结构
#[derive(Deserialize, Debug, Default)]
pub struct SysLoginIndate {
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
}

/// 登录响应数据结构
#[derive(Serialize, Default, Debug)]
pub struct SysLoginOutDate {
    /// 用户ID
    pub id: i64,
    /// 用户名
    pub username: String,
    /// JWT令牌
    pub token: String,
    /// 令牌过期时间戳
    pub exp: i64,
}

/// 管理员登录处理器
/// 验证用户凭据并生成JWT令牌
#[handler]
pub async fn sys_login(
    idate: JsonBody<SysLoginIndate>,
    res: &mut Response,
) -> JsonResult<SysLoginOutDate> {
    let login_date = idate.into_inner();
    info!("管理员登录尝试:UserName={}", &login_date.username);

    // 查询用户信息
    let user = get_user_by_username(&login_date.username)
        .await?
        .ok_or_else(|| AppError::Public("用户名或密码错误".to_string()))?;
    let (user_id, username, password_hash) = user;

    // 验证密码
    match verify_password(&login_date.password, &password_hash) {
        Ok(true) => (),
        Ok(false) => {
            warn!("管理员登录失败:UserName={}", &login_date.username);
            return Err(AppError::Public("用户名或密码错误".to_string()));
        }
        Err(e) => {
            error!("密码验证过程中发生错误: {}", e);
            return Err(AppError::Public(format!("密码验证失败: {}", e)));
        }
    }

    // 获取用户角色
    let roles = get_user_roles(user_id).await?;
    info!(
        "管理员登录成功:UserName={},Role={:?}",
        &login_date.username, &roles
    );

    // 生成JWT令牌
    let jwt_config = &get_config().jwt;
    let token = generate_token(user_id, &username, &roles)?;

    // 构建响应数据
    let outdata = SysLoginOutDate {
        id: user_id,
        username,
        token: token.clone(),
        exp: (OffsetDateTime::now_utc() + Duration::seconds(jwt_config.expires_in as i64))
            .unix_timestamp(),
    };

    // 设置JWT Cookie
    let cookie = Cookie::build(("jwt_token", token))
        .path("/")
        .http_only(true)
        .build();
    res.add_cookie(cookie);

    Ok(Json(outdata))
}

/// TODO: 新增显示用户列表接口
/// TODO: 新增删除用户接口
/// TODO: 新增更新用户接口
/// TODO: 新增禁用用户接口
/// TODO: 新增启用用户接口
/// TODO: 新增重置密码接口
/// TODO: 新增修改密码接口
/// TODO: 新增修改角色接口
/// TODO: 新增新增角色接口
/// TODO: 新增删除角色接口
/// TODO: 新增显示角色列表接口

static RE_USERNAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_]+$").expect("用户名正则表达式编译失败"));

#[derive(Deserialize, Debug, Validate, Default)]
pub struct CreateUserIndate {
    /// 用户名
    #[validate(length(min = 3, max = 20, message = "用户名长度必须在3-20个字符之间"))]
    #[validate(regex(path = *RE_USERNAME, message = "用户名只能包含字母、数字和下划线"))]
    pub username: String,
    /// 密码
    #[validate(length(min = 8, max = 32, message = "密码长度必须在8-32个字符之间"))]
    pub password: String,
    /// 邮箱
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: Option<String>,
    /// 角色ID列表
    #[validate(length(min = 1, message = "至少需要分配一个角色"))]
    pub role_ids: Vec<i64>,
}

/// 创建用户响应数据结构
#[derive(Serialize, Default, Debug)]
pub struct CreateUserOutDate {
    /// 用户ID
    pub id: i64,
    /// 用户名
    pub username: String,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 角色列表
    pub roles: Vec<String>,
}

/// 创建用户处理器
/// 接收用户信息并创建新用户
#[handler]
pub async fn create_user(
    _req: &mut Request,
    depot: &Depot,
    idate: JsonBody<CreateUserIndate>,
) -> JsonResult<CreateUserOutDate> {
    // 权限检查：验证当前用户是否为超级管理员
    let data = match depot.jwt_auth_state() {
        JwtAuthState::Authorized => {
            let data = depot.jwt_auth_data::<Claims>().unwrap();
            data
        }
        JwtAuthState::Unauthorized => return Err(AppError::Public("未登录".to_string())),
        JwtAuthState::Forbidden => return Err(AppError::Public("没有权限".to_string())),
    };
    if !&data.claims.role.contains(&"super_admin".to_string()) {
        warn!("权限不足: 用户 {} 尝试创建管理员用户", &data.claims.username);
        // 由于 AppError 中不存在 Forbidden 变体，需要替换为合适的变体。这里假设替换为 Public 变体，具体根据实际情况调整。
        return Err(AppError::Public("没有创建用户的权限".to_string()));
    }

    let create_date = idate.into_inner();
    // 输入验证
    create_date.validate()?;
    info!("尝试创建新用户: 用户名={}", &create_date.username);

    // 验证用户名是否已存在
    let existing_user = get_user_by_username(&create_date.username).await?;
    if existing_user.is_some() {
        warn!("创建用户失败: 用户名 {} 已存在", &create_date.username);
        return Err(AppError::Public("用户名已存在".to_string()));
    }

    // 验证角色是否存在
    let valid_roles = validate_roles(&create_date.role_ids).await?;
    if valid_roles.is_empty() {
        return Err(AppError::Public("指定的角色不存在".to_string()));
    }

    // 哈希密码
    let password_hash = hash_password(&create_date.password).await?;

    // 创建新用户及角色分配
    let (user_id, created_at) = create_new_user(
        &create_date.username,
        &password_hash,
        &create_date.email,
        &create_date.role_ids,
    )
    .await?;

    // 获取用户角色名称
    let roles = get_user_role_names(user_id).await?;

    // 构建响应数据
    let outdata = CreateUserOutDate {
        id: user_id,
        username: create_date.username,
        created_at,
        roles,
    };

    info!(
        "用户创建成功: 用户ID={}, 用户名={}",
        user_id, &outdata.username
    );

    Ok(Json(outdata))
}

/// 密码哈希处理
async fn hash_password(password: &str) -> Result<String, AppError> {
    let cost = bcrypt::DEFAULT_COST;
    bcrypt::hash(password, cost).map_err(|e| AppError::Internal(format!("密码哈希失败: {}", e)))
}

/// 创建新用户并分配角色
async fn create_new_user(
    username: &str,
    password_hash: &str,
    email: &Option<String>,
    role_ids: &[i64],
) -> Result<(i64, chrono::DateTime<chrono::Utc>), AppError> {
    let pool = crate::db::get_pool();
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("数据库事务开始失败: {}", e)))?;

    // 插入用户
    let user = sqlx::query_as!(
        User,"INSERT INTO admin_user (username, password_hash, email, created_at, updated_at) VALUES (?, ?, ?, NOW(), NOW()) RETURNING id, created_at",
        username,
        password_hash,
        email,
    )
    .fetch_one(&mut tx)
    .await
    .map_err(|e| {
        error!("用户插入数据库失败: {}", e);
        AppError::Internal(format!("用户创建失败: {}", e))
    })?;

    // 分配角色
    for &role_id in role_ids {
        sqlx::query!(
            "INSERT INTO admin_user_role (user_id, role_id) VALUES (?, ?)",
            user.id,
            role_id
        )
        .execute(&mut tx)
        .await
        .map_err(|e| {
            error!(
                "用户角色分配失败: user_id={}, role_id={}, error={}",
                user.id, role_id, e
            );
            AppError::Internal(format!("角色分配失败: {}", e))
        })?;
    }

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("数据库事务提交失败: {}", e)))?;

    Ok((user.id, user.created_at))
}

/// 验证角色是否存在
async fn validate_roles(role_ids: &[i64]) -> Result<Vec<i64>, AppError> {
    let pool = crate::db::get_pool();
    let roles = sqlx::query!(
        "SELECT id FROM admin_roles WHERE id = ANY($1)",
        role_ids
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::Internal(format!("角色验证失败: {}", e)))?;

    Ok(roles.iter().map(|r| r.id as i64).collect())
}

/// 获取用户角色名称
async fn get_user_role_names(user_id: i64) -> Result<Vec<String>, AppError> {
    let pool = crate::db::get_pool();
    let roles = sqlx::query!(
        "SELECT r.role_name FROM admin_roles r JOIN admin_user_roles ur ON r.id = ur.role_id WHERE ur.user_id = $1",
        user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::Internal(format!("获取用户角色失败: {}", e)))?;

    Ok(roles.iter().map(|r| r.role_name.clone()).collect())
}
