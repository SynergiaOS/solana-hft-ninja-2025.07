//! Cerebro Integration Module
//! 
//! Bridge between Rust HFT engine and Python Cerebro AI system

pub mod webhook_client;

pub use webhook_client::{
    WebhookClient, WebhookConfig, OpportunityEvent, ExecutionEvent, 
    RiskEvent, WalletEvent, WebhookStats
};

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// Cerebro integration manager
pub struct CerebroIntegration {
    webhook_client: Arc<WebhookClient>,
    config: WebhookConfig,
    is_enabled: bool,
}

impl CerebroIntegration {
    /// Create new Cerebro integration
    pub fn new(config: WebhookConfig) -> Self {
        let webhook_client = Arc::new(WebhookClient::new(config.clone()));
        
        Self {
            webhook_client,
            config,
            is_enabled: true,
        }
    }

    /// Initialize Cerebro integration
    pub async fn initialize(&self) -> Result<()> {
        if !self.is_enabled {
            warn!("🧠 Cerebro integration is disabled");
            return Ok(());
        }

        info!("🧠 Initializing Cerebro integration...");

        // Test webhook connection
        match self.webhook_client.test_connection().await {
            Ok(_) => {
                info!("✅ Cerebro webhook connection established");
            }
            Err(e) => {
                warn!("⚠️ Cerebro webhook connection failed: {}", e);
                // Don't fail initialization, just log warning
            }
        }

        info!("🧠 Cerebro integration initialized");
        Ok(())
    }

    /// Send MEV opportunity to Cerebro
    pub async fn notify_opportunity(
        &self,
        token_address: &str,
        opportunity_type: &str,
        confidence: f64,
        profit_potential: f64,
        risk_score: f64,
        trigger_wallet: Option<&str>,
        dex_involved: &str,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        if !self.is_enabled {
            return Ok(());
        }

        let event = self.webhook_client.create_opportunity_event(
            token_address,
            opportunity_type,
            confidence,
            profit_potential,
            risk_score,
            trigger_wallet,
            dex_involved,
            metadata,
        );

        self.webhook_client.send_opportunity(event).await
    }

    /// Send execution result to Cerebro
    pub async fn notify_execution(
        &self,
        transaction_id: &str,
        strategy: &str,
        token_address: &str,
        outcome: &str,
        pnl_sol: f64,
        execution_time_ms: u64,
        gas_used: u64,
        trigger_wallet: Option<&str>,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        if !self.is_enabled {
            return Ok(());
        }

        let event = self.webhook_client.create_execution_event(
            transaction_id,
            strategy,
            token_address,
            outcome,
            pnl_sol,
            execution_time_ms,
            gas_used,
            trigger_wallet,
            metadata,
        );

        self.webhook_client.send_execution(event).await
    }

    /// Send risk event to Cerebro
    pub async fn notify_risk(
        &self,
        risk_type: &str,
        severity: &str,
        description: &str,
        affected_strategies: Vec<String>,
        action_taken: &str,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        if !self.is_enabled {
            return Ok(());
        }

        let event = self.webhook_client.create_risk_event(
            risk_type,
            severity,
            description,
            affected_strategies,
            action_taken,
            metadata,
        );

        self.webhook_client.send_risk(event).await
    }

    /// Send wallet event to Cerebro
    pub async fn notify_wallet(
        &self,
        wallet_address: &str,
        event_subtype: &str,
        token_address: Option<&str>,
        amount_sol: Option<f64>,
        confidence: f64,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        if !self.is_enabled {
            return Ok(());
        }

        let event = self.webhook_client.create_wallet_event(
            wallet_address,
            event_subtype,
            token_address,
            amount_sol,
            confidence,
            metadata,
        );

        self.webhook_client.send_wallet(event).await
    }

    /// Get webhook statistics
    pub async fn get_stats(&self) -> WebhookStats {
        self.webhook_client.get_stats().await
    }

    /// Enable/disable Cerebro integration
    pub fn set_enabled(&mut self, enabled: bool) {
        self.is_enabled = enabled;
        if enabled {
            info!("🧠 Cerebro integration enabled");
        } else {
            warn!("🧠 Cerebro integration disabled");
        }
    }

