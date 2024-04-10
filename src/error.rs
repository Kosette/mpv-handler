use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Base64 failed to decode ({0})")]
    UnableToDecode(#[from] base64::DecodeError),
    #[error("Convert to String failed")]
    StringReadFailed(#[from] std::string::FromUtf8Error),
    #[error("Failed to parse config")]
    ConfigParseFailed(#[from] toml::de::Error),
    #[error("Reading error")]
    Disconnect(#[from] std::io::Error),
}
