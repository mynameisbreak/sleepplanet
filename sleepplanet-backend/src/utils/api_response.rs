use salvo::prelude::Json;
use serde::Serialize;



/// 自定义JSON响应结构
#[derive(Serialize)]
pub(crate) struct JsonResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: T,
}


/// 统一API响应工具类
/// 提供成功响应和错误响应的标准化处理
pub struct ApiResponse;

// 定义错误码常量
// 请求成功
const CODE_SUCCESS: i32 = 200;
// 请求参数验证错误

impl ApiResponse {


    /// 生成自定义消息的成功响应
    /// # 参数
    /// * `data` - 要返回的数据
    /// * `message` - 自定义成功消息
    pub fn success<T: Serialize>(data: T, message: &str) -> Json<JsonResponse<T>> {
        Json(JsonResponse {
            code: CODE_SUCCESS,
            message: message.to_string(),
            data,
        })
    }
    
}