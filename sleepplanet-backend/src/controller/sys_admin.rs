use crate::db::get_pool;
use anyhow::Result;
use bcrypt::verify;
use sqlx::PgPool;

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
    // 由于数据库返回的 id 类型可能是 i32，而期望的是 i64，因此进行类型转换
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










/// 验证密码与哈希是否匹配
///
/// 使用bcrypt算法验证原始密码与存储的哈希值是否匹配
///
/// # 参数
/// * `password` - 原始密码
/// * `hashed_password` - 存储的密码哈希
///
/// # 返回值
/// * `Ok(true)` - 密码匹配
/// * `Ok(false)` - 密码不匹配
/// * `Err(_)` - 验证过程中发生错误
pub fn verify_password(password: &str, hashed_password: &str) -> Result<bool> {
    // 修复错误处理：正确传播bcrypt验证错误
    Ok(verify(password, hashed_password)?)
}

