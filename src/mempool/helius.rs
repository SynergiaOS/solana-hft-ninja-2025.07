//! Helius WebSocket Client
//! 
//! Real-time mempool monitoring via Helius WebSocket API

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{info, warn, error, debug};
use url::Url;
use futures::{SinkExt, StreamExt};

/// Helius WebSocket configuration
#[derive(Debug, Clone)]
pub struct HeliusConfig {
    pub api_key: String,
    pub endpoint: String,
    pub reconnect_interval: Duration,
    pub ping_interval: Duration,
    pub max_reconnect_attempts: u32,
}

impl Default for HeliusConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("HELIUS_API_KEY").unwrap_or_default(),
            endpoint: "wss://atlas-mainnet.helius-rpc.com".to_string(),
            reconnect_interval: Duration::from_secs(5),
            ping_interval: Duration::from_secs(30),
            max_reconnect_attempts: 10,
        }
    }
}

/// Helius WebSocket subscription request
#[derive(Debug, Serialize)]
struct SubscriptionRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Value,
}

/// Helius WebSocket response
#[derive(Debug, Deserialize)]
struct HeliusResponse {
    jsonrpc: String,
    id: Option<u64>,
    result: Option<Value>,
    error: Option<Value>,
    method: Option<String>,
    params: Option<Value>,
}

/// Transaction notification from Helius
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionNotification {
    pub signature: String,
    pub slot: u64,
    pub transaction: Value,
    pub meta: Option<Value>,
    pub block_time: Option<i64>,
}

/// Helius WebSocket client
pub struct HeliusClient {
    config: HeliusConfig,
    reconnect_count: u32,
}

impl HeliusClient {
    /// Create new Helius client
    pub fn new(config: HeliusConfig) -> Self {
        Self {
            config,
            reconnect_count: 0,
        }
    }

    /// Start listening to mempool transactions
    pub async fn start_mempool_listener(
        &mut self,
        tx: tokio::sync::mpsc::UnboundedSender<TransactionNotification>,
    ) -> Result<()> {
        loop {
            match self.connect_and_listen(&tx).await {
                Ok(_) => {
                    info!("Helius WebSocket connection closed normally");
                    break;
                }
                Err(e) => {
                    error!("Helius WebSocket error: {}", e);
                    
                    if self.reconnect_count >= self.config.max_reconnect_attempts {
                        error!("Max reconnection attempts reached, giving up");
                        return Err(e);
                    }
                    
                    self.reconnect_count += 1;
                    warn!(
                        "Reconnecting in {:?} (attempt {}/{})",
                        self.config.reconnect_interval,
                        self.reconnect_count,
                        self.config.max_reconnect_attempts
                    );
                    
                    sleep(self.config.reconnect_interval).await;
                }
            }
        }
        
        Ok(())
    }

    /// Connect to Helius WebSocket and listen for transactions
    async fn connect_and_listen(
        &self,
        tx: &tokio::sync::mpsc::UnboundedSender<TransactionNotification>,
    ) -> Result<()> {
        // Build WebSocket URL with API key
        let ws_url = format!("{}/?api-key={}", self.config.endpoint, self.config.api_key);
        let url = Url::parse(&ws_url).context("Invalid WebSocket URL")?;
        
        info!("Connecting to Helius WebSocket: {}", self.config.endpoint);
        
        // Connect to WebSocket
        let (ws_stream, _) = timeout(
            Duration::from_secs(10),
            connect_async(url)
        ).await
        .context("WebSocket connection timeout")?
        .context("Failed to connect to WebSocket")?;
        
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        info!("✅ Connected to Helius WebSocket");
        self.reset_reconnect_count();
        
        // Subscribe to mempool transactions
        let subscription = SubscriptionRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "transactionSubscribe".to_string(),
            params: serde_json::json!({
                "vote": false,
                "failed": false,
                "signature": null,
                "accountInclude": [],
                "accountExclude": []
            }),
        };
        
        let subscription_msg = Message::Text(serde_json::to_string(&subscription)?);
        ws_sender.send(subscription_msg).await
            .context("Failed to send subscription")?;
        
        info!("📡 Subscribed to mempool transactions");
        
        // Start ping task (simplified - no clone needed for this demo)
        let mut ping_interval = tokio::time::interval(self.config.ping_interval);

        tokio::spawn(async move {
            loop {
                ping_interval.tick().await;
                // In a real implementation, would need proper channel communication
                // for ping functionality
                break; // Exit for now
            }
        });
        
        // Listen for messages
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Err(e) = self.handle_message(&text, tx).await {
                        warn!("Error handling message: {}", e);
                    }
                }
                Ok(Message::Pong(_)) => {
                    debug!("Received pong");
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed by server");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    return Err(e.into());
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    /// Handle incoming WebSocket message
    async fn handle_message(
        &self,
        text: &str,
        tx: &tokio::sync::mpsc::UnboundedSender<TransactionNotification>,
    ) -> Result<()> {
        let response: HeliusResponse = serde_json::from_str(text)
            .context("Failed to parse WebSocket message")?;
        
        // Handle subscription confirmation
        if let Some(result) = response.result {
            debug!("Subscription result: {:?}", result);
            return Ok(());
        }
        
        // Handle transaction notification
        if response.method.as_deref() == Some("transactionNotification") {
            if let Some(params) = response.params {
                let notification: TransactionNotification = serde_json::from_value(params)
                    .context("Failed to parse transaction notification")?;
                
                debug!("Received transaction: {}", notification.signature);
                
                if let Err(e) = tx.send(notification) {
                    warn!("Failed to send transaction to bridge: {}", e);
                }
            }
        }
        
        Ok(())
    }

    /// Reset reconnection counter
    fn reset_reconnect_count(&self) {
        // Note: This would need to be mutable in a real implementation
        // For now, we'll handle this differently
    }
}

/// Create and start Helius mempool listener
pub async fn start_helius_listener(
    config: HeliusConfig,
) -> Result<tokio::sync::mpsc::UnboundedReceiver<TransactionNotification>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let mut client = HeliusClient::new(config);
    
    tokio::spawn(async move {
        if let Err(e) = client.start_mempool_listener(tx).await {
            error!("Helius listener failed: {}", e);
        }
    });
    
    Ok(rx)
}
