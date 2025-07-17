// 🛡️ RISK MANAGEMENT SYSTEM
// Production-grade risk controls for mainnet trading

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tokio::time::{interval, Instant};
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimits {
    pub max_position_size_sol: f64,
    pub max_daily_loss_sol: f64,
    pub stop_loss_percentage: f64,
    pub max_slippage_percentage: f64,
    pub min_liquidity_usd: f64,
    pub max_trades_per_minute: u32,
    pub circuit_breaker_enabled: bool,
    pub kill_switch_loss_sol: f64,
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            max_position_size_sol: 1.0,
            max_daily_loss_sol: 0.5,
            stop_loss_percentage: 5.0,
            max_slippage_percentage: 2.0,
            min_liquidity_usd: 10_000.0,
            max_trades_per_minute: 10,
            circuit_breaker_enabled: true,
            kill_switch_loss_sol: 2.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub daily_pnl: f64,
    pub current_positions: HashMap<String, f64>,
    pub trades_last_minute: u32,
    pub max_drawdown: f64,
    pub total_volume_sol: f64,
    pub last_reset: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskEvent {
    PositionLimitExceeded,
    DailyLossLimitExceeded,
    StopLossTriggered,
    SlippageExceeded,
    LiquidityTooLow,
    TradingRateExceeded,
    CircuitBreakerTriggered,
    KillSwitchActivated,
    EmergencyStop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAlert {
    pub event: RiskEvent,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: SystemTime,
    pub action_taken: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

pub struct RiskManager {
    limits: Arc<Mutex<RiskLimits>>,
    metrics: Arc<Mutex<RiskMetrics>>,
    alerts: Arc<Mutex<Vec<RiskAlert>>>,
    trading_enabled: Arc<Mutex<bool>>,
    emergency_stop: Arc<Mutex<bool>>,
}

impl RiskManager {
    pub fn new(limits: RiskLimits) -> Self {
        let metrics = RiskMetrics {
            daily_pnl: 0.0,
            current_positions: HashMap::new(),
            trades_last_minute: 0,
            max_drawdown: 0.0,
            total_volume_sol: 0.0,
            last_reset: SystemTime::now(),
        };

        Self {
            limits: Arc::new(Mutex::new(limits)),
            metrics: Arc::new(Mutex::new(metrics)),
            alerts: Arc::new(Mutex::new(Vec::new())),
            trading_enabled: Arc::new(Mutex::new(false)),
            emergency_stop: Arc::new(Mutex::new(false)),
        }
    }

    /// Check if a trade is allowed based on risk limits
    pub fn check_trade_allowed(
        &self,
        symbol: &str,
        size_sol: f64,
        expected_slippage: f64,
        liquidity_usd: f64,
    ) -> Result<bool> {
        let limits = self.limits.lock().unwrap();
        let mut metrics = self.metrics.lock().unwrap();

        // Check emergency stop
        if *self.emergency_stop.lock().unwrap() {
            return Err(anyhow!("Emergency stop activated"));
        }

        // Check if trading is enabled
        if !*self.trading_enabled.lock().unwrap() {
            return Err(anyhow!("Trading is disabled"));
        }

        // Check position size limit
        if size_sol > limits.max_position_size_sol {
            self.trigger_alert(RiskEvent::PositionLimitExceeded, 
                format!("Position size {} SOL exceeds limit {} SOL", 
                    size_sol, limits.max_position_size_sol),
                AlertSeverity::Warning);
            return Ok(false);
        }

        // Check daily loss limit
        if metrics.daily_pnl < -limits.max_daily_loss_sol {
            self.trigger_alert(RiskEvent::DailyLossLimitExceeded,
                format!("Daily loss {} SOL exceeds limit {} SOL",
                    -metrics.daily_pnl, limits.max_daily_loss_sol),
                AlertSeverity::Critical);
            self.disable_trading("Daily loss limit exceeded");
            return Ok(false);
        }

        // Check slippage limit
        if expected_slippage > limits.max_slippage_percentage {
            self.trigger_alert(RiskEvent::SlippageExceeded,
                format!("Expected slippage {}% exceeds limit {}%",
                    expected_slippage, limits.max_slippage_percentage),
                AlertSeverity::Warning);
            return Ok(false);
        }

        // Check liquidity requirement
        if liquidity_usd < limits.min_liquidity_usd {
            self.trigger_alert(RiskEvent::LiquidityTooLow,
                format!("Liquidity ${} below minimum ${}",
                    liquidity_usd, limits.min_liquidity_usd),
                AlertSeverity::Warning);
            return Ok(false);
        }

        // Check trading rate limit
        if metrics.trades_last_minute >= limits.max_trades_per_minute {
            self.trigger_alert(RiskEvent::TradingRateExceeded,
                format!("Trading rate {} trades/min exceeds limit {}",
                    metrics.trades_last_minute, limits.max_trades_per_minute),
                AlertSeverity::Warning);
            return Ok(false);
        }

        // Check kill switch
        if -metrics.daily_pnl >= limits.kill_switch_loss_sol {
            self.trigger_alert(RiskEvent::KillSwitchActivated,
                format!("Kill switch activated at {} SOL loss",
                    -metrics.daily_pnl),
                AlertSeverity::Emergency);
            self.emergency_shutdown("Kill switch triggered");
            return Ok(false);
        }

        Ok(true)
    }

    /// Record a completed trade
    pub fn record_trade(&self, symbol: &str, pnl: f64, size_sol: f64) -> Result<()> {
        let mut metrics = self.metrics.lock().unwrap();
        
        // Update daily P&L
        metrics.daily_pnl += pnl;
        
        // Update position
        let current_position = metrics.current_positions.get(symbol).unwrap_or(&0.0);
        metrics.current_positions.insert(symbol.to_string(), current_position + size_sol);
        
        // Update volume
        metrics.total_volume_sol += size_sol.abs();
        
        // Update max drawdown
        if metrics.daily_pnl < metrics.max_drawdown {
            metrics.max_drawdown = metrics.daily_pnl;
        }
        
        // Increment trade counter
        metrics.trades_last_minute += 1;

        // Check stop loss
        let limits = self.limits.lock().unwrap();
        if pnl < 0.0 && (-pnl / size_sol * 100.0) > limits.stop_loss_percentage {
            self.trigger_alert(RiskEvent::StopLossTriggered,
                format!("Stop loss triggered: {}% loss on {} SOL position",
                    -pnl / size_sol * 100.0, size_sol),
                AlertSeverity::Critical);
        }

        Ok(())
    }

    /// Enable trading
    pub fn enable_trading(&self, reason: &str) {
        *self.trading_enabled.lock().unwrap() = true;
        self.trigger_alert(RiskEvent::EmergencyStop,
            format!("Trading enabled: {}", reason),
            AlertSeverity::Info);
    }

    /// Disable trading
    pub fn disable_trading(&self, reason: &str) {
        *self.trading_enabled.lock().unwrap() = false;
        self.trigger_alert(RiskEvent::EmergencyStop,
            format!("Trading disabled: {}", reason),
            AlertSeverity::Critical);
    }

    /// Emergency shutdown
    pub fn emergency_shutdown(&self, reason: &str) {
        *self.emergency_stop.lock().unwrap() = true;
        *self.trading_enabled.lock().unwrap() = false;
        
        self.trigger_alert(RiskEvent::EmergencyStop,
            format!("EMERGENCY SHUTDOWN: {}", reason),
            AlertSeverity::Emergency);
    }

    /// Reset daily metrics (called at midnight)
    pub fn reset_daily_metrics(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.daily_pnl = 0.0;
        metrics.current_positions.clear();
        metrics.max_drawdown = 0.0;
        metrics.total_volume_sol = 0.0;
        metrics.last_reset = SystemTime::now();
    }

    /// Reset minute-based counters
    pub fn reset_minute_metrics(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.trades_last_minute = 0;
    }

    /// Trigger risk alert
    fn trigger_alert(&self, event: RiskEvent, message: String, severity: AlertSeverity) {
        let alert = RiskAlert {
            event,
            message: message.clone(),
            severity,
            timestamp: SystemTime::now(),
            action_taken: "Alert logged".to_string(),
        };

        self.alerts.lock().unwrap().push(alert);
        
        // Log to console
        match severity {
            AlertSeverity::Emergency => log::error!("🚨 EMERGENCY: {}", message),
            AlertSeverity::Critical => log::error!("❌ CRITICAL: {}", message),
            AlertSeverity::Warning => log::warn!("⚠️ WARNING: {}", message),
            AlertSeverity::Info => log::info!("ℹ️ INFO: {}", message),
        }
    }

    /// Get current risk status
    pub fn get_risk_status(&self) -> serde_json::Value {
        let limits = self.limits.lock().unwrap();
        let metrics = self.metrics.lock().unwrap();
        let trading_enabled = *self.trading_enabled.lock().unwrap();
        let emergency_stop = *self.emergency_stop.lock().unwrap();

        serde_json::json!({
            "trading_enabled": trading_enabled,
            "emergency_stop": emergency_stop,
            "daily_pnl": metrics.daily_pnl,
            "max_drawdown": metrics.max_drawdown,
            "trades_last_minute": metrics.trades_last_minute,
            "total_volume_sol": metrics.total_volume_sol,
            "limits": {
                "max_position_size_sol": limits.max_position_size_sol,
                "max_daily_loss_sol": limits.max_daily_loss_sol,
                "stop_loss_percentage": limits.stop_loss_percentage,
                "max_slippage_percentage": limits.max_slippage_percentage,
                "min_liquidity_usd": limits.min_liquidity_usd,
                "max_trades_per_minute": limits.max_trades_per_minute,
                "kill_switch_loss_sol": limits.kill_switch_loss_sol
            },
            "current_positions": metrics.current_positions,
            "risk_utilization": {
                "position_utilization": metrics.current_positions.values().sum::<f64>() / limits.max_position_size_sol * 100.0,
                "loss_utilization": (-metrics.daily_pnl) / limits.max_daily_loss_sol * 100.0,
                "trade_rate_utilization": metrics.trades_last_minute as f64 / limits.max_trades_per_minute as f64 * 100.0
            }
        })
    }

    /// Start background risk monitoring
    pub async fn start_monitoring(&self) {
        let risk_manager = self.clone();
        
        // Reset minute counters every minute
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                risk_manager.reset_minute_metrics();
            }
        });

        // Reset daily metrics at midnight
        let risk_manager = self.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(3600)); // Check every hour
            loop {
                interval.tick().await;
                
                let now = SystemTime::now();
                let duration_since_epoch = now.duration_since(UNIX_EPOCH).unwrap();
                let hours = (duration_since_epoch.as_secs() / 3600) % 24;
                
                // Reset at midnight UTC
                if hours == 0 {
                    risk_manager.reset_daily_metrics();
                }
            }
        });
    }
}

impl Clone for RiskManager {
    fn clone(&self) -> Self {
        Self {
            limits: Arc::clone(&self.limits),
            metrics: Arc::clone(&self.metrics),
            alerts: Arc::clone(&self.alerts),
            trading_enabled: Arc::clone(&self.trading_enabled),
            emergency_stop: Arc::clone(&self.emergency_stop),
        }
    }
}
