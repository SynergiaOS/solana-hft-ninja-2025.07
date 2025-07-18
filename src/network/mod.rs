// 🥷 Network Layer - High-Performance Solana Integration
// Optimized for sub-millisecond trading operations

pub mod solana;
pub mod websocket;
pub mod rpc;

// Re-export main types
pub use solana::SolanaClient;
pub use websocket::WebSocketClient;
pub use rpc::RpcClient;

use crate::core::types::*;
use crate::utils::config::Config;
use anyhow::Result;

/// Data collector for market data
pub struct DataCollector {
    config: Config,
}

impl DataCollector {
    pub async fn new(config: Config) -> Result<Self> {
        Ok(Self { config })
    }
    
    pub async fn start(&self, sources: Vec<String>) -> Result<()> {
        tracing::info!("Starting data collection from sources: {:?}", sources);
        
        // Placeholder implementation
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}
