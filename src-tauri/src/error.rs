use std::fmt::{Display, Formatter};

use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct AppError {
    pub code: String,
    pub message: String,
}

impl AppError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::new("io_error", value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::new("serde_error", value.to_string())
    }
}

impl From<image::ImageError> for AppError {
    fn from(value: image::ImageError) -> Self {
        Self::new("image_error", value.to_string())
    }
}

impl From<serde_yaml::Error> for AppError {
    fn from(value: serde_yaml::Error) -> Self {
        Self::new("yaml_error", value.to_string())
    }
}
