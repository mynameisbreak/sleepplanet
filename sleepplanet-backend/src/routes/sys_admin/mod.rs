use std::time::Duration;

use crate::config::get_config;
use crate::controller::sys_admin::*;
use crate::utils::jwt::generate_token;


use salvo::http::cookie::Cookie;
use salvo::oapi::extract::JsonBody;
use salvo::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use sqlx::types::time::OffsetDateTime;
use tracing::info;
use tracing::warn;

use crate::JsonResult;
use crate::utils::error::AppError;

#[derive(Deserialize, Debug, Default)]
pub struct SysLoginIndate {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Default, Debug)]
pub struct SysLoginOutDate {
    pub id: i64,
    pub username: String,
    pub token: String,
    pub exp: i64,
}

#[handler]
pub async fn sys_login(
    idate: JsonBody<SysLoginIndate>,
    res: &mut Response,
) -> JsonResult<SysLoginOutDate> {
    let login_date = idate.into_inner();
    info!("管理员登录尝试:UserName={}", &login_date.username);
    let user = get_user_by_username(&login_date.username)
        .await?
        .ok_or_else(|| AppError::Public("用户名或密码错误".to_string()))?;
    let (user_id, username, password_hash) = user;
    if !verify_password(&login_date.password, &password_hash)? {
        warn!("管理员登录失败:UserName={}", &login_date.username);
        return Err(AppError::Public("用户名或密码错误".to_string()));
    }
    let roles = get_user_roles(user_id).await?;
    info!(
        "管理员登录成功:UserName={},Role={:?}",
        &login_date.username, &roles
    );

    let jwt_config = &get_config().jwt;

    let token = generate_token(user_id, &username, &roles)?;

    let outdata = SysLoginOutDate {
        id: user_id,
        username,
        token,
        exp: (OffsetDateTime::now_utc() + Duration::from_secs(jwt_config.expires_in as u64))
            .unix_timestamp(),
    };

    let  cookie = Cookie::build(("jwt_token",outdata.token.clone())).path("/").http_only(true).build();
    res.add_cookie(cookie);
    Ok(Json(outdata))
}



