use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Base64 failed to decode: {0}")]
    UnableToDecode(#[from] base64::DecodeError),
    #[error("Convert to String failed: {0}")]
    StringReadFailed(#[from] std::string::FromUtf8Error),
    #[error("Failed to parse config: {0}")]
    ConfigParseFailed(#[from] toml::de::Error),
    #[error("Read error: {0}")]
    IOFailed(#[from] std::io::Error),
    #[error("Bad Status Code: {0}")]
    BadStatus(reqwest::StatusCode),
    #[error("Request timeout: {0}")]
    TimeOut(#[from] reqwest::Error),
}
