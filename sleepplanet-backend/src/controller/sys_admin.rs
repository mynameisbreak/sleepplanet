use anyhow::Result;
use argon2::{Argon2, PasswordHash, PasswordVerifier, password_hash::SaltString};
use rand::rngs::OsRng;
use tracing::warn;

use crate::db::get_pool;
use crate::utils::error::AppError;
use sqlx::Row;

/// 根据用户名查询用户信息
///
/// 从数据库中查询指定用户名的活跃用户，返回用户ID、用户名和密码哈希
///
/// # 参数
/// * `username` - 要查询的用户名
///
/// # 返回值
/// * `Ok(Some((id, username, password_hash)))` - 找到用户时返回用户信息元组
/// * `Ok(None)` - 未找到用户时返回None
/// * `Err(_)` - 数据库查询失败时返回错误
pub async fn get_user_by_username(username: &str) -> Result<Option<(i64, String, String)>> {
    let pool = get_pool();
    let user = sqlx::query!(
        "SELECT id, username, password_hash FROM admin_user WHERE username = $1 AND is_active = true",
        username
    )
    .fetch_optional(pool)
    .await?;

    // 数据库id字段为i32类型，转换为i64以满足上层接口需求
    Ok(user.map(|u| (u.id as i64, u.username, u.password_hash)))
}

/// 获取用户的角色列表
///
/// 通过用户ID查询该用户拥有的所有角色名称
///
/// # 参数
/// * `user_id` - 用户ID
///
/// # 返回值
/// * `Ok(Vec<String>)` - 包含角色名称的向量
/// * `Err(_)` - 数据库查询失败时返回错误
pub async fn get_user_roles(user_id: i64) -> Result<Vec<String>> {
    let pool = get_pool();
    let roles = sqlx::query!(
        "SELECT r.name FROM roles r JOIN user_roles ur ON r.id = ur.role_id WHERE ur.user_id = $1",
        user_id as i32,
    )
    .fetch_all(pool)
    .await?;
    Ok(roles.into_iter().map(|r| r.name).collect())
}

/// 验证当前用户是否为super_admin
///
/// # 参数
/// * `current_user_id` - 当前登录用户ID
///
/// # 返回值
/// * `Ok(true)` - 是super_admin
/// * `Err(AppError)` - 不是super_admin或查询失败
pub async fn is_super_admin(current_user_id: i64) -> Result<bool, AppError> {
    let roles = get_user_roles(current_user_id)
        .await
        .map_err(|e| AppError::Internal(format!("查询用户角色失败: {}", e)))?;
    Ok(roles.contains(&"super_admin".to_string()))
}

/// 获取角色ID by角色名称
///
/// # 参数
/// * `role_name` - 角色名称
///
/// # 返回值
/// * `Ok(i32)` - 角色ID
/// * `Err(AppError)` - 角色不存在或查询失败
pub async fn get_role_id_by_name(role_name: &str) -> Result<i32, AppError> {
    let pool = get_pool();
    let role = sqlx::query!("SELECT id FROM roles WHERE name = $1", role_name)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(format!("查询角色失败: {}", e)))?
        .ok_or_else(|| AppError::Public(format!("角色不存在: {}", role_name)))?;
    Ok(role.id)
}

/// 创建管理员用户并分配角色
///
/// 只有super_admin可以调用此函数，通过事务确保用户创建和角色分配的原子性
///
/// # 参数
/// * `current_user_id` - 当前登录用户ID（必须是super_admin）
/// * `username` - 新用户的用户名
/// * `password` - 新用户的原始密码
/// * `email` - 新用户的邮箱
/// * `phone_number` - 新用户的手机号（可选）
/// * `role_names` - 要分配的角色名称列表
///
/// # 返回值
/// * `Ok(i64)` - 成功创建的用户ID
/// * `Err(AppError)` - 创建失败（权限不足/数据重复/数据库错误等）
pub async fn create_admin_user(
    current_user_id: i64,
    username: &str,
    password: &str,
    email: &str,
    phone_number: Option<&str>,
    role_names: &[&str],
) -> Result<i64, AppError> {
    // 1. 权限校验：仅super_admin可创建管理员
    if !is_super_admin(current_user_id).await? {
        warn!("创建管理员用户失败: 非super_admin用户尝试创建管理员用户");
        return Err(AppError::Public(
            "需要super_admin权限才能创建管理员用户".to_string(),
        ));
    }

    // 2. 密码安全处理：使用Argon2id算法哈希密码
    let password_hash =
        hash_password(password).map_err(|e| AppError::Internal(format!("密码哈希失败: {}", e)))?;

    let pool = get_pool();
    // 3. 开启数据库事务：确保用户创建和角色分配操作的原子性
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("开启数据库事务失败: {}", e)))?;

    // 4. 唯一性校验：用户名
    check_unique_constraint(
        &mut tx,
        "username",
        username,
        &format!("用户名已存在: {}", username),
    )
    .await?;

    // 5. 唯一性校验：邮箱
    check_unique_constraint(&mut tx, "email", email, &format!("邮箱已存在: {}", email)).await?;

    // 6. 唯一性校验：手机号（如果提供）
    if let Some(phone) = phone_number {
        check_unique_constraint(
            &mut tx,
            "phone_number",
            phone,
            &format!("手机号已存在: {}", phone),
        )
        .await?;
    }

    // 7. 创建用户记录
    let user = sqlx::query!(
        r#"
        INSERT INTO admin_user (username, email, password_hash, phone_number, is_active)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        username,
        email,
        password_hash,
        phone_number,
        true
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(format!("创建用户失败: {}", e)))?;

    let user_id = user.id as i64;

    // 8. 批量获取角色ID和名称
    let mut roles = Vec::new();
    for &role_name in role_names {
        let role_id = get_role_id_by_name(role_name).await?;
        roles.push((role_id, role_name));
    }

    // 9. 批量分配角色
    for &(role_id, role_name) in &roles {
        sqlx::query!(
            "INSERT INTO user_roles (user_id, role_id, username, rolename) VALUES ($1, $2, $3, $4)",
            user_id as i32,
            role_id,
            username,
            role_name
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("分配角色 {} 失败: {}", role_name, e)))?;
    }

    // 10. 提交事务
    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("提交事务失败: {}", e)))?;

    Ok(user_id)
}

