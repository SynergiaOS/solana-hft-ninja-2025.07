//! Webhook Client for HFT Ninja → Cerebro Communication
//!
//! Sends real-time events from Rust HFT engine to Python Cerebro AI

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Webhook client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub cerebro_bff_url: String,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
    pub batch_size: usize,
    pub flush_interval_ms: u64,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            cerebro_bff_url: "http://localhost:8002".to_string(),
            timeout_seconds: 5,
            retry_attempts: 3,
            retry_delay_ms: 1000,
            batch_size: 10,
            flush_interval_ms: 5000,
        }
    }
}

/// MEV opportunity event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityEvent {
    pub event_type: String,
    pub token_address: String,
    pub opportunity_type: String,
    pub confidence: f64,
    pub profit_potential: f64,
    pub risk_score: f64,
    pub trigger_wallet: Option<String>,
    pub dex_involved: String,
    pub timestamp: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Trade execution result event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEvent {
    pub event_type: String,
    pub transaction_id: String,
    pub strategy: String,
    pub token_address: String,
    pub outcome: String,
    pub pnl_sol: f64,
    pub execution_time_ms: u64,
    pub gas_used: u64,
    pub trigger_wallet: Option<String>,
    pub timestamp: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Risk management event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEvent {
    pub event_type: String,
    pub risk_type: String,
    pub severity: String,
    pub description: String,
    pub affected_strategies: Vec<String>,
    pub action_taken: String,
    pub timestamp: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Wallet tracking event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletEvent {
    pub event_type: String,
    pub wallet_address: String,
    pub event_subtype: String,
    pub token_address: Option<String>,
    pub amount_sol: Option<f64>,
    pub confidence: f64,
    pub timestamp: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Webhook event types
#[derive(Debug, Clone)]
pub enum WebhookEvent {
    Opportunity(OpportunityEvent),
    Execution(ExecutionEvent),
    Risk(RiskEvent),
    Wallet(WalletEvent),
}

/// Webhook client for sending events to Cerebro
pub struct WebhookClient {
    config: WebhookConfig,
    client: Client,
    event_queue: RwLock<Vec<WebhookEvent>>,
    stats: RwLock<WebhookStats>,
}

/// Webhook statistics
#[derive(Debug, Clone, Default)]
pub struct WebhookStats {
    pub events_sent: u64,
    pub events_failed: u64,
    pub total_retries: u64,
    pub average_latency_ms: f64,
    pub last_success: Option<u64>,
    pub last_failure: Option<u64>,
}

impl WebhookClient {
    /// Create new webhook client
    pub fn new(config: WebhookConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            event_queue: RwLock::new(Vec::new()),
            stats: RwLock::new(WebhookStats::default()),
        }
    }

    /// Send MEV opportunity event
    pub async fn send_opportunity(&self, event: OpportunityEvent) -> Result<()> {
        debug!(
            "🎯 Sending opportunity event: {} for {}",
            event.opportunity_type, event.token_address
        );

        let webhook_event = WebhookEvent::Opportunity(event);
        self.send_event(webhook_event).await
    }

    /// Send trade execution event
    pub async fn send_execution(&self, event: ExecutionEvent) -> Result<()> {
        info!(
            "📊 Sending execution event: {} - {} SOL",
            event.outcome, event.pnl_sol
        );

        let webhook_event = WebhookEvent::Execution(event);
        self.send_event(webhook_event).await
    }

    /// Send risk management event
    pub async fn send_risk(&self, event: RiskEvent) -> Result<()> {
        warn!(
            "⚠️ Sending risk event: {} - {}",
            event.risk_type, event.severity
        );

        let webhook_event = WebhookEvent::Risk(event);
        self.send_event(webhook_event).await
    }

    /// Send wallet tracking event
    pub async fn send_wallet(&self, event: WalletEvent) -> Result<()> {
        debug!(
            "👛 Sending wallet event: {} from {}",
            event.event_subtype, event.wallet_address
        );

        let webhook_event = WebhookEvent::Wallet(event);
        self.send_event(webhook_event).await
    }

    /// Send event with retry logic
    async fn send_event(&self, event: WebhookEvent) -> Result<()> {
        let start_time = SystemTime::now();

        for attempt in 1..=self.config.retry_attempts {
            match self.send_event_once(&event).await {
                Ok(_) => {
                    // Update success stats
                    self.update_success_stats(start_time).await;
                    return Ok(());
                }
                Err(e) => {
                    if attempt == self.config.retry_attempts {
                        // Final attempt failed
                        self.update_failure_stats().await;
                        return Err(e);
                    } else {
                        // Retry with delay
                        warn!("🔄 Webhook attempt {} failed, retrying: {}", attempt, e);
                        tokio::time::sleep(std::time::Duration::from_millis(
                            self.config.retry_delay_ms * attempt as u64,
                        ))
                        .await;
                    }
                }
            }
        }

        Err(anyhow::anyhow!("All webhook attempts failed"))
    }

    /// Send single event attempt
    async fn send_event_once(&self, event: &WebhookEvent) -> Result<()> {
        let (endpoint, payload) = match event {
            WebhookEvent::Opportunity(e) => ("/webhook/opportunity", serde_json::to_value(e)?),
            WebhookEvent::Execution(e) => ("/webhook/execution", serde_json::to_value(e)?),
            WebhookEvent::Risk(e) => ("/webhook/risk", serde_json::to_value(e)?),
            WebhookEvent::Wallet(e) => ("/webhook/wallet", serde_json::to_value(e)?),
        };

        let url = format!("{}{}", self.config.cerebro_bff_url, endpoint);

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send webhook request")?;

        if response.status().is_success() {
            debug!("✅ Webhook sent successfully to {}", endpoint);
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!(
                "Webhook failed with status {}: {}",
                status,
                error_text
            ))
        }
    }

    /// Update success statistics
    async fn update_success_stats(&self, start_time: SystemTime) {
        let mut stats = self.stats.write().await;
        stats.events_sent += 1;

        if let Ok(duration) = start_time.elapsed() {
            let latency_ms = duration.as_millis() as f64;
            stats.average_latency_ms = (stats.average_latency_ms * (stats.events_sent - 1) as f64
                + latency_ms)
                / stats.events_sent as f64;
        }

        stats.last_success = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
    }

    /// Update failure statistics
    async fn update_failure_stats(&self) {
        let mut stats = self.stats.write().await;
        stats.events_failed += 1;
        stats.last_failure = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
    }

    /// Get webhook statistics
    pub async fn get_stats(&self) -> WebhookStats {
        self.stats.read().await.clone()
    }

    /// Create opportunity event from MEV data
    pub fn create_opportunity_event(
        &self,
        token_address: &str,
        opportunity_type: &str,
        confidence: f64,
        profit_potential: f64,
        risk_score: f64,
        trigger_wallet: Option<&str>,
        dex_involved: &str,
        metadata: HashMap<String, serde_json::Value>,
    ) -> OpportunityEvent {
        OpportunityEvent {
            event_type: "opportunity_detected".to_string(),
            token_address: token_address.to_string(),
            opportunity_type: opportunity_type.to_string(),
            confidence,
            profit_potential,
            risk_score,
            trigger_wallet: trigger_wallet.map(|s| s.to_string()),
            dex_involved: dex_involved.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            metadata,
        }
    }

    /// Create execution event from trade result
    pub fn create_execution_event(
        &self,
        transaction_id: &str,
        strategy: &str,
        token_address: &str,
        outcome: &str,
        pnl_sol: f64,
        execution_time_ms: u64,
        gas_used: u64,
        trigger_wallet: Option<&str>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> ExecutionEvent {
        ExecutionEvent {
            event_type: "execution_result".to_string(),
            transaction_id: transaction_id.to_string(),
            strategy: strategy.to_string(),
            token_address: token_address.to_string(),
            outcome: outcome.to_string(),
            pnl_sol,
            execution_time_ms,
            gas_used,
            trigger_wallet: trigger_wallet.map(|s| s.to_string()),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            metadata,
        }
    }

    /// Create risk event
    pub fn create_risk_event(
        &self,
        risk_type: &str,
        severity: &str,
        description: &str,
        affected_strategies: Vec<String>,
        action_taken: &str,
        metadata: HashMap<String, serde_json::Value>,
    ) -> RiskEvent {
        RiskEvent {
            event_type: "risk_event".to_string(),
            risk_type: risk_type.to_string(),
            severity: severity.to_string(),
            description: description.to_string(),
            affected_strategies,
            action_taken: action_taken.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            metadata,
        }
    }

    /// Create wallet event
    pub fn create_wallet_event(
        &self,
        wallet_address: &str,
        event_subtype: &str,
        token_address: Option<&str>,
        amount_sol: Option<f64>,
        confidence: f64,
        metadata: HashMap<String, serde_json::Value>,
    ) -> WalletEvent {
        WalletEvent {
            event_type: "wallet_event".to_string(),
            wallet_address: wallet_address.to_string(),
            event_subtype: event_subtype.to_string(),
            token_address: token_address.map(|s| s.to_string()),
            amount_sol,
            confidence,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            metadata,
        }
    }

    /// Test webhook connection
    pub async fn test_connection(&self) -> Result<()> {
        let test_event = self.create_risk_event(
            "connection_test",
            "low",
            "Testing webhook connection",
            vec!["test".to_string()],
            "none",
            HashMap::new(),
        );

        self.send_risk(test_event)
            .await
            .context("Webhook connection test failed")
    }
}
