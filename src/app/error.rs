use thiserror::Error;

#[derive(Error, Debug)]
pub enum ErisError {
    #[error("Failed to open and read file on disk.")]
    ReadFromDisk,
    #[error("Failed to write file to disk.")]
    WriteToDisk,
    #[error("Failed to deserialize from a file.")]
    DeserializeFromFile,
    #[error("Failed to serialize into a file.")]
    SerializeToFile,
    #[error("Unknown error.")]
    Unknown,
}

impl From<anyhow::Error> for ErisError {
    fn from(_err: anyhow::Error) -> Self {
        ErisError::Unknown
    }
}