/// 使用Argon2id算法和随机盐哈希密码
///
/// # 参数
/// * `password` - 原始密码字符串
///
/// # 返回值
/// * `Ok(String)` - 加密后的密码哈希字符串
/// * `Err(_)` - 哈希过程失败
pub fn hash_password(password: &str) -> Result<String> {
    // 生成安全随机盐值（使用操作系统提供的随机数生成器）
    let salt = SaltString::generate(OsRng);

    Ok(PasswordHash::generate(Argon2::default(), &password, &salt)
        .map_err(|e| anyhow::anyhow!("密码哈希失败: {}", e))?
        .to_string())
}

/// 验证密码与Argon2哈希值是否匹配
///
/// # 参数
/// * `password` - 待验证的原始密码
/// * `password_hash` - 存储的密码哈希字符串
///
/// # 返回值
/// * `Ok(true)` - 验证成功
/// * `Ok(false)` - 验证失败
/// * `Err(_)` - 哈希解析失败
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
    let parsed_hash =
        PasswordHash::new(password_hash).map_err(|e| anyhow::anyhow!("解析密码哈希失败: {}", e))?;

    // 使用Argon2算法验证密码与哈希值的匹配性
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_or(false, |_| true))
}

/// 检查字段唯一性的通用辅助函数
async fn check_unique_constraint(
    tx: &mut sqlx::PgTransaction<'_>,
    column: &str,
    value: &str,
    error_msg: &str,
) -> Result<(), AppError> {
    // 使用EXISTS子句优化查询效率（找到匹配记录后立即停止搜索）
    // 动态构建查询以支持列名参数化，同时避免SQL注入风险
    let query_str = format!(
        "SELECT EXISTS(SELECT 1 FROM admin_user WHERE {} = $1) AS exists",
        column
    );

    // 使用基础query API执行动态SQL
    let row = sqlx::query(&query_str)
        .bind(value)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::Internal(format!("查询{}是否存在失败: {}", column, e)))?;

    // 手动提取exists字段值（PostgreSQL返回的布尔值）
    // 引入 sqlx::Row 特质，使 PgRow 可以使用 try_get 方法
    let exists: bool = row
        .try_get("exists")
        .map_err(|e| AppError::Internal(format!("解析查询结果失败: {}", e)))?;

    if exists {
        return Err(AppError::Public(error_msg.to_string()));
    }
    Ok(())
}




/// 获取所有管理员用户列表（包含创建时间和角色信息）
///
/// 只有super_admin可以查看所有管理员用户列表及权限信息
///
/// # 参数
/// * `current_user_id` - 当前登录用户ID（必须是super_admin）
///
/// # 返回值
/// * `Ok(Vec<AdminUserWithCreateTimeAndRoles>)` - 包含用户基本信息、创建时间和角色的列表
/// * `Err(AppError)` - 查询失败（权限不足/数据库错误等）
#[derive(Debug)]
pub struct AdminUsers {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub phone_number: Option<String>,
    pub _is_active: bool,
    pub _created_at: Option<time::OffsetDateTime>,
    pub _roles: Vec<String>,
}

