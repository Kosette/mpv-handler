use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to decode ({0})")]
    FailedParseUTF8(#[from] std::string::FromUtf8Error),
    #[error("Failed to decode ({0})")]
    DecodeTOMLFailed(#[from] toml::de::Error),
    #[error("Failed to decode ({0})")]
    ReadFileFailed(#[from] std::io::Error),
}
