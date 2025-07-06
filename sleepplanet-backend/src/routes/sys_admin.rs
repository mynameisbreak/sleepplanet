use std::fmt::format;

use crate::config::get_config;
use crate::controller::sys_admin::*;
use crate::utils::api_response::{ApiResponse, JsonResponse};
use crate::utils::error::AppError;
use crate::utils::jwt::{Claims, generate_token, verify_token};

use jsonwebtoken::errors::ErrorKind;
// 外部依赖（按字母序排列）
use salvo::http::cookie::Cookie;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use tracing::{error, info, warn};
use validator::Validate;

// 内部依赖（按模块层级排序）

// 请求数据结构体（按功能相关性排序）
#[derive(Debug, Validate, Deserialize)]
pub struct SysLoginIndate {
    /// 用户名
    #[validate(length(min = 4, max = 20, message = "用户名长度需4-20位"))]
    pub username: String,

    /// 密码
    #[validate(length(min = 8, max = 32, message = "密码长度需8-32位"))]
    pub password: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct SysUserCreateData {
    /// 用户名（4-20位）
    #[validate(length(min = 4, max = 20, message = "用户名长度需4-20位"))]
    pub username: String,
    /// 密码（8-32位）
    #[validate(length(min = 8, max = 32, message = "密码长度需8-32位"))]
    pub password: String,
    /// 邮箱（必填，格式验证）
    #[validate(
        length(min = 1, message = "邮箱不能为空"),
        email(message = "邮箱格式不正确")
    )]
    pub email: String,
    /// 手机号（可选，11位数字）
    #[validate(length(min = 11, max = 11, message = "手机号必须为11位数字"))]
    pub phone_number: Option<String>,
    /// 角色名称列表（至少一个）
    #[validate(length(min = 1, message = "请至少指定一个角色"))]
    pub role_names: Vec<String>,
}

// 响应数据结构体
#[derive(Serialize, Default, Debug)]
pub struct SysLoginOutDate {
    /// 用户ID
    pub id: i64,
    /// 用户名
    pub username: String,
}

