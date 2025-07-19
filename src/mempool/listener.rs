//! Mempool listener using Helius WebSocket API

use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
use tracing::{error, info, warn};

use crate::mempool::{error::*, metrics::*, parser::*};

/// Helius WebSocket configuration
#[derive(Debug, Clone)]
pub struct HeliusConfig {
    pub api_key: String,
    pub endpoint: String,
    pub commitment: CommitmentLevel,
    pub max_reconnect_attempts: u32,
    pub reconnect_delay_ms: u64,
}

impl Default for HeliusConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("HELIUS_KEY").unwrap_or_default(),
            endpoint: "https://api.helius.xyz".to_string(),
            commitment: CommitmentLevel::Processed,
            max_reconnect_attempts: 10,
            reconnect_delay_ms: 1000,
        }
    }
}

/// Commitment levels for Helius API
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CommitmentLevel {
    Processed,
    Confirmed,
    Finalized,
}

impl std::fmt::Display for CommitmentLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommitmentLevel::Processed => write!(f, "processed"),
            CommitmentLevel::Confirmed => write!(f, "confirmed"),
            CommitmentLevel::Finalized => write!(f, "finalized"),
        }
    }
}

/// Helius WebSocket subscription message
#[derive(Debug, Serialize, Deserialize)]
struct SubscriptionMessage {
    jsonrpc: String,
    id: u64,
    method: String,
    params: SubscriptionParams,
}

#[derive(Debug, Serialize, Deserialize)]
struct SubscriptionParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    commitment: Option<CommitmentLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    encoding: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transaction_details: Option<String>,
}

/// Helius transaction notification
#[derive(Debug, Serialize, Deserialize)]
struct TransactionNotification {
    jsonrpc: String,
    method: String,
    params: TransactionParams,
}

#[derive(Debug, Serialize, Deserialize)]
struct TransactionParams {
    result: TransactionResult,
    subscription: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct TransactionResult {
    slot: u64,
    transaction: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<serde_json::Value>,
    block_time: Option<u64>,
}

/// Mempool listener for real-time transaction monitoring
pub struct MempoolListener {
    config: HeliusConfig,
    parser: ZeroCopyParser,
    metrics: MempoolMetrics,
    is_running: Arc<RwLock<bool>>,
    tx_sender: mpsc::UnboundedSender<ParsedTransaction>,
}

impl MempoolListener {
    pub fn new(
        config: HeliusConfig,
        parser: ZeroCopyParser,
        metrics: MempoolMetrics,
        tx_sender: mpsc::UnboundedSender<ParsedTransaction>,
    ) -> Self {
        Self {
            config,
            parser,
            metrics,
            is_running: Arc::new(RwLock::new(false)),
            tx_sender,
        }
    }

    /// Start the mempool listener
    pub async fn start(&self) -> Result<()> {
        info!("Starting mempool listener...");

        *self.is_running.write().await = true;

        let mut reconnect_attempts = 0;

        while *self.is_running.read().await {
            if reconnect_attempts >= self.config.max_reconnect_attempts {
                error!("Max reconnect attempts reached, stopping listener");
                break;
            }

            match self.connect_and_listen().await {
                Ok(_) => {
                    reconnect_attempts = 0;
                    info!("WebSocket connection closed gracefully");
                }
                Err(e) => {
                    reconnect_attempts += 1;
                    self.metrics.increment_connection_failures();
                    error!(
                        "WebSocket connection error: {}, reconnecting in {}ms (attempt {}/{})",
                        e,
                        self.config.reconnect_delay_ms,
                        reconnect_attempts,
                        self.config.max_reconnect_attempts
                    );

                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        self.config.reconnect_delay_ms,
                    ))
                    .await;
                }
            }
        }

        *self.is_running.write().await = false;
        info!("Mempool listener stopped");

