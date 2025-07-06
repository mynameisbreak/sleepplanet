use salvo::handler;
use salvo::prelude::*;

use crate::config::get_config;
use crate::utils::jwt::auth_hoop;

pub mod sys_admin;

#[handler]
pub async fn hello_world(res: &mut Response) {
    res.render(Text::Plain("Hello, Salvo!"));
}

pub fn root() -> Router {
    // 构建并返回Router
    Router::new()
        .get(hello_world)
        .push(
            Router::with_path("sys")
                .hoop(auth_hoop(&get_config().jwt))
                .push(Router::with_path("login").post(sys_admin::sys_login))
                .push(Router::with_path("logout").post(sys_admin::sys_logout))
                .push(Router::with_path("create_sys_user").post(sys_admin::create_sys_user))
                .push(Router::with_path("delete_sys_user").post(sys_admin::delete_admin))
                .push(Router::with_path("freeze_sys_user/{user_id}").get(sys_admin::freeze_admin))
                .push(Router::with_path("users").get(sys_admin::get_admin_users)),
        )
}