/// 登录响应数据结构
#[derive(Serialize)]
struct LoginResponse {
    /// 用户ID
    pub user_id: i64,
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
    req: &mut Request,
    _depot: &mut Depot,
    res: &mut Response,
) -> Result<Json<JsonResponse<LoginResponse>>, AppError> {
    let login_data = req.parse_json::<SysLoginIndate>().await.map_err(|e| {
        tracing::error!(error = %e, "登录请求数据解析失败");
        AppError::Public("登录请求数据解析错误".to_string())
    })?;

    if login_data.username.is_empty() || login_data.password.is_empty() {
        warn!("登录参数验证失败: 用户名或密码不能为空");
        AppError::Public("用户名或密码不能为空".to_string());
    }

    login_data.validate().map_err(|e| {
        warn!("登录参数验证失败: {:?}", e);
        AppError::Public(format!("登录验证失败: {}", e))
    })?;
    info!("管理员登录尝试:UserName={}", &login_data.username);

    // 查询用户信息
    let user = get_user_by_username(&login_data.username)
        .await?
        .ok_or_else(|| AppError::Public("用户名或密码错误".to_string()))?;
    let (user_id, username, password_hash) = user;

    // 验证密码
    match verify_password(&login_data.password, &password_hash) {
        Ok(true) => (),

        Ok(false) => {
            warn!("管理员登录失败:UserName={}", &login_data.username);
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
        &login_data.username, &roles
    );

    // 生成JWT令牌
    let jwt_config = &get_config().jwt;
    let token = generate_token(user_id, &username, &roles)?;

    // 构建响应数据
    let login_response = LoginResponse {
        user_id,
        username: username.to_string(),
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

    Ok(ApiResponse::success(login_response, "登录成功"))
}

/// 管理员登出处理器
#[handler]
pub async fn sys_logout(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<Json<JsonResponse<()>>, AppError> {
    match depot.jwt_auth_state() {
        JwtAuthState::Authorized => {
            let data = depot
                .jwt_auth_data::<Claims>()
                .ok_or(AppError::Public("JWT数据获取失败".to_string()))?;

            res.remove_cookie("jwt_token");
            info!("管理员登出:UserName={}", &data.claims.username);
            Ok(ApiResponse::success((), "登出成功"))
        }
        JwtAuthState::Forbidden => {
              handle_jwt_auth_error(&depot, &req)
                .map_err(|e| e)
                .and_then(|_| Err(AppError::Public("拒绝访问".to_string())))
        }
        JwtAuthState::Unauthorized => {
            tracing::warn!(target: "jwt_auth", "凭证失效: path={}", req.uri());
            res.status_code(StatusCode::UNAUTHORIZED);
            Err(AppError::Public("凭证失效，请重新登录".to_string()))
        }
    }
}

/// 创建管理员用户处理器
#[handler]
pub async fn create_sys_user(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<Json<JsonResponse<String>>, AppError> {
    match depot.jwt_auth_state() {
        JwtAuthState::Authorized => {
            // 1. 获取并验证JWT数据（合并重复逻辑）
            let claims_data = depot
                .jwt_auth_data::<Claims>()
                .ok_or(AppError::Public("JWT数据获取失败".to_string()))?;

            // 3. 解析并验证请求数据
            let create_data = req.parse_json::<SysUserCreateData>().await.map_err(|e| {
                tracing::error!(error = %e, "创建用户请求数据解析失败");
                AppError::Public("用户创建数据解析错误".to_string())
            })?;
            create_data.validate().map_err(|e| {
                warn!("用户创建参数验证失败: {:?}", e);
                AppError::Public(format!("用户创建验证失败: {}", e))
            })?;

            // 4. 创建用户（优化角色名称转换代码）
            let current_user_id = claims_data.claims.user_id;
            let role_names: Vec<&str> = create_data.role_names.iter().map(String::as_str).collect();
            let user_id = create_admin_user(
                current_user_id,
                &create_data.username,
                &create_data.password,
                &create_data.email,
                create_data.phone_number.as_deref(),
                &role_names,
            )
            .await?;

            info!("管理员用户创建成功: user_id={}", user_id);
            Ok(ApiResponse::success(
                "管理员用户创建成功".to_string(),
                "用户创建成功",
            ))
        }
        JwtAuthState::Forbidden => {
              handle_jwt_auth_error(&depot, &req)
                .map_err(|e| e)
                .and_then(|_| Err(AppError::Public("拒绝访问".to_string())))
        }
        JwtAuthState::Unauthorized => {
            tracing::warn!(target: "jwt_auth", "凭证失效: path={}", req.uri());
            res.status_code(StatusCode::UNAUTHORIZED);
            Err(AppError::Public("凭证失效，请重新登录".to_string()))
        }
    }
}

/// 获取管理员用户列表处理器
#[handler]
pub async fn get_admin_users(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<Json<JsonResponse<Vec<AdminInfo>>>, AppError> {
    match depot.jwt_auth_state() {
        JwtAuthState::Authorized => {
            // 1. 获取并验证JWT数据
            let claims_data = depot
                .jwt_auth_data::<Claims>()
                .ok_or(AppError::Public("JWT数据获取失败".to_string()))?;

            // 3. 获取管理员用户列表
            let admin_users = get_all_admin_users(claims_data.claims.user_id).await?;
            info!("获取管理员用户列表成功，数量: {}", admin_users.len());
            // 将元组转换为AdminInfo结构体
            let result: Vec<AdminInfo> = admin_users
                .into_iter()
                .map(|user| AdminInfo {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    phone_number: user.phone_number,
                })
                .collect();
            Ok(ApiResponse::success(result, "获取管理员用户列表成功"))
        }
        JwtAuthState::Forbidden => {


            handle_jwt_auth_error(&depot, &req)
                .map_err(|e| e)
                .and_then(|_| Err(AppError::Public("拒绝访问".to_string())))
        }
        JwtAuthState::Unauthorized => {
            tracing::warn!(target: "jwt_auth", "凭证失效: path={}", req.uri());
            res.status_code(StatusCode::UNAUTHORIZED);
            Err(AppError::Public("凭证失效，请重新登录".to_string()))
        }
    }
}

pub(crate) fn handle_jwt_auth_error(
    depot: &Depot,
    req: &Request,
) -> Result<Json<JsonResponse<()>>, AppError> {
    match depot.jwt_auth_error() {
        Some(error) => match *error.kind() {
            ErrorKind::ExpiredSignature => {
                tracing::warn!(target: "jwt_auth", "凭证已过期: path={} ", req.uri());
                Err(AppError::Public("凭证已过期，请重新登录".to_string()))
            }
            ErrorKind::InvalidSignature => {
                tracing::warn!(target: "jwt_auth", "凭证无效: path={} ", req.uri());
                Err(AppError::Public("凭证无效，请重新登录".to_string()))
            }
            ErrorKind::InvalidToken | ErrorKind::InvalidIssuer | ErrorKind::InvalidAudience => {
                tracing::warn!(target: "jwt_auth", "令牌格式错误: {:?}, path={}", error, req.uri());
                Err(AppError::Public("令牌格式错误，请检查请求".to_string()))
            }
            ErrorKind::ImmatureSignature => {
                tracing::warn!(target: "jwt_auth", "令牌尚未生效: path={}", req.uri());
                Err(AppError::Public("令牌尚未生效，请稍后再试".to_string()))
            }
            _ => {
                tracing::error!(target: "jwt_auth", "认证处理错误: {:?}, path={}", error, req.uri());
                Err(AppError::Internal("认证处理失败，请联系管理员".to_string()))
            }
        },
        None => {
            // 没有认证错误，继续处理请求
            Err(AppError::Public("拒绝访问".to_string()))
        }
    }
}

// 在请求数据结构体区域添加AdminInfo结构体定义
#[derive(Debug, Serialize)]
pub struct AdminInfo {
    /// 用户ID
    pub id: i64,
    /// 用户名
    pub username: String,
    /// 邮箱
    pub email: String,
    /// 手机号
    pub phone_number: Option<String>,
}

/// 删除管理员用户处理器
#[handler]
pub async fn delete_admin(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<Json<JsonResponse<String>>, AppError> {
    match depot.jwt_auth_state() {
        JwtAuthState::Authorized => {
            // 1. 获取并验证JWT数据
            let claims_data = depot
                .jwt_auth_data::<Claims>()
                .ok_or(AppError::Public("JWT数据获取失败".to_string()))?;

            // 2. 从请求中获取要删除的用户ID
            let target_user_id = req
                .param::<i64>("user_id")
                .ok_or(AppError::Public("缺少用户ID参数".to_string()))?;

            // 3. 删除用户
            delete_admin_user(claims_data.claims.user_id, target_user_id).await?;

            info!("管理员用户删除成功: user_id={}", target_user_id);
            Ok(ApiResponse::success(
                "管理员用户删除成功".to_string(),
                "用户删除成功",
            ))
        }
        JwtAuthState::Forbidden => {
            handle_jwt_auth_error(&depot, &req)
                .map_err(|e| e)
                .and_then(|_| Err(AppError::Public("拒绝访问".to_string())))
        }
        JwtAuthState::Unauthorized => {
            tracing::warn!(target: "jwt_auth", "凭证失效: path={}", req.uri());
            res.status_code(StatusCode::UNAUTHORIZED);
            Err(AppError::Public("凭证失效，请重新登录".to_string()))
        }
    }
}

/// 冻结管理员用户处理器
#[handler]
pub async fn freeze_admin(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<Json<JsonResponse<String>>, AppError> {
    match depot.jwt_auth_state() {
        JwtAuthState::Authorized => {
            // 1. 获取并验证JWT数据
            let claims_data = depot
                .jwt_auth_data::<Claims>()
                .ok_or(AppError::Public("JWT数据获取失败".to_string()))?;

            // 2. 从请求中获取要冻结的用户ID
            let target_user_id = req
                .param::<i64>("user_id")
                .ok_or(AppError::Public("缺少用户ID参数".to_string()))?;

            // 3. 冻结用户
            freeze_admin_user(claims_data.claims.user_id, target_user_id).await?;

            info!("管理员用户冻结成功: user_id={}", target_user_id);
            Ok(ApiResponse::success(
                "管理员用户冻结成功".to_string(),
                "用户冻结成功",
            ))
        }
        JwtAuthState::Forbidden => {
            handle_jwt_auth_error(&depot, &req)
                .map_err(|e| e)
                .and_then(|_| Err(AppError::Public("拒绝访问".to_string())))
        }
        JwtAuthState::Unauthorized => {
            tracing::warn!(target: "jwt_auth", "凭证失效: path={}", req.uri());
            res.status_code(StatusCode::UNAUTHORIZED);
            Err(AppError::Public("凭证失效，请重新登录".to_string()))
        }
    }
}
