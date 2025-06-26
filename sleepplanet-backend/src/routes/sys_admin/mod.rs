use crate::JsonResult;
use crate::config::get_config;
use crate::controller::sys_admin::*;
use crate::utils::error::AppError;
use crate::utils::jwt::generate_token;

use time::{Duration,OffsetDateTime};
use salvo::http::cookie::Cookie;
use salvo::oapi::extract::JsonBody;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use validator::Validate;
#[derive(Debug,Validate, Deserialize)]
pub struct SysLoginIndate {
    /// 用户名
    #[validate(
        length(min = 4, max = 20, message = "用户名长度需4-20位"),
        regex(
            path = "*crate::utils::validation::USERNAME_REGEX",
            message = "只允许字母、数字和下划线组合"
        )
    )]
    pub username: String,

    /// 密码
    #[validate(
        length(min = 8, max = 32, message = "密码长度需8-32位"),
        regex(
            path = "*crate::utils::validation::PASSWORD_REGEX",
            message = "密码必须包含至少一个数字"
        )
    )]
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
/// TODO:包含其他字符会抛出异常
/// TODO:缺少参数，会直接返回401
#[handler]
pub async fn sys_login(
    req: &mut Request, depot: &mut Depot, res: &mut Response,
) -> JsonResult<SysLoginOutDate> {
    let login_data = req.parse_json::<SysLoginIndate>().await.map_err(|e| {
        tracing::error!(error = %e, "登录请求数据解析失败");
        AppError::Public("SysLoginDate解析错误".to_string())
    })?;
    
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
    let outdata = SysLoginOutDate {
        id: user_id,
        username: (&username).to_string(),
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