    /// Check if Cerebro integration is healthy
    pub async fn health_check(&self) -> bool {
        if !self.is_enabled {
            return false;
        }

        match self.webhook_client.test_connection().await {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

/// Helper functions for creating metadata
pub mod metadata {
    use std::collections::HashMap;
    use serde_json::Value;

    /// Create metadata for MEV opportunity
    pub fn mev_opportunity(
        pool_address: Option<&str>,
        liquidity_sol: Option<f64>,
        volume_24h: Option<f64>,
        holder_count: Option<u32>,
    ) -> HashMap<String, Value> {
        let mut metadata = HashMap::new();
        
        if let Some(pool) = pool_address {
            metadata.insert("pool_address".to_string(), Value::String(pool.to_string()));
        }
        if let Some(liquidity) = liquidity_sol {
            metadata.insert("liquidity_sol".to_string(), Value::Number(serde_json::Number::from_f64(liquidity).unwrap()));
        }
        if let Some(volume) = volume_24h {
            metadata.insert("volume_24h".to_string(), Value::Number(serde_json::Number::from_f64(volume).unwrap()));
        }
        if let Some(holders) = holder_count {
            metadata.insert("holder_count".to_string(), Value::Number(serde_json::Number::from(holders)));
        }
        
        metadata
    }

    /// Create metadata for trade execution
    pub fn trade_execution(
        slippage: Option<f64>,
        price_impact: Option<f64>,
        route: Option<&str>,
        bundle_position: Option<u32>,
    ) -> HashMap<String, Value> {
        let mut metadata = HashMap::new();
        
        if let Some(slip) = slippage {
            metadata.insert("slippage".to_string(), Value::Number(serde_json::Number::from_f64(slip).unwrap()));
        }
        if let Some(impact) = price_impact {
            metadata.insert("price_impact".to_string(), Value::Number(serde_json::Number::from_f64(impact).unwrap()));
        }
        if let Some(r) = route {
            metadata.insert("route".to_string(), Value::String(r.to_string()));
        }
        if let Some(pos) = bundle_position {
            metadata.insert("bundle_position".to_string(), Value::Number(serde_json::Number::from(pos)));
        }
        
        metadata
    }

    /// Create metadata for risk event
    pub fn risk_event(
        current_balance: Option<f64>,
        position_count: Option<u32>,
        daily_pnl: Option<f64>,
        max_drawdown: Option<f64>,
    ) -> HashMap<String, Value> {
        let mut metadata = HashMap::new();
        
        if let Some(balance) = current_balance {
            metadata.insert("current_balance".to_string(), Value::Number(serde_json::Number::from_f64(balance).unwrap()));
        }
        if let Some(positions) = position_count {
            metadata.insert("position_count".to_string(), Value::Number(serde_json::Number::from(positions)));
        }
        if let Some(pnl) = daily_pnl {
            metadata.insert("daily_pnl".to_string(), Value::Number(serde_json::Number::from_f64(pnl).unwrap()));
        }
        if let Some(drawdown) = max_drawdown {
            metadata.insert("max_drawdown".to_string(), Value::Number(serde_json::Number::from_f64(drawdown).unwrap()));
        }
        
        metadata
    }

    /// Create metadata for wallet event
    pub fn wallet_event(
        transaction_count: Option<u32>,
        success_rate: Option<f64>,
        average_profit: Option<f64>,
        risk_score: Option<f64>,
    ) -> HashMap<String, Value> {
        let mut metadata = HashMap::new();
        
        if let Some(tx_count) = transaction_count {
            metadata.insert("transaction_count".to_string(), Value::Number(serde_json::Number::from(tx_count)));
        }
        if let Some(success) = success_rate {
            metadata.insert("success_rate".to_string(), Value::Number(serde_json::Number::from_f64(success).unwrap()));
        }
        if let Some(profit) = average_profit {
            metadata.insert("average_profit".to_string(), Value::Number(serde_json::Number::from_f64(profit).unwrap()));
        }
        if let Some(risk) = risk_score {
            metadata.insert("risk_score".to_string(), Value::Number(serde_json::Number::from_f64(risk).unwrap()));
        }
        
        metadata
    }
}
