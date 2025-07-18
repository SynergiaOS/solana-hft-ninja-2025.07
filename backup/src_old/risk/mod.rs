//! Risk Management Module
//! 
//! Provides real-time risk assessment for MEV opportunities and trading decisions.
//! Implements position limits, drawdown controls, and opportunity validation.

use crate::config::RiskConfig;
use crate::engine::{MevOpportunity};
use crate::mempool::router::OpportunityType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn, error};

/// Risk assessment decision
#[derive(Debug, Clone, PartialEq)]
pub enum RiskDecision {
    /// Approve the opportunity
    Approve,
    /// Reject with reason
    Reject(String),
    /// Approve with reduced size
    ReduceSize(f64), // New size multiplier (0.0 to 1.0)
}

/// Position tracking for risk management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub token: String,
    pub size_sol: f64,
    pub entry_price: f64,
    pub entry_time: u64,
    pub unrealized_pnl: f64,
}

/// Daily trading statistics
#[derive(Debug, Default)]
pub struct DailyStats {
    pub trades_count: u32,
    pub successful_trades: u32,
    pub total_pnl_sol: f64,
    pub max_drawdown_sol: f64,
    pub largest_loss_sol: f64,
    pub largest_win_sol: f64,
}

/// Risk manager for MEV opportunities
pub struct RiskManager {
    config: RiskConfig,
    positions: HashMap<String, Position>,
    daily_stats: DailyStats,
    daily_reset_time: u64,
    total_capital_sol: f64,
    available_capital_sol: f64,
}

impl RiskManager {
    /// Create new risk manager
    pub fn new(config: RiskConfig, initial_capital_sol: f64) -> Self {
        let today = get_today_timestamp();
        
        Self {
            config,
            positions: HashMap::new(),
            daily_stats: DailyStats::default(),
            daily_reset_time: today,
            total_capital_sol: initial_capital_sol,
            available_capital_sol: initial_capital_sol,
        }
    }
    
    /// Assess MEV opportunity risk
    pub fn assess_opportunity(&mut self, opportunity: &MevOpportunity) -> RiskDecision {
        // Reset daily stats if new day
        self.check_daily_reset();
        
        // Check daily loss limits
        if let Some(rejection) = self.check_daily_limits() {
            return rejection;
        }
        
        // Check position size limits
        if let Some(rejection) = self.check_position_limits(opportunity) {
            return rejection;
        }
        
        // Check opportunity-specific risks
        match &opportunity.opportunity_type {
            OpportunityType::Sandwich { .. } => self.assess_sandwich_risk(opportunity),
            OpportunityType::Arbitrage { .. } => self.assess_arbitrage_risk(opportunity),
            OpportunityType::NewToken { .. } => self.assess_new_token_risk(opportunity),
            OpportunityType::Liquidation { .. } => self.assess_liquidation_risk(opportunity),
            _ => RiskDecision::Reject("Unknown opportunity type".to_string()),
        }
    }
    
    /// Record trade execution result
    pub fn record_trade_result(&mut self, opportunity: &MevOpportunity, profit_sol: f64, success: bool) {
        self.daily_stats.trades_count += 1;
        
        if success {
            self.daily_stats.successful_trades += 1;
        }
        
        self.daily_stats.total_pnl_sol += profit_sol;
        
        if profit_sol > self.daily_stats.largest_win_sol {
            self.daily_stats.largest_win_sol = profit_sol;
        }
        
        if profit_sol < self.daily_stats.largest_loss_sol {
            self.daily_stats.largest_loss_sol = profit_sol;
        }
        
        // Update drawdown
        if self.daily_stats.total_pnl_sol < self.daily_stats.max_drawdown_sol {
            self.daily_stats.max_drawdown_sol = self.daily_stats.total_pnl_sol;
        }
        
        // Update available capital
        self.available_capital_sol += profit_sol;
        
        info!(
            "Trade recorded: Profit: {:.4} SOL | Daily P&L: {:.4} SOL | Success Rate: {:.1}%",
            profit_sol,
            self.daily_stats.total_pnl_sol,
            self.get_success_rate()
        );
    }
    
    /// Check daily trading limits
    fn check_daily_limits(&self) -> Option<RiskDecision> {
        let max_daily_loss = (self.config.max_drawdown_bps as f64 / 10_000.0) * self.total_capital_sol;
        
        if self.daily_stats.total_pnl_sol <= -max_daily_loss {
            return Some(RiskDecision::Reject(format!(
                "Daily loss limit exceeded: {:.4} SOL (limit: {:.4} SOL)",
                -self.daily_stats.total_pnl_sol,
                max_daily_loss
            )));
        }
        
        // Check maximum trades per day
        if self.daily_stats.trades_count >= 1000 {
            return Some(RiskDecision::Reject("Daily trade limit exceeded".to_string()));
        }
        
        None
    }
    