pub async fn get_all_admin_users(
    current_user_id: i64,
) -> Result<Vec<AdminUsers>, AppError> {
    // 权限校验：仅super_admin可查看用户列表
    if !is_super_admin(current_user_id).await? {
        warn!(
            "非super_admin用户尝试获取管理员列表: current_user_id={}",
            current_user_id
        );
        return Err(AppError::Public(
            "需要super_admin权限才能查看管理员列表".to_string(),
        ));
    }

    let pool = get_pool();
    // 查询所有活跃的管理员用户基本信息及创建时间
    let users = sqlx::query!(
        r#"SELECT id, username, email, phone_number, is_active, created_at FROM admin_user WHERE is_active = true"#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Internal(format!("查询管理员用户列表失败: {}", e)))?;

    let mut result = Vec::new();
    for user in users {
        let user_id = user.id as i64;
        // 获取用户角色
        let roles = get_user_roles(user_id).await?;

        result.push(AdminUsers {
            id: user.id as i64,
            username: user.username,
            email: user.email,
            phone_number: user.phone_number,
            _is_active: user.is_active.unwrap_or(false),
            _created_at: user.created_at,
            _roles: roles,
        });
    }

    Ok(result)
}




/// 删除管理员用户
///
/// 只有super_admin可以调用此函数，通过事务确保用户删除和角色解除分配的原子性
///
/// # 参数
/// * `current_user_id` - 当前登录用户ID（必须是super_admin）
/// * `target_user_id` - 要删除的目标用户ID
///
/// # 返回值
/// * `Ok(())` - 删除成功
/// * `Err(AppError)` - 删除失败（权限不足/用户不存在/数据库错误等）
pub async fn delete_admin_user(
    current_user_id: i64,
    target_user_id: i64,
) -> Result<(), AppError> {
    // 1. 权限校验：仅super_admin可删除管理员
    if !is_super_admin(current_user_id).await? {
        warn!("删除管理员用户失败: 非super_admin用户尝试删除管理员用户");
        return Err(AppError::Public(
            "需要super_admin权限才能删除管理员用户".to_string(),
        ));
    }

    let pool = get_pool();
    // 2. 开启数据库事务：确保用户删除和角色解除分配操作的原子性
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("开启数据库事务失败: {}", e)))?;

    // 3. 检查目标用户是否存在
    let user_exists = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM admin_user WHERE id = $1 AND is_active = true) AS exists",
        target_user_id as i32
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| { 
        tracing::warn!("查询目标用户是否存在失败: {}", &e);
        AppError::Internal(format!("查询目标用户是否存在失败: {}", &e))})?;

    if !user_exists.exists.unwrap_or(false) {
        tracing::warn!("目标用户不存在或已被删除");
        return Err(AppError::Public(
            "目标用户不存在或已被删除".to_string(),
        ));
    }

    // 4. 删除用户角色关联记录
    sqlx::query!(
        "DELETE FROM user_roles WHERE user_id = $1",
        target_user_id as i32
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(format!("删除用户角色关联失败: {}", e)))?;

    // 5. 删除用户记录
    sqlx::query!(
        "DELETE FROM admin_user WHERE id = $1",
        target_user_id as i32
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(format!("删除用户失败: {}", e)))?;

    // 6. 提交事务
    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("提交事务失败: {}", e)))?;

    Ok(())
}


/// 冻结管理员用户
/// 
/// 仅超级管理员可执行此操作，通过将 `is_active` 字段设为 false 实现冻结
/// 
/// # 参数
/// * `current_user_id` - 执行操作的当前用户 ID
/// * `target_user_id` - 被冻结的目标用户 ID
/// 
/// # 返回值
/// * `Ok(())` - 冻结成功
/// * `Err(AppError)` - 权限不足、用户不存在或数据库错误等情况
pub async fn freeze_admin_user(
    current_user_id: i64,
    target_user_id: i64,
) -> Result<(), AppError> {
    // 1. 权限校验：仅super_admin可冻结管理员
    if !is_super_admin(current_user_id).await? {
        warn!("冻结管理员用户失败: 非super_admin用户尝试冻结管理员用户");
        return Err(AppError::Public(
            "需要super_admin权限才能冻结管理员用户".to_string(),
        ));
    }

    let pool = get_pool();
    // 2. 开启数据库事务
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("开启数据库事务失败: {}", e)))?;

    // 3. 检查目标用户是否存在且处于激活状态
    let user_status = sqlx::query!(
        "SELECT id, is_active FROM admin_user WHERE id = $1",
        target_user_id as i32
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        tracing::warn!("查询目标用户状态失败: {}", &e);
        AppError::Internal(format!("查询目标用户状态失败: {}", &e))
    })?;

    if !user_status.is_active.unwrap_or(false) {
        tracing::warn!("目标用户已处于冻结状态");
        return Err(AppError::Public(
            "目标用户已处于冻结状态".to_string(),
        ));
    }

    // 4. 更新用户状态为冻结（is_active = false）
    sqlx::query!(
        "UPDATE admin_user SET is_active = false, updated_at = CURRENT_TIMESTAMP WHERE id = $1",
        target_user_id as i32
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(format!("冻结用户失败: {}", e)))?;

    // 5. 提交事务
    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("提交事务失败: {}", e)))?;

    Ok(())
}
