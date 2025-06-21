use salvo::prelude::*;
use thiserror::Error;

/// 应用错误类型，包含各种可能的错误场景
#[derive(Error, Debug)]
pub enum AppError {
    /// 数据库操作错误，通常由底层数据库操作失败引起
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] ::sqlx::Error),

    /// 参数验证错误，用于处理请求参数不符合要求的情况
    #[error("验证错误: {0}")]
    ValidationError(#[from] validator::ValidationErrors),

    /// 资源未找到错误，当请求的资源不存在时使用
    #[error("未找到")]
    NotFound,

    /// 未授权错误，用于需要身份验证但验证失败的情况
    #[error("未授权")]
    Unauthorized,

    /// 禁止访问错误，表示用户无权访问该资源
    #[error("禁止访问")]
    Forbidden,

    /// 内部服务器错误，用于处理其他未明确分类的错误
    #[error("内部服务器错误")]
// 移除 `#[from]` 属性以避免 `From<anyhow::Error>` 实现冲突
    InternalServerError(anyhow::Error),

    /// HTTP请求错误，包含状态码和错误信息
    #[error("HTTP请求错误: {status_code} {message}")]
    HttpRequestError { status_code: u16, message: String },

    /// Salvo内部错误
    #[error("salvo internal error: `{0}`")]
    Salvo(#[from] salvo::Error),

    /// HTTP状态错误
    #[error("http status error: `{0}`")]
    HttpStatus(#[from] StatusError),

    /// HTTP解析错误
    #[error("http parse error:`{0}`")]
    HttpParse(#[from] ParseError),

    /// JSON序列化/反序列化错误
    #[error("JSON处理错误: {0}")]
    JsonError(#[from] serde_json::Error),

    /// 文件操作错误
    #[error("文件操作错误: {0}")]
    FileError(#[from] std::io::Error),

    /// JWT认证错误
    #[error("认证错误: {0}")]
    AuthError(String),

    /// 业务逻辑错误，用于特定业务规则的违反
    #[error("业务逻辑错误: {0}")]
    BusinessError(String),
}

impl AppError {
    /// 获取错误对应的HTTP状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::HttpRequestError { status_code, .. } => {
                StatusCode::from_u16(*status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            }
            AppError::Salvo(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::HttpStatus(e) => StatusCode::from_u16(e.0).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            AppError::HttpParse(_) => StatusCode::BAD_REQUEST,
            AppError::JsonError(_) => StatusCode::BAD_REQUEST,
            AppError::FileError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AuthError(_) => StatusCode::UNAUTHORIZED,
            AppError::BusinessError(_) => StatusCode::CONFLICT,
        }
    }



/// 错误响应Trait，用于将错误转换为Salvo结果




impl ErrorResponse for AppError {
    fn status_code(&self) -> StatusCode {
        AppError::status_code(self)
    }
    
    fn user_message(&self) -> String {
        AppError::user_message(self)
    }
    fn into_result(self) -> SalvoResult<Self> {
        Err(salvo::Error::Other(Box::new(self)))
    }
}

/// 实现Writer trait，用于将错误转换为HTTP响应
#[async_trait::async_trait]
impl Writer for AppError {
    async fn write<'a>(
        self,
        _req: &'a mut salvo::Request,
        _depot: &'a mut salvo::Depot,
        res: &'a mut salvo::Response,
    ) -> salvo::Result<()> {
        let status = self.status_code();
        res.status_code(status);
        
        // 记录错误日志
        error!("处理请求时发生错误: {:?}", self);

        // 构建标准错误响应JSON
        let body = match serde_json::to_string(&self.to_json_response()) {
            Ok(json) => json,
            // 如果JSON序列化失败，返回一个基本错误响应
            Err(_) => r#"{"code":500,"message":"Failed to serialize error","user_message":"服务器内部错误"}"#.to_string(),
        };

        res.render(Text::Plain(body));
        Ok(())
    }
}