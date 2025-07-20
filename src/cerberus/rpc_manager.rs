use anyhow::{Result, anyhow};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn, error, info};
use crate::cerberus::{MarketData};

/// Dual RPC manager with QuickNode primary + Helius fallback
pub struct RpcManager {
    primary_client: Arc<RpcClient>,
    fallback_client: Arc<RpcClient>,
    primary_endpoint: String,
    fallback_endpoint: String,
    primary_healthy: Arc<RwLock<bool>>,
    fallback_healthy: Arc<RwLock<bool>>,
    last_health_check: Arc<RwLock<Instant>>,
    health_check_interval: Duration,
}

impl RpcManager {
    /// Create new RPC manager with dual endpoints
    pub async fn new(
        quicknode_endpoint: &str,
        helius_endpoint: &str,
    ) -> Result<Self> {
        info!("🌐 Initializing dual RPC manager");
        info!("📊 Primary (QuickNode): {}", quicknode_endpoint);
        info!("📊 Fallback (Helius): {}", helius_endpoint);

        let primary_client = Arc::new(RpcClient::new_with_timeout(
            quicknode_endpoint.to_string(),
            Duration::from_secs(10),
        ));

        let fallback_client = Arc::new(RpcClient::new_with_timeout(
            helius_endpoint.to_string(),
            Duration::from_secs(10),
        ));

        let manager = Self {
            primary_client,
            fallback_client,
            primary_endpoint: quicknode_endpoint.to_string(),
            fallback_endpoint: helius_endpoint.to_string(),
            primary_healthy: Arc::new(RwLock::new(true)),
            fallback_healthy: Arc::new(RwLock::new(true)),
            last_health_check: Arc::new(RwLock::new(Instant::now())),
            health_check_interval: Duration::from_secs(30),
        };

        // Initial health check
        manager.check_health().await;

        Ok(manager)
    }

    /// Get the best available RPC client
    async fn get_best_client(&self) -> Arc<RpcClient> {
        let primary_healthy = *self.primary_healthy.read().await;
        let fallback_healthy = *self.fallback_healthy.read().await;

        // Check if we need to run health check
        let last_check = *self.last_health_check.read().await;
        if last_check.elapsed() > self.health_check_interval {
            tokio::spawn({
                let manager = self.clone();
                async move {
                    manager.check_health().await;
                }
            });
        }

        // Return best available client
        if primary_healthy {
            debug!("🟢 Using primary RPC (QuickNode)");
            Arc::clone(&self.primary_client)
        } else if fallback_healthy {
            warn!("🟡 Primary RPC down, using fallback (Helius)");
            Arc::clone(&self.fallback_client)
        } else {
            error!("🔴 Both RPC endpoints unhealthy, using primary anyway");
            Arc::clone(&self.primary_client)
        }
    }

    /// Check health of both endpoints
    async fn check_health(&self) {
        debug!("🏥 Running RPC health check");

        // Check primary
        let primary_healthy = self.check_endpoint_health(&self.primary_client).await;
        *self.primary_healthy.write().await = primary_healthy;

        // Check fallback
        let fallback_healthy = self.check_endpoint_health(&self.fallback_client).await;
        *self.fallback_healthy.write().await = fallback_healthy;

        // Update last check time
        *self.last_health_check.write().await = Instant::now();

        info!("🏥 Health check complete - Primary: {}, Fallback: {}", 
              if primary_healthy { "✅" } else { "❌" },
              if fallback_healthy { "✅" } else { "❌" });
    }

    /// Check health of a single endpoint
    async fn check_endpoint_health(&self, client: &RpcClient) -> bool {
        // Use a simple balance check instead of get_health which is not async
        match tokio::time::timeout(
            Duration::from_secs(5),
            async { client.get_balance(&solana_sdk::pubkey::Pubkey::default()) }
        ).await {
            Ok(Ok(_)) => true,
            Ok(Err(e)) => {
                debug!("RPC health check failed: {}", e);
                false
            },
            Err(_) => {
                debug!("RPC health check timed out");
                false
            }
        }
    }

    /// Get market data for a token
    pub async fn get_market_data(&self, mint: &str) -> Result<MarketData> {
        let client = self.get_best_client().await;
        
        // Parse mint address
        let mint_pubkey = Pubkey::from_str(mint)
            .map_err(|e| anyhow!("Invalid mint address {}: {}", mint, e))?;

        // For now, we'll use a simple price fetch
        // In production, this would integrate with Jupiter, Raydium, or Orca APIs
        let price = self.fetch_token_price(&client, &mint_pubkey).await?;
        
        let mut market_data = MarketData::new(mint.to_string(), price);
        
        // Fetch additional market data
        if let Ok(volume) = self.fetch_24h_volume(&client, &mint_pubkey).await {
            market_data.volume_24h = volume;
        }

        if let Ok(liquidity) = self.fetch_liquidity(&client, &mint_pubkey).await {
            market_data.liquidity = liquidity;
        }

        Ok(market_data)
    }

