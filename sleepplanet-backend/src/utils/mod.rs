//! 通用工具函数和错误处理模块
//! 包含应用程序中常用的工具函数、错误类型定义和错误处理机制

pub mod error;

// 导出错误处理相关类型和宏
pub use error::AppError;
pub use error::ErrorResponse;
pub use error::anyhow_err;
pub use error::validation_err;
pub use error::auth_err;
pub use error::business_err;