        Ok(())
    }

    /// Stop the mempool listener
    pub async fn stop(&self) {
        info!("Stopping mempool listener...");
        *self.is_running.write().await = false;
    }

    /// Connect to Helius WebSocket and listen for transactions
    async fn connect_and_listen(&self) -> Result<()> {
        let ws_url = format!(
            "wss://mainnet.helius-rpc.com/?api-key={}",
            self.config.api_key
        );

        info!("Connecting to Helius WebSocket: {}", ws_url);
        self.metrics.increment_connection_attempts();

        let (ws_stream, _) = connect_async(ws_url).await.map_err(MempoolError::from)?;

        info!("Connected to Helius WebSocket");

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Subscribe to transaction notifications
        let subscription = SubscriptionMessage {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "transactionSubscribe".to_string(),
            params: SubscriptionParams {
                commitment: Some(self.config.commitment),
                encoding: Some("base64".to_string()),
                transaction_details: Some("full".to_string()),
            },
        };

        let subscription_msg = serde_json::to_string(&subscription)?;
        ws_sender
            .send(WsMessage::Text(subscription_msg))
            .await
            .map_err(MempoolError::from)?;

        info!("Subscribed to transaction notifications");

        // Listen for messages
        while *self.is_running.read().await {
            match ws_receiver.next().await {
                Some(Ok(msg)) => {
                    if let Err(e) = self.handle_message(msg).await {
                        error!("Error handling message: {}", e);
                    }
                }
                Some(Err(e)) => {
                    return Err(MempoolError::from(e));
                }
                None => {
                    info!("WebSocket connection closed");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle incoming WebSocket message
    async fn handle_message(&self, msg: WsMessage) -> Result<()> {
        match msg {
            WsMessage::Text(text) => {
                if let Ok(notification) = serde_json::from_str::<TransactionNotification>(&text) {
                    self.process_transaction_notification(notification).await?;
                }
            }
            WsMessage::Binary(data) => {
                // Handle binary data if needed
                if let Ok(text) = String::from_utf8(data) {
                    if let Ok(notification) = serde_json::from_str::<TransactionNotification>(&text)
                    {
                        self.process_transaction_notification(notification).await?;
                    }
                }
            }
            WsMessage::Ping(_) | WsMessage::Pong(_) => {
                // Handle ping/pong for connection health
            }
            WsMessage::Close(_) => {
                info!("Received close message from server");
            }
            WsMessage::Frame(_) => {
                // Raw frame, usually handled by the library
            }
        }

        Ok(())
    }

    /// Process transaction notification
    async fn process_transaction_notification(
        &self,
        notification: TransactionNotification,
    ) -> Result<()> {
        let slot = notification.params.result.slot;
        let timestamp = notification.params.result.block_time.unwrap_or(0);
        let transaction_data = notification.params.result.transaction;

        // Parse transaction using zero-copy deserialization
        match self
            .parser
            .parse_transaction(&transaction_data, timestamp, slot)
        {
            Ok(parsed_tx) => {
                // Send parsed transaction to processing channel
                if let Err(e) = self.tx_sender.send(parsed_tx) {
                    error!("Failed to send parsed transaction: {}", e);
                }
            }
            Err(e) => {
                self.metrics.increment_deserialization_errors();
                warn!("Failed to parse transaction: {}", e);
            }
        }

        Ok(())
    }

    /// Check if listener is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Get current metrics
    pub fn metrics(&self) -> &MempoolMetrics {
        &self.metrics
    }
}

/// Builder for MempoolListener
pub struct MempoolListenerBuilder {
    config: HeliusConfig,
    parser: Option<ZeroCopyParser>,
    metrics: Option<MempoolMetrics>,
    tx_sender: Option<mpsc::UnboundedSender<ParsedTransaction>>,
}

impl MempoolListenerBuilder {
    pub fn new() -> Self {
        Self {
            config: HeliusConfig::default(),
            parser: None,
            metrics: None,
            tx_sender: None,
        }
    }

    pub fn with_config(mut self, config: HeliusConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_parser(mut self, parser: ZeroCopyParser) -> Self {
        self.parser = Some(parser);
        self
    }

    pub fn with_metrics(mut self, metrics: MempoolMetrics) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn with_sender(mut self, sender: mpsc::UnboundedSender<ParsedTransaction>) -> Self {
        self.tx_sender = Some(sender);
        self
    }

    pub fn build(self) -> Result<MempoolListener> {
        let parser = self.parser.unwrap_or_else(|| {
            ZeroCopyParser::new(
                self.metrics.clone().unwrap_or_else(MempoolMetrics::new),
                16 * 1024 * 1024, // 16MB default
            )
        });

        let metrics = self.metrics.unwrap_or_else(MempoolMetrics::new);
        let tx_sender = self.tx_sender.ok_or_else(|| {
            MempoolError::Config("Transaction sender channel required".to_string())
        })?;

        Ok(MempoolListener::new(
            self.config,
            parser,
            metrics,
            tx_sender,
        ))
    }
}

impl Default for MempoolListenerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_listener_builder() {
        let (tx, _rx) = mpsc::unbounded_channel();

        let listener = MempoolListenerBuilder::new()
            .with_sender(tx)
            .build()
            .unwrap();

        assert!(!listener.is_running().await);
    }

    #[tokio::test]
    async fn test_config_validation() {
        let mut config = HeliusConfig::default();
        config.api_key = "test-key".to_string();

        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.endpoint, "https://api.helius.xyz");
    }

    #[tokio::test]
    async fn test_subscription_message() {
        let subscription = SubscriptionMessage {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "transactionSubscribe".to_string(),
            params: SubscriptionParams {
                commitment: Some(CommitmentLevel::Processed),
                encoding: Some("base64".to_string()),
                transaction_details: Some("full".to_string()),
            },
        };

        let json = serde_json::to_string(&subscription).unwrap();
        assert!(json.contains("transactionSubscribe"));
        // CommitmentLevel::Processed is serialized as "Processed" (with capital P)
        assert!(json.contains("Processed"));
    }
}