    /// Fetch token price (placeholder - integrate with Jupiter/Raydium)
    async fn fetch_token_price(&self, client: &RpcClient, mint: &Pubkey) -> Result<f64> {
        // This is a placeholder implementation
        // In production, you would:
        // 1. Query Jupiter for price
        // 2. Query Raydium pools
        // 3. Query Orca whirlpools
        // 4. Use the best available price
        
        debug!("💰 Fetching price for {}", mint);
        
        // For now, return a mock price
        // TODO: Implement real price fetching
        Ok(0.001) // Placeholder price
    }

    /// Fetch 24h volume (placeholder)
    async fn fetch_24h_volume(&self, _client: &RpcClient, _mint: &Pubkey) -> Result<f64> {
        // TODO: Implement volume fetching from DEX APIs
        Ok(10000.0) // Placeholder volume
    }

    /// Fetch liquidity (placeholder)
    async fn fetch_liquidity(&self, _client: &RpcClient, _mint: &Pubkey) -> Result<f64> {
        // TODO: Implement liquidity fetching from DEX APIs
        Ok(50000.0) // Placeholder liquidity
    }

    /// Get account balance
    pub async fn get_account_balance(&self, account: &Pubkey) -> Result<u64> {
        let client = self.get_best_client().await;
        
        match client.get_balance(account) {
            Ok(balance) => Ok(balance),
            Err(e) => {
                error!("Failed to get account balance: {}", e);
                Err(anyhow!("Failed to get account balance: {}", e))
            }
        }
    }

    /// Get token account balance
    pub async fn get_token_balance(&self, token_account: &Pubkey) -> Result<u64> {
        let client = self.get_best_client().await;
        
        match client.get_token_account_balance(token_account) {
            Ok(balance) => {
                Ok(balance.amount.parse().unwrap_or(0))
            },
            Err(e) => {
                error!("Failed to get token balance: {}", e);
                Err(anyhow!("Failed to get token balance: {}", e))
            }
        }
    }

    /// Send transaction with retry logic
    pub async fn send_transaction(&self, transaction: &solana_sdk::transaction::Transaction) -> Result<String> {
        let client = self.get_best_client().await;
        
        // Try primary first
        match client.send_and_confirm_transaction(transaction) {
            Ok(signature) => {
                debug!("✅ Transaction sent successfully: {}", signature);
                Ok(signature.to_string())
            },
            Err(e) => {
                warn!("❌ Transaction failed on primary RPC: {}", e);
                
                // Try fallback if primary failed
                if *self.fallback_healthy.read().await {
                    match self.fallback_client.send_and_confirm_transaction(transaction) {
                        Ok(signature) => {
                            debug!("✅ Transaction sent via fallback: {}", signature);
                            Ok(signature.to_string())
                        },
                        Err(e2) => {
                            error!("❌ Transaction failed on both RPCs: primary={}, fallback={}", e, e2);
                            Err(anyhow!("Transaction failed on both RPCs: {}", e))
                        }
                    }
                } else {
                    Err(anyhow!("Transaction failed and fallback unavailable: {}", e))
                }
            }
        }
    }

    /// Get recent blockhash
    pub async fn get_recent_blockhash(&self) -> Result<solana_sdk::hash::Hash> {
        let client = self.get_best_client().await;
        
        match client.get_latest_blockhash() {
            Ok(blockhash) => Ok(blockhash),
            Err(e) => Err(anyhow!("Failed to get recent blockhash: {}", e))
        }
    }

    /// Get RPC health status
    pub async fn get_health_status(&self) -> (bool, bool) {
        let primary = *self.primary_healthy.read().await;
        let fallback = *self.fallback_healthy.read().await;
        (primary, fallback)
    }
}

// Implement Clone for RpcManager
impl Clone for RpcManager {
    fn clone(&self) -> Self {
        Self {
            primary_client: Arc::clone(&self.primary_client),
            fallback_client: Arc::clone(&self.fallback_client),
            primary_endpoint: self.primary_endpoint.clone(),
            fallback_endpoint: self.fallback_endpoint.clone(),
            primary_healthy: Arc::clone(&self.primary_healthy),
            fallback_healthy: Arc::clone(&self.fallback_healthy),
            last_health_check: Arc::clone(&self.last_health_check),
            health_check_interval: self.health_check_interval,
        }
    }
}
