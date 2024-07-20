use http::status::StatusCode;
use leptos_router::ParamsError;
use thiserror::Error;

// 应用错误枚举类型
#[derive(Clone, Debug, Error)]
pub enum AppError {
    // 未找到资源错误
    #[error("Not Found")]
    NotFound,
    // 参数读取错误，并包含原始错误信息
    #[error("Error reading new game settings: {0}")]
    ParamsError(#[from] ParamsError),
}

impl AppError {
    // 返回对应的HTTP状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            // 未找到资源错误对应404状态码
            AppError::NotFound => StatusCode::NOT_FOUND,
            // 参数错误对应400状态码
            AppError::ParamsError(_) => StatusCode::BAD_REQUEST,
        }
    }
}
