use native_tls::HandshakeError;
use tencent_sdk::core::TencentCloudError;
use thiserror::Error;
use zip::result::ZipError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Config Error: {0}")]
    ConfigError(String),

    #[error("Cloud API Error: {0}")]
    CloudError(String),

    #[error("HTTP Error: {0}")]
    HttpError(String),

    #[error("JSON Error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Io Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Other Error: {0}")]
    Other(String),
}

impl From<native_tls::Error> for AppError {
    fn from(err: native_tls::Error) -> Self {
        AppError::HttpError(err.to_string())
    }
}

impl From<HandshakeError<std::net::TcpStream>> for AppError {
    fn from(err: HandshakeError<std::net::TcpStream>) -> Self {
        AppError::HttpError(err.to_string())
    }
}

impl From<std::string::String> for AppError {
    fn from(err: std::string::String) -> Self {
        AppError::Other(err)
    }
}

impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        AppError::Other(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::HttpError(err.to_string())
    }
}

impl From<TencentCloudError> for AppError {
    fn from(err: TencentCloudError) -> Self {
        AppError::CloudError(err.to_string())
    }
}

impl From<ZipError> for AppError {
    fn from(err: ZipError) -> Self {
        AppError::Other(err.to_string())
    }
}

impl From<base64::DecodeError> for AppError {
    fn from(err: base64::DecodeError) -> Self {
        AppError::Other(err.to_string())
    }
}
