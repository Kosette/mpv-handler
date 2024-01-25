use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to decode ({0})")]
    FromBase64Error(#[from] base64::DecodeError),
    #[error("Failed to decode ({0})")]
    FromStringError(#[from] std::string::FromUtf8Error),
    #[error("Failed to decode ({0})")]
    FromTomlError(#[from] toml::de::Error),
    #[error("Failed to decode ({0})")]
    FromIoError(#[from] std::io::Error),
}
