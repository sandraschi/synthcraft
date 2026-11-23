//! Error types for SynthCraft L1 core.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SynthError {
    #[error("Patch or resource not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Audio I/O or WAV export error: {0}")]
    Io(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("SQLite storage error: {0}")]
    Storage(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}
