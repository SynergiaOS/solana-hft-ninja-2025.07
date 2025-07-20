//! 🚀 High-Performance RPC Connection Pool
//!
//! Optimized connection pool with 32 connections and 60s keep-alive

use anyhow::Result;
use reqwest::{Client, ClientBuilder};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

/// RPC connection pool configuration
#[derive(Debug, Clone)]
pub struct RpcPoolConfig {
    pub max_connections: usize,
    pub keep_alive_timeout: Duration,
    pub connection_timeout: Duration,
    pub request_timeout: Duration,
    pub retry_attempts: u32,
}

impl Default for RpcPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 32,
            keep_alive_timeout: Duration::from_secs(60),
            connection_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(10),
            retry_attempts: 3,
        }
    }
}

/// High-performance RPC connection pool
pub struct RpcPool {
    config: RpcPoolConfig,
    client: Client,
    semaphore: Arc<Semaphore>,
}

impl RpcPool {
    /// Create new RPC pool with optimized settings
    pub fn new(config: RpcPoolConfig) -> Result<Self> {
        info!(
            "🚀 Creating RPC pool with {} connections",
            config.max_connections
        );

        let client = ClientBuilder::new()
            .pool_max_idle_per_host(config.max_connections)
            .pool_idle_timeout(config.keep_alive_timeout)
            .connect_timeout(config.connection_timeout)
            .timeout(config.request_timeout)
            .tcp_keepalive(config.keep_alive_timeout)
            .tcp_nodelay(true) // Disable Nagle's algorithm for low latency
            .http2_prior_knowledge() // Use HTTP/2 for better performance
            .build()?;

        let semaphore = Arc::new(Semaphore::new(config.max_connections));

        Ok(Self {
            config,
            client,
            semaphore,
        })
    }

    /// Execute RPC request with connection pooling
    pub async fn execute_request<T>(&self, url: &str, body: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let _permit = self.semaphore.acquire().await?;

        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.config.retry_attempts {
            match self.try_request(url, body).await {
                Ok(response) => {
                    debug!("RPC request successful on attempt {}", attempts + 1);
                    return Ok(response);
                }
                Err(e) => {
                    attempts += 1;
                    last_error = Some(e);

                    if attempts < self.config.retry_attempts {
                        warn!(
                            "RPC request failed, retrying... (attempt {}/{})",
                            attempts, self.config.retry_attempts
                        );
                        tokio::time::sleep(Duration::from_millis(100 * attempts as u64)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
    }

    /// Try single RPC request
    async fn try_request<T>(&self, url: &str, body: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "RPC request failed with status: {}",
                response.status()
            ));
        }

        let text = response.text().await?;
        let result: T = serde_json::from_str(&text)?;

        Ok(result)
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> RpcPoolStats {
        RpcPoolStats {
            max_connections: self.config.max_connections,
            available_connections: self.semaphore.available_permits(),
            keep_alive_timeout: self.config.keep_alive_timeout,
        }
    }
}

/// RPC pool statistics
#[derive(Debug)]
pub struct RpcPoolStats {
    pub max_connections: usize,
    pub available_connections: usize,
    pub keep_alive_timeout: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rpc_pool_creation() {
        let config = RpcPoolConfig::default();
        let pool = RpcPool::new(config).unwrap();

        let stats = pool.get_stats();
        assert_eq!(stats.max_connections, 32);
        assert_eq!(stats.available_connections, 32);
    }

    #[tokio::test]
    async fn test_connection_limiting() {
        let config = RpcPoolConfig {
            max_connections: 2,
            ..Default::default()
        };
        let pool = Arc::new(RpcPool::new(config).unwrap());

        // Simulate concurrent requests
        let handles: Vec<_> = (0..5)
            .map(|_| {
                let pool = pool.clone();
                tokio::spawn(async move {
                    let _permit = pool.semaphore.acquire().await.unwrap();
                    tokio::time::sleep(Duration::from_millis(100)).await;
                })
            })
            .collect();

        // Wait for all to complete
        for handle in handles {
            handle.await.unwrap();
        }

        let stats = pool.get_stats();
        assert_eq!(stats.available_connections, 2);
    }
}