    /// Check position size limits
    fn check_position_limits(&self, opportunity: &MevOpportunity) -> Option<RiskDecision> {
        let max_position_sol = (self.config.risk_limit_bps as f64 / 10_000.0) * self.total_capital_sol;
        let required_capital = self.estimate_required_capital(opportunity);
        
        if required_capital > max_position_sol {
            let size_multiplier = max_position_sol / required_capital;
            if size_multiplier < 0.1 {
                return Some(RiskDecision::Reject(format!(
                    "Position too large: {:.4} SOL (limit: {:.4} SOL)",
                    required_capital,
                    max_position_sol
                )));
            } else {
                return Some(RiskDecision::ReduceSize(size_multiplier));
            }
        }
        
        if required_capital > self.available_capital_sol {
            return Some(RiskDecision::Reject(format!(
                "Insufficient capital: need {:.4} SOL, have {:.4} SOL",
                required_capital,
                self.available_capital_sol
            )));
        }
        
        None
    }
    
    /// Assess sandwich attack risk
    fn assess_sandwich_risk(&self, opportunity: &MevOpportunity) -> RiskDecision {
        // Sandwich attacks are high-risk due to regulatory concerns
        warn!("Sandwich attack detected - high regulatory risk");
        
        if opportunity.estimated_profit_sol < 0.05 {
            return RiskDecision::Reject("Sandwich profit too low for risk".to_string());
        }
        
        // Only allow small sandwich attacks
        if self.estimate_required_capital(opportunity) > 1.0 {
            return RiskDecision::Reject("Sandwich position too large".to_string());
        }
        
        RiskDecision::ReduceSize(0.5) // Reduce size by 50% for safety
    }
    
    /// Assess arbitrage risk
    fn assess_arbitrage_risk(&self, opportunity: &MevOpportunity) -> RiskDecision {
        // Arbitrage is generally low-risk
        if opportunity.estimated_profit_sol < 0.01 {
            return RiskDecision::Reject("Arbitrage profit too low".to_string());
        }
        
        RiskDecision::Approve
    }
    
    /// Assess new token risk
    fn assess_new_token_risk(&self, opportunity: &MevOpportunity) -> RiskDecision {
        // New tokens are high-risk (rug pulls, low liquidity)
        if opportunity.estimated_profit_sol < 0.02 {
            return RiskDecision::Reject("New token profit too low for risk".to_string());
        }
        
        // Limit exposure to new tokens
        RiskDecision::ReduceSize(0.3) // Only 30% of calculated size
    }
    
    /// Assess liquidation risk
    fn assess_liquidation_risk(&self, opportunity: &MevOpportunity) -> RiskDecision {
        // Liquidations are generally safe and beneficial to ecosystem
        RiskDecision::Approve
    }
    
    /// Estimate required capital for opportunity
    fn estimate_required_capital(&self, opportunity: &MevOpportunity) -> f64 {
        match &opportunity.opportunity_type {
            OpportunityType::Sandwich { swap_amount_in, .. } => {
                (*swap_amount_in as f64 / 1e9) * 0.1 // 10% of victim swap
            }
            OpportunityType::Arbitrage { optimal_amount, .. } => {
                *optimal_amount as f64 / 1e9
            }
            OpportunityType::NewToken { initial_liquidity_sol, .. } => {
                (*initial_liquidity_sol as f64 / 1e9) * 0.05 // 5% of initial liquidity
            }
            _ => 0.1, // Default 0.1 SOL
        }
    }
    
    /// Check if daily stats need reset
    fn check_daily_reset(&mut self) {
        let today = get_today_timestamp();
        if today > self.daily_reset_time {
            info!("Resetting daily trading statistics");
            self.daily_stats = DailyStats::default();
            self.daily_reset_time = today;
        }
    }
    
    /// Get current success rate
    pub fn get_success_rate(&self) -> f64 {
        if self.daily_stats.trades_count == 0 {
            0.0
        } else {
            (self.daily_stats.successful_trades as f64 / self.daily_stats.trades_count as f64) * 100.0
        }
    }
    
    /// Get daily statistics
    pub fn get_daily_stats(&self) -> &DailyStats {
        &self.daily_stats
    }
    
    /// Get available capital
    pub fn get_available_capital(&self) -> f64 {
        self.available_capital_sol
    }
}

/// Get timestamp for start of today (UTC)
fn get_today_timestamp() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Round down to start of day
    (now / 86400) * 86400
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RiskConfig;
    
    fn create_test_risk_config() -> RiskConfig {
        RiskConfig {
            stop_loss_bps: 200,      // 2%
            take_profit_bps: 300,    // 3%
            max_drawdown_bps: 500,   // 5%
        }
    }
    
    #[test]
    fn test_risk_manager_creation() {
        let config = create_test_risk_config();
        let rm = RiskManager::new(config, 10.0);
        
        assert_eq!(rm.total_capital_sol, 10.0);
        assert_eq!(rm.available_capital_sol, 10.0);
        assert_eq!(rm.daily_stats.trades_count, 0);
    }
    
    #[test]
    fn test_daily_loss_limit() {
        let config = create_test_risk_config();
        let mut rm = RiskManager::new(config, 10.0);
        
        // Simulate large loss
        rm.daily_stats.total_pnl_sol = -0.6; // 6% loss (exceeds 5% limit)
        
        let opportunity = MevOpportunity {
            opportunity_type: OpportunityType::Unknown,
            estimated_profit_sol: 0.1,
            estimated_gas_cost: 10_000,
            execution_deadline_ns: 0,
            priority: crate::mempool::router::EventPriority::Low,
        };
        
        let decision = rm.assess_opportunity(&opportunity);
        assert!(matches!(decision, RiskDecision::Reject(_)));
    }
}
