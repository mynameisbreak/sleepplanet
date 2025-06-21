use salvo::{http::ParseError, prelude::*};
use serde::Serialize;
use thiserror::Error;
use tracing::error;

/// 自定义JSON响应结构
#[derive(Serialize)]
struct JsonResponse<T> {
    code: i32,
    message: String,
    data: T,
}

/// 应用错误类型，包含各种可能的错误场景
#[derive(Error, Debug)]
pub enum AppError {
    /// 公共错误，用于客户端可见的错误信息
    #[error("public: `{0}`")]
    Public(String),

    /// 内部错误，用于服务器内部错误，不向客户端暴露详细信息
    #[error("internal: `{0}`")]
    Internal(String),

    /// Salvo框架内部错误
    #[error("salvo internal error: `{0}`")]
    Salvo(#[from] ::salvo::Error),

    /// HTTP状态错误
    #[error("http status error: `{0}`")]
    HttpStatus(#[from] StatusError),

    /// HTTP解析错误
    #[error("http parse error: `{0}`")]
    HttpParse(#[from] ParseError),

    /// Anyhow错误包装
    #[error("anyhow error: `{0}`")]
    Anyhow(#[from] anyhow::Error),

    /// SQLx数据库错误
    #[error("sqlx::Error: `{0}`")]
    SqlxError(#[from] sqlx::Error),

    /// 参数验证错误
    #[error("validation error: `{0}`")]
    Validation(#[from] validator::ValidationErrors),
}

impl AppError {
    /// 将错误映射为HTTP状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Public(_) => StatusCode::BAD_REQUEST,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Salvo(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::HttpStatus(e) => e.code,
            AppError::HttpParse(_) => StatusCode::BAD_REQUEST,
            AppError::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
        }
    }
}

/// 实现Salvo的Writer特征，将错误转换为JSON响应
#[async_trait]
impl Writer for AppError {

    /// 将错误序列化为JSON响应
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        let status_code = self.status_code();
        res.status_code(status_code);
        
        let message = match &self {
            AppError::Public(msg) => msg.clone(),
            AppError::Internal(msg) => {
                error!("内部服务器错误: {}", msg);
                "服务器内部发生错误".to_string()
            },
            AppError::Salvo(e) => {
                error!("Salvo框架错误: {:?}", e);
                "服务器处理请求失败".to_string()
            },
            AppError::HttpStatus(e) => e.to_string(),
            AppError::HttpParse(e) => {
                error!("HTTP解析错误: {:?}", e);
                format!("请求格式无效: {}", e)
            },
            AppError::Anyhow(e) => {
                error!("操作错误: {:?}", e);
                "内部操作执行失败".to_string()
            },
            AppError::SqlxError(e) => {
                error!("数据库错误: {:?}", e);
                "数据库操作失败".to_string()
            },
            AppError::Validation(e) => {
                error!("参数验证错误: {:?}", e);
                format!("输入参数无效: {}", e)
            },
        };
        
        let response = JsonResponse {
            code: status_code.as_u16() as i32,
            message,
            data: (),
        };
        res.render(Json(response));
    }
}
