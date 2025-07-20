use anyhow::{Result, anyhow};
use redis::{aio::Connection, AsyncCommands, Client};
use serde_json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn, error};
use futures::StreamExt;
use crate::cerberus::{PositionState, PositionStatus, CerberusExecutor};

/// Redis/DragonflyDB store for position management
pub struct CerberusStore {
    connection: Arc<Mutex<Connection>>,
    redis_url: String,
}

impl CerberusStore {
    /// Create new store connection
    pub async fn new(redis_url: &str) -> Result<Self> {
        info!("🗄️ Connecting to Redis/DragonflyDB: {}", redis_url);
        
        let client = Client::open(redis_url)?;
        let connection = client.get_async_connection().await?;
        
        info!("✅ Connected to Redis/DragonflyDB");
        
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            redis_url: redis_url.to_string(),
        })
    }

    /// Store a position
    pub async fn store_position(&self, position: &PositionState) -> Result<()> {
        let mut conn = self.connection.lock().await;
        let key = position.redis_key();
        let value = position.to_json()?;
        
        conn.set(&key, &value).await?;
        
        // Also add to active positions set
        if position.status == PositionStatus::Open {
            conn.sadd("active_positions", &position.mint).await?;
        }
        
        debug!("💾 Stored position: {}", position.mint);
        Ok(())
    }

    /// Get a position by mint
    pub async fn get_position(&self, mint: &str) -> Result<Option<PositionState>> {
        let mut conn = self.connection.lock().await;
        let key = format!("position:{}", mint);
        
        let value: Option<String> = conn.get(&key).await?;
        
        match value {
            Some(json) => {
                let position = PositionState::from_json(&json)?;
                Ok(Some(position))
            },
            None => Ok(None),
        }
    }

    /// Update existing position
    pub async fn update_position(&self, position: &PositionState) -> Result<()> {
        self.store_position(position).await
    }

    /// Get all open positions
    pub async fn get_all_open_positions(&self) -> Result<Vec<PositionState>> {
        let mut conn = self.connection.lock().await;
        
        // Get all active position mints
        let mints: Vec<String> = conn.smembers("active_positions").await?;
        
        let mut positions = Vec::new();
        
        for mint in mints {
            let key = format!("position:{}", mint);
            if let Ok(Some(json)) = conn.get::<_, Option<String>>(&key).await {
                if let Ok(position) = PositionState::from_json(&json) {
                    if position.status == PositionStatus::Open {
                        positions.push(position);
                    } else {
                        // Remove from active set if not open
                        let _: () = conn.srem("active_positions", &mint).await.unwrap_or(());
                    }
                }
            }
        }
        
        debug!("📊 Retrieved {} open positions", positions.len());
        Ok(positions)
    }

    /// Close a position
    pub async fn close_position(&self, mint: &str, reason: &str) -> Result<()> {
        let mut conn = self.connection.lock().await;
        
        // Get existing position
        if let Some(mut position) = self.get_position(mint).await? {
            position.status = PositionStatus::Closed;
            
            // Store updated position
            let key = position.redis_key();
            let value = position.to_json()?;
            conn.set(&key, &value).await?;
            
            // Remove from active positions
            conn.srem("active_positions", mint).await?;
            
            // Store close reason and timestamp
            let close_key = format!("position_close:{}:{}", mint, chrono::Utc::now().timestamp());
            conn.set(&close_key, reason).await?;
            conn.expire(&close_key, 86400 * 7).await?; // Keep for 7 days
            
            info!("🔒 Closed position {} - Reason: {}", mint, reason);
        }
        
        Ok(())
    }

    /// Get position count
    pub async fn get_position_count(&self) -> Result<usize> {
        let mut conn = self.connection.lock().await;
        let count: usize = conn.scard("active_positions").await?;
        Ok(count)
    }

    /// Listen for external commands (Guardian alerts, Cerebro signals)
    pub async fn listen_commands(&self, executor: Arc<CerberusExecutor>) -> Result<()> {
        info!("👂 Starting command listener for guardian_alerts and cerebro_commands");
        
        let client = Client::open(self.redis_url.as_str())?;
        let mut pubsub = client.get_async_connection().await?.into_pubsub();
        
        // Subscribe to channels
        pubsub.subscribe("guardian_alerts").await?;
        pubsub.subscribe("cerebro_commands").await?;
        
        info!("✅ Subscribed to Redis channels");
        
        loop {
            match pubsub.on_message().next().await {
                Some(msg) => {
                    let channel = msg.get_channel_name();
                    let payload: String = msg.get_payload().unwrap_or_default();
                    
                    debug!("📨 Received message on {}: {}", channel, payload);
                    
                    if let Err(e) = self.handle_external_command(channel, &payload, &executor).await {
                        error!("Failed to handle command: {}", e);
                    }
                },
                None => {
                    warn!("Redis pubsub connection lost, reconnecting...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    break;
                }
            }
        }
        
        // Restart listener with a small delay
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        Box::pin(self.listen_commands(executor)).await
    }

    /// Handle external commands
    async fn handle_external_command(
        &self,
        channel: &str,
        payload: &str,
        executor: &CerberusExecutor,
    ) -> Result<()> {
        
        let command: serde_json::Value = serde_json::from_str(payload)
            .map_err(|e| anyhow!("Invalid JSON in command: {}", e))?;
        
        match channel {
            "guardian_alerts" => {
                self.handle_guardian_alert(&command, executor).await?;
            },
            "cerebro_commands" => {
                self.handle_cerebro_command(&command, executor).await?;
            },
            _ => {
                warn!("Unknown channel: {}", channel);
            }
        }
        
        Ok(())
    }

    /// Handle Guardian alerts (emergency situations)
    async fn handle_guardian_alert(
        &self,
        command: &serde_json::Value,
        executor: &CerberusExecutor,
    ) -> Result<()> {
        
        let action = command["action"].as_str().unwrap_or("");
        
        match action {
            "EXIT_ALL_POSITIONS" => {
                warn!("🚨 Guardian alert: EXIT_ALL_POSITIONS");
                let reason = command["reason"].as_str().unwrap_or("GUARDIAN_ALERT");
                
                let positions = self.get_all_open_positions().await?;
                for position in positions {
                    if let Err(e) = executor.execute_sell(&position, reason).await {
                        error!("Failed to emergency sell {}: {}", position.mint, e);
                    } else {
                        self.close_position(&position.mint, reason).await?;
                    }
                }
                
                info!("✅ Guardian emergency exit completed");
            },
            "PAUSE_TRADING" => {
                warn!("⏸️ Guardian alert: PAUSE_TRADING");
                // Set a global pause flag
                let mut conn = self.connection.lock().await;
                conn.set("trading_paused", "true").await?;
                conn.expire("trading_paused", 3600).await?; // Auto-resume after 1 hour
            },
            "RESUME_TRADING" => {
                info!("▶️ Guardian alert: RESUME_TRADING");
                let mut conn = self.connection.lock().await;
                conn.del("trading_paused").await?;
            },
            _ => {
                warn!("Unknown guardian action: {}", action);
            }
        }
        
        Ok(())
    }

    /// Handle Cerebro AI commands
    async fn handle_cerebro_command(
        &self,
        command: &serde_json::Value,
        executor: &CerberusExecutor,
    ) -> Result<()> {
        
        let action = command["action"].as_str().unwrap_or("");
        let mint = command["mint"].as_str().unwrap_or("");
        
        if mint.is_empty() {
            return Err(anyhow!("Missing mint in Cerebro command"));
        }
        
        match action {
            "SELL" => {
                let reason = command["reason"].as_str().unwrap_or("AI_SIGNAL");
                info!("🤖 Cerebro SELL signal for {}: {}", mint, reason);
                
                if let Some(position) = self.get_position(mint).await? {
                    if position.status == PositionStatus::Open {
                        executor.execute_sell(&position, reason).await?;
                        self.close_position(mint, reason).await?;
                    }
                }
            },
            "BUY_MORE" => {
                let amount_sol = command["amount_sol"].as_f64().unwrap_or(0.0);
                let reason = command["reason"].as_str().unwrap_or("AI_SCALE_IN");
                
                info!("🤖 Cerebro BUY_MORE signal for {}: {} SOL ({})", mint, amount_sol, reason);
                
                if let Some(mut position) = self.get_position(mint).await? {
                    if position.status == PositionStatus::Open && amount_sol > 0.0 {
                        executor.execute_buy_more(&position, amount_sol).await?;
                        
                        // Update position size
                        position.position_size_sol += amount_sol;
                        self.update_position(&position).await?;
                    }
                }
            },
            "UPDATE_TARGETS" => {
                let take_profit = command["take_profit_percent"].as_f64();
                let stop_loss = command["stop_loss_percent"].as_f64();
                
                info!("🤖 Cerebro UPDATE_TARGETS for {}", mint);
                
                if let Some(mut position) = self.get_position(mint).await? {
                    if let Some(tp) = take_profit {
                        position.take_profit_target_percent = tp;
                    }
                    if let Some(sl) = stop_loss {
                        position.stop_loss_target_percent = sl;
                    }
                    self.update_position(&position).await?;
                }
            },
            _ => {
                warn!("Unknown Cerebro action: {}", action);
            }
        }
        
        Ok(())
    }

    /// Check if trading is paused
    pub async fn is_trading_paused(&self) -> Result<bool> {
        let mut conn = self.connection.lock().await;
        let paused: Option<String> = conn.get("trading_paused").await?;
        Ok(paused.is_some())
    }

    /// Get position statistics
    pub async fn get_position_stats(&self) -> Result<serde_json::Value> {
        let positions = self.get_all_open_positions().await?;
        
        let total_positions = positions.len();
        let total_value_sol: f64 = positions.iter().map(|p| p.position_size_sol).sum();
        
        let profitable_positions = positions.iter()
            .filter(|p| p.pnl_unrealized_percent.unwrap_or(0.0) > 0.0)
            .count();
        
        Ok(serde_json::json!({
            "total_positions": total_positions,
            "total_value_sol": total_value_sol,
            "profitable_positions": profitable_positions,
            "loss_positions": total_positions - profitable_positions,
            "timestamp": chrono::Utc::now().timestamp()
        }))
    }
}
