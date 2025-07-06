use crate::config::JwtConfig;
use crate::config::get_config;
use crate::utils::error::AppError;
use jsonwebtoken::errors::{Error, ErrorKind}; // 确保导入错误类型
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use salvo::jwt_auth::{ConstDecoder, CookieFinder, HeaderFinder, QueryFinder};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
// 定义一个结构体 `Claims`，用于表示 JWT 中的声明信息
pub struct Claims {
    // 用户的唯一标识符
    pub user_id: i64,
    // 用户名
    pub username: String,
    // 用户角色
    pub role: String,
    // 令牌的过期时间戳
    pub exp: u64,
}

pub fn auth_hoop(config: &JwtConfig) -> JwtAuth<Claims, ConstDecoder> {
    JwtAuth::new(ConstDecoder::from_secret(
        config.secret.to_owned().as_bytes(),
    ))
    .finders(vec![
        Box::new(HeaderFinder::new()),
        Box::new(QueryFinder::new("token")),
        Box::new(CookieFinder::new("jwt_token")),
    ])
    .force_passed(true)
}

// 生成 JWT 令牌的函数
// 参数:
// - user_id: 用户的唯一标识符
// - username: 用户名
// - role: 用户角色
// 返回值:
// - anyhow::Result<String>: 包含生成的 JWT 令牌的结果，如果生成失败则包含错误信息
pub fn generate_token(user_id: i64, username: &str, roles: &Vec<String>) -> anyhow::Result<String> {
    // 获取配置信息
    let config = get_config();
    // 创建 JWT 声明信息结构体
    let claims = Claims {
        user_id,
        // 将传入的用户名转换为 String 类型
        username: username.to_string(),
        // 将传入的用户角色转换为 String 类型
        role: roles.join(","),
        // 计算令牌的过期时间戳，当前时间加上配置中的过期时间
        exp: (chrono::Utc::now().timestamp() as u64 + config.jwt.expires_in),
    };
    // 使用 jsonwebtoken 库的 encode 函数生成 JWT 令牌
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt.secret.as_bytes()),
    )?;
    // 返回生成的 JWT 令牌
    Ok(token)
}

// 验证 JWT 令牌的函数
// 参数:
// - token: 待验证的 JWT 令牌
// 返回值:
// - anyhow::Result<Claims>: 包含解析后的 JWT 声明信息的结果，如果验证失败则包含错误信息
pub fn verify_token(token: &str) -> anyhow::Result<Claims> {
    // 获取配置信息
    let config = get_config();
    // 检查配置中的 JWT 配置是否有效
    if config.jwt.secret.is_empty() {
        return Err(anyhow::anyhow!("JWT secret is empty"));
    }
    if config.jwt.expires_in == 0 {
        return Err(anyhow::anyhow!("JWT expires_in is invalid"));
    }

    // 显式启用exp过期时间校验
    let mut validation = Validation::default();
    validation.algorithms = vec![Algorithm::HS256];
    validation.validate_exp = true;

    // 使用 match 语句捕获并处理校验错误
    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt.secret.as_bytes()),
        &validation,
    ) {
        Ok(data) => data,
        Err(e) => match e.kind() {
            ErrorKind::ExpiredSignature => {
                // 处理令牌过期错误
                
                return Err(anyhow::anyhow!("令牌已过期"));
            }
            ErrorKind::InvalidSignature => {
                // 处理签名无效错误
                return Err(anyhow::anyhow!("无效的令牌签名"));
            }
            ErrorKind::InvalidToken => {
                // 处理令牌格式错误
                return Err(anyhow::anyhow!("令牌格式无效"));
            }
            _ => {
                // 处理其他校验错误
                return Err(anyhow::anyhow!("令牌验证失败"));
            }
        },
    };
    // 返回解析后的 JWT 声明信息
    Ok(token_data.claims)
}
