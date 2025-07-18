//! Risk Limits Module
//! 
//! Position sizing, daily loss limits, and risk management

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

use super::{SecurityConfig, TransactionResult};

/// Risk limits manager
pub struct RiskLimits {
    config: SecurityConfig,
    daily_stats: Arc<RwLock<DailyStats>>,
    position_tracker: Arc<RwLock<PositionTracker>>,
}

/// Daily trading statistics
#[derive(Debug, Clone)]
struct DailyStats {
    pub date: chrono::NaiveDate,
    pub total_pnl_sol: f64,
    pub total_volume_sol: f64,
    pub trade_count: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub max_drawdown_sol: f64,
    pub current_drawdown_sol: f64,
}

/// Position tracking
#[derive(Debug, Clone)]
struct PositionTracker {
    pub current_positions: HashMap<String, Position>,
    pub total_exposure_sol: f64,
    pub max_position_size_sol: f64,
}

/// Individual position
#[derive(Debug, Clone)]
struct Position {
    pub symbol: String,
    pub size_sol: f64,
    pub entry_price: f64,
    pub unrealized_pnl_sol: f64,
    pub timestamp: u64,
}

impl RiskLimits {
    /// Create new risk limits manager
    pub fn new(config: &SecurityConfig) -> Result<Self> {
        info!("📊 Initializing Risk Limits...");
        
        let daily_stats = Arc::new(RwLock::new(DailyStats::new()));
        let position_tracker = Arc::new(RwLock::new(PositionTracker::new(config.max_position_size_sol)));
        
        info!("✅ Risk Limits initialized");
        
        Ok(Self {
            config: config.clone(),
            daily_stats,
            position_tracker,
        })
    }
    
    /// Check if position size is within limits
    pub fn check_position_size(&self, amount_sol: f64) -> Result<bool> {
        if amount_sol > self.config.max_position_size_sol {
            warn!("🚨 Position size {} SOL exceeds limit {} SOL", 
                  amount_sol, self.config.max_position_size_sol);
            return Ok(false);
        }
        
        debug!("✅ Position size {} SOL within limits", amount_sol);
        Ok(true)
    }
    
    /// Check daily loss limit
    pub async fn check_daily_loss_limit(&self) -> Result<bool> {
        let stats = self.daily_stats.read().await;
        
        // Check if we need to reset daily stats (new day)
        let today = chrono::Utc::now().naive_utc().date();
        if stats.date != today {
            drop(stats);
            self.reset_daily_stats().await?;
            return Ok(true);
        }
        
        let daily_loss = -stats.total_pnl_sol.min(0.0);
        
        if daily_loss >= self.config.max_daily_loss_sol {
            warn!("🚨 Daily loss {} SOL exceeds limit {} SOL", 
                  daily_loss, self.config.max_daily_loss_sol);
            return Ok(false);
        }
        
        debug!("✅ Daily loss {} SOL within limit {} SOL", 
               daily_loss, self.config.max_daily_loss_sol);
        Ok(true)
    }
    
    /// Update risk tracking with transaction result
    pub async fn update_with_result(&mut self, result: &TransactionResult) -> Result<()> {
        // Update daily stats
        {
            let mut stats = self.daily_stats.write().await;
            
            // Check if new day
            let today = chrono::Utc::now().naive_utc().date();
            if stats.date != today {
                *stats = DailyStats::new();
            }
            
            stats.total_pnl_sol += result.profit_loss_sol;
            stats.trade_count += 1;
            
            if result.profit_loss_sol > 0.0 {
                stats.winning_trades += 1;
            } else {
                stats.losing_trades += 1;
                
                // Update drawdown
                stats.current_drawdown_sol += result.profit_loss_sol.abs();
                if stats.current_drawdown_sol > stats.max_drawdown_sol {
                    stats.max_drawdown_sol = stats.current_drawdown_sol;
                }
            }
            
            // Reset drawdown on profitable trade
            if result.profit_loss_sol > 0.0 {
                stats.current_drawdown_sol = 0.0;
            }
        }
        
        // Update position tracking
        {
            let mut positions = self.position_tracker.write().await;
            // Simplified position tracking - in real implementation would track actual positions
            positions.total_exposure_sol = positions.current_positions.values()
                .map(|p| p.size_sol)
                .sum();
        }
        
        debug!("📊 Risk tracking updated with P&L: {} SOL", result.profit_loss_sol);
        Ok(())
    }
    
    /// Get daily loss ratio (0.0 to 1.0)
    pub fn get_daily_loss_ratio(&self) -> f64 {
        // Simplified sync version - in async context use proper async version
        0.0 // Placeholder
    }
    
    /// Get position utilization ratio (0.0 to 1.0)
    pub fn get_position_utilization(&self) -> f64 {
        // Simplified sync version - in async context use proper async version
        0.0 // Placeholder
    }
    
    /// Reset daily statistics
    async fn reset_daily_stats(&self) -> Result<()> {
        let mut stats = self.daily_stats.write().await;
        *stats = DailyStats::new();
        info!("📅 Daily statistics reset for new trading day");
        Ok(())
    }
    
    /// Get risk metrics
    pub async fn get_risk_metrics(&self) -> RiskMetrics {
        let stats = self.daily_stats.read().await;
        let positions = self.position_tracker.read().await;
        
        let win_rate = if stats.trade_count > 0 {
            stats.winning_trades as f64 / stats.trade_count as f64
        } else {
            0.0
        };
        
        let daily_loss_ratio = if self.config.max_daily_loss_sol > 0.0 {
            (-stats.total_pnl_sol.min(0.0)) / self.config.max_daily_loss_sol
        } else {
            0.0
        };
        
        let position_utilization = if self.config.max_position_size_sol > 0.0 {
            positions.total_exposure_sol / self.config.max_position_size_sol
        } else {
            0.0
        };
        
        RiskMetrics {
            daily_pnl_sol: stats.total_pnl_sol,
            daily_loss_ratio,
            position_utilization,
            trade_count: stats.trade_count,
            win_rate,
            max_drawdown_sol: stats.max_drawdown_sol,
            current_drawdown_sol: stats.current_drawdown_sol,
            total_exposure_sol: positions.total_exposure_sol,
        }
    }
}

impl DailyStats {
    fn new() -> Self {
        Self {
            date: chrono::Utc::now().naive_utc().date(),
            total_pnl_sol: 0.0,
            total_volume_sol: 0.0,
            trade_count: 0,
            winning_trades: 0,
            losing_trades: 0,
            max_drawdown_sol: 0.0,
            current_drawdown_sol: 0.0,
        }
    }
}

impl PositionTracker {
    fn new(max_position_size_sol: f64) -> Self {
        Self {
            current_positions: HashMap::new(),
            total_exposure_sol: 0.0,
            max_position_size_sol,
        }
    }
}

/// Risk metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub daily_pnl_sol: f64,
    pub daily_loss_ratio: f64,
    pub position_utilization: f64,
    pub trade_count: u32,
    pub win_rate: f64,
    pub max_drawdown_sol: f64,
    pub current_drawdown_sol: f64,
    pub total_exposure_sol: f64,
}
