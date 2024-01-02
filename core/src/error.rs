use std::io;

use crate::device::DeviceInfo;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("IO operation failed: {0}")]
    Io(#[from] io::Error),
    #[error("Serde operation failed: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Network packet has wrong header")]
    WrongNetworkPacketHeader,
    #[error("No device was found")]
    NoDevice,
    #[error("The device cannot be reached")]
    DeviceUnreachable(DeviceInfo),
    #[error("The device is not linked anymore")]
    DeviceUnlinked,
    #[error("The transition from the current runnable state to the next one is forbidden")]
    WrongRunnableState,
    #[error("Failed to cast integer to enum")]
    IntToEnumCastFailed,
    #[error("Failed to parse encoded audio header")]
    EncodedAudioHeaderParseFailed,
    #[error("Other error occured: {0}")]
    Other(String),
}
