use once_cell::sync::Lazy;
use regex::{Regex};

// 为 Lazy<Regex> 实现 AsRegex trait

pub static USERNAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_]+").expect("无效的用户名正则表达式"));

pub static PASSWORD_REGEX: Lazy<Regex> =
    // 密码复杂度正则表达式（仅要求包含数字）
    Lazy::new(|| Regex::new(r"^(?=.*\d)").expect("密码必须包含至少一个数字"));
