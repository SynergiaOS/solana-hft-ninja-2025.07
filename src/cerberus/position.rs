use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Result;

/// Position state stored in DragonflyDB/Redis
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PositionState {
    pub mint: String,
    pub entry_price: f64,
    pub entry_timestamp: u64,
    pub position_size_sol: f64,
    pub strategy_id: String,
    pub status: PositionStatus,
    pub take_profit_target_percent: f64,
    pub stop_loss_target_percent: f64,
    pub timeout_seconds: u64,
    pub risk_score_at_entry: u8,
    
    // Runtime fields (updated during analysis)
    pub current_price: Option<f64>,
    pub pnl_unrealized_percent: Option<f64>,
    pub last_analysis_timestamp: u64,
    
    // Metadata
    pub wallet_address: String,
    pub dex_used: String, // "Raydium", "Orca", "Jupiter"
    pub slippage_tolerance: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum PositionStatus {
    Open,
    Closed,
    Pending,
    Failed,
}

impl PositionState {
    /// Create new position
    pub fn new(
        mint: String,
        entry_price: f64,
        position_size_sol: f64,
        strategy_id: String,
        wallet_address: String,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            mint,
            entry_price,
            entry_timestamp: now,
            position_size_sol,
            strategy_id,
            status: PositionStatus::Open,
            take_profit_target_percent: 100.0, // 100% profit target
            stop_loss_target_percent: -25.0,   // 25% loss limit
            timeout_seconds: 600,              // 10 minutes
            risk_score_at_entry: 50,           // Medium risk
            current_price: None,
            pnl_unrealized_percent: None,
            last_analysis_timestamp: now,
            wallet_address,
            dex_used: "Jupiter".to_string(),
            slippage_tolerance: 1.0, // 1%
        }
    }

    /// Redis key for this position
    pub fn redis_key(&self) -> String {
        format!("position:{}", self.mint)
    }

    /// Position age in seconds
    pub fn age_seconds(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now.saturating_sub(self.entry_timestamp)
    }

    /// Check if position has timed out
    pub fn is_timed_out(&self) -> bool {
        self.age_seconds() > self.timeout_seconds
    }

    /// Calculate current PnL percentage
    pub fn calculate_pnl(&self, current_price: f64) -> f64 {
        ((current_price - self.entry_price) / self.entry_price) * 100.0
    }

    /// Check if stop loss should trigger
    pub fn should_stop_loss(&self, current_pnl: f64) -> bool {
        current_pnl <= self.stop_loss_target_percent
    }

    /// Check if take profit should trigger
    pub fn should_take_profit(&self, current_pnl: f64) -> bool {
        current_pnl >= self.take_profit_target_percent
    }

    /// Get position value in SOL
    pub fn current_value_sol(&self) -> Option<f64> {
        self.current_price.map(|price| {
            let token_amount = self.position_size_sol / self.entry_price;
            token_amount * price
        })
    }

    /// Get unrealized PnL in SOL
    pub fn unrealized_pnl_sol(&self) -> Option<f64> {
        self.current_value_sol().map(|current_value| {
            current_value - self.position_size_sol
        })
    }

    /// Update with latest market data
    pub fn update_market_data(&mut self, current_price: f64) {
        self.current_price = Some(current_price);
        self.pnl_unrealized_percent = Some(self.calculate_pnl(current_price));
        self.last_analysis_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Serialize to JSON for Redis storage
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

/// Market data for decision making
#[derive(Debug, Clone)]
pub struct MarketData {
    pub mint: String,
    pub price: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
    pub liquidity: f64,
    pub bid_ask_spread: f64,
    pub timestamp: u64,
}

impl MarketData {
    pub fn new(mint: String, price: f64) -> Self {
        Self {
            mint,
            price,
            volume_24h: 0.0,
            price_change_24h: 0.0,
            liquidity: 0.0,
            bid_ask_spread: 0.0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Check if market data is stale (older than 5 seconds)
    pub fn is_stale(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now.saturating_sub(self.timestamp) > 5
    }

    /// Check if market has sufficient liquidity
    pub fn has_sufficient_liquidity(&self, min_liquidity: f64) -> bool {
        self.liquidity >= min_liquidity
    }

    /// Check if spread is acceptable for trading
    pub fn has_acceptable_spread(&self, max_spread_percent: f64) -> bool {
        self.bid_ask_spread <= max_spread_percent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let position = PositionState::new(
            "So11111111111111111111111111111111111111112".to_string(),
            0.001,
            0.1,
            "test-strategy".to_string(),
            "test-wallet".to_string(),
        );

        assert_eq!(position.status, PositionStatus::Open);
        assert_eq!(position.entry_price, 0.001);
        assert_eq!(position.position_size_sol, 0.1);
    }

    #[test]
    fn test_pnl_calculation() {
        let position = PositionState::new(
            "test".to_string(),
            0.001,
            0.1,
            "test".to_string(),
            "test".to_string(),
        );

        // 50% profit
        let pnl = position.calculate_pnl(0.0015);
        assert_eq!(pnl, 50.0);

        // 25% loss
        let pnl = position.calculate_pnl(0.00075);
        assert_eq!(pnl, -25.0);
    }

    #[test]
    fn test_stop_loss_take_profit() {
        let position = PositionState::new(
            "test".to_string(),
            0.001,
            0.1,
            "test".to_string(),
            "test".to_string(),
        );

        assert!(position.should_stop_loss(-30.0));
        assert!(!position.should_stop_loss(-20.0));
        
        assert!(position.should_take_profit(150.0));
        assert!(!position.should_take_profit(50.0));
    }
}
