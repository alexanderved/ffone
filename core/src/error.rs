use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO operation failed")]
    Io(#[from] io::Error),
    #[error("Serde failed")]
    Serde(#[from] serde_json::Error),
    #[error("No value")]
    None,
}
