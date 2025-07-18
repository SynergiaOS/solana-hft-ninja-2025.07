// 🥷 Solana Client - High-Performance RPC Integration
// Optimized for sub-millisecond trading operations

use anyhow::{Result, Context};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::Signature,
    transaction::Transaction,
};
use tracing::{info, warn, debug};

/// High-performance Solana client
pub struct SolanaClient {
    rpc_client: RpcClient,
    commitment: CommitmentConfig,
}

impl SolanaClient {
    pub async fn new(rpc_url: &str) -> Result<Self> {
        let rpc_client = RpcClient::new_with_commitment(
            rpc_url.to_string(),
            CommitmentConfig::confirmed(),
        );
        
        // Test connection
        let health = rpc_client.get_health().context("Failed to connect to Solana RPC")?;
        info!("Connected to Solana RPC: {:?}", health);
        
        Ok(Self {
            rpc_client,
            commitment: CommitmentConfig::confirmed(),
        })
    }
    
    pub async fn get_health(&self) -> Result<String> {
        match self.rpc_client.get_health() {
            Ok(_) => Ok("ok".to_string()),
            Err(e) => Ok(format!("error: {}", e)),
        }
    }
    
    pub async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64> {
        self.rpc_client
            .get_balance_with_commitment(pubkey, self.commitment)
            .map(|response| response.value)
            .context("Failed to get balance")
    }
    
    pub async fn send_transaction(&self, transaction: &Transaction) -> Result<Signature> {
        self.rpc_client
            .send_and_confirm_transaction(transaction)
            .context("Failed to send transaction")
    }
    
    pub async fn get_latest_blockhash(&self) -> Result<solana_sdk::hash::Hash> {
        self.rpc_client
            .get_latest_blockhash_with_commitment(self.commitment)
            .map(|(hash, _)| hash)
            .context("Failed to get latest blockhash")
    }
}
