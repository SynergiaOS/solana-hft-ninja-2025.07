//! Error types for mempool listener module

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MempoolError {
    #[error("WebSocket connection error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Deserialization error: {0}")]
    Deserialization(#[from] bincode::Error),

    #[error("Zero-copy deserialization error: {0}")]
    ZeroCopy(String),

    #[error("Helius API error: {0}")]
    HeliusApi(String),

    #[error("Connection timeout")]
    Timeout,

    #[error("Invalid transaction format")]
    InvalidTransaction,

    #[error("DEX program not recognized")]
    UnknownDexProgram,

    #[error("Memory limit exceeded: {0}MB")]
    MemoryLimitExceeded(usize),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, MempoolError>;
