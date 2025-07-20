use anyhow::Result;
use tracing::{debug, warn};
use crate::cerberus::{PositionState, MarketData};

/// Decision output from the decision tree
#[derive(Debug, Clone)]
pub enum Decision {
    /// Sell position with reason
    Sell(String),
    /// Buy more tokens (scale in)
    BuyMore(f64), // amount in SOL
    /// Hold position (no action)
    Hold,
}

/// Run the complete decision tree for a position
/// 
/// Decision priority:
/// 1. Hard Rules (immediate execution, no AI override)
///    - Timeout
///    - Stop Loss
///    - Take Profit
///    - Emergency Exit
/// 2. Soft Rules (AI-driven signals)
///    - Cerebro SELL signal
///    - Cerebro BUY_MORE signal
/// 3. Default: HOLD
pub async fn run_decision_tree(
    position: &PositionState,
    market_data: &MarketData,
) -> Result<Decision> {
    debug!("🧠 Running decision tree for {}", position.mint);

    // Calculate current PnL
    let current_pnl = position.calculate_pnl(market_data.price);
    
    // === HARD RULES (Non-negotiable) ===
    
    // 1. Timeout Check
    if position.is_timed_out() {
        warn!("⏰ Position {} timed out after {} seconds", 
              position.mint, position.age_seconds());
        return Ok(Decision::Sell("TIMEOUT".to_string()));
    }

    // 2. Stop Loss Check
    if position.should_stop_loss(current_pnl) {
        warn!("🛑 Stop loss triggered for {} at {:.2}% (target: {:.2}%)", 
              position.mint, current_pnl, position.stop_loss_target_percent);
        return Ok(Decision::Sell("STOP_LOSS".to_string()));
    }

    // 3. Take Profit Check
    if position.should_take_profit(current_pnl) {
        debug!("💰 Take profit triggered for {} at {:.2}% (target: {:.2}%)", 
               position.mint, current_pnl, position.take_profit_target_percent);
        return Ok(Decision::Sell("TAKE_PROFIT".to_string()));
    }

    // 4. Market Quality Checks
    if let Some(market_decision) = check_market_conditions(position, market_data).await? {
        return Ok(market_decision);
    }

    // === SOFT RULES (AI-driven, checked via Redis) ===
    
    // These are handled by external command listener in the main loop
    // The Redis pubsub system will trigger immediate actions when needed
    
    // 5. Risk Management Checks
    if let Some(risk_decision) = check_risk_conditions(position, market_data).await? {
        return Ok(risk_decision);
    }

    // === DEFAULT: HOLD ===
    debug!("📊 Holding position {} (PnL: {:.2}%)", position.mint, current_pnl);
    Ok(Decision::Hold)
}

/// Check market conditions that might trigger a sell
async fn check_market_conditions(
    position: &PositionState,
    market_data: &MarketData,
) -> Result<Option<Decision>> {
    
    // Check if market data is too stale
    if market_data.is_stale() {
        warn!("📊 Stale market data for {}, considering exit", position.mint);
        return Ok(Some(Decision::Sell("STALE_DATA".to_string())));
    }

    // Check liquidity
    let min_liquidity = position.position_size_sol * 10.0; // 10x position size
    if !market_data.has_sufficient_liquidity(min_liquidity) {
        warn!("💧 Insufficient liquidity for {} (need: {}, have: {})", 
              position.mint, min_liquidity, market_data.liquidity);
        return Ok(Some(Decision::Sell("LOW_LIQUIDITY".to_string())));
    }

    // Check spread
    if !market_data.has_acceptable_spread(5.0) { // 5% max spread
        warn!("📈 Excessive spread for {} ({:.2}%)", 
              position.mint, market_data.bid_ask_spread);
        return Ok(Some(Decision::Sell("HIGH_SPREAD".to_string())));
    }

    Ok(None)
}

/// Check risk management conditions
async fn check_risk_conditions(
    position: &PositionState,
    market_data: &MarketData,
) -> Result<Option<Decision>> {
    
    // Extreme volatility check
    if market_data.price_change_24h.abs() > 50.0 {
        warn!("🌪️ Extreme volatility detected for {} ({:.2}% 24h change)", 
              position.mint, market_data.price_change_24h);
        return Ok(Some(Decision::Sell("HIGH_VOLATILITY".to_string())));
    }

    // Position size vs account balance check
    // This would require account balance data - placeholder for now
    
    // Time-based risk scaling
    let age_hours = position.age_seconds() as f64 / 3600.0;
    if age_hours > 2.0 {
        // After 2 hours, tighten stop loss
        let tighter_stop_loss = position.stop_loss_target_percent * 0.8; // 20% tighter
        let current_pnl = position.calculate_pnl(market_data.price);
        
        if current_pnl <= tighter_stop_loss {
            warn!("⏱️ Time-based tighter stop loss triggered for {} at {:.2}% (tighter target: {:.2}%)", 
                  position.mint, current_pnl, tighter_stop_loss);
            return Ok(Some(Decision::Sell("TIME_BASED_STOP".to_string())));
        }
    }

    Ok(None)
}

/// Advanced decision tree for scaling strategies
pub async fn run_scaling_decision_tree(
    position: &PositionState,
    market_data: &MarketData,
    account_balance_sol: f64,
) -> Result<Decision> {
    
    // First run standard decision tree
    let base_decision = run_decision_tree(position, market_data).await?;
    
    // If it's a HOLD, consider scaling opportunities
    if matches!(base_decision, Decision::Hold) {
        if let Some(scale_decision) = check_scaling_opportunities(
            position, 
            market_data, 
            account_balance_sol
        ).await? {
            return Ok(scale_decision);
        }
    }

    Ok(base_decision)
}

/// Check for scaling (DCA) opportunities
async fn check_scaling_opportunities(
    position: &PositionState,
    market_data: &MarketData,
    account_balance_sol: f64,
) -> Result<Option<Decision>> {
    
    let current_pnl = position.calculate_pnl(market_data.price);
    
    // Only scale in if we're down but not at stop loss
    if current_pnl < -5.0 && current_pnl > position.stop_loss_target_percent {
        
        // Check if we have enough balance
        let scale_amount = position.position_size_sol * 0.5; // 50% of original position
        
        if account_balance_sol >= scale_amount {
            // Additional checks for scaling
            if market_data.volume_24h > 1000.0 && // Sufficient volume
               market_data.has_acceptable_spread(3.0) { // Tighter spread for scaling
                
                debug!("📈 Scaling opportunity for {} at {:.2}% down", 
                       position.mint, current_pnl);
                return Ok(Some(Decision::BuyMore(scale_amount)));
            }
        }
    }

    Ok(None)
}

/// Emergency decision tree (bypasses normal rules)
pub async fn run_emergency_decision_tree(
    position: &PositionState,
    emergency_reason: &str,
) -> Result<Decision> {
    
    warn!("🚨 Emergency decision for {}: {}", position.mint, emergency_reason);
    
    match emergency_reason {
        "GLOBAL_MARKET_CRASH" => Ok(Decision::Sell("EMERGENCY_CRASH".to_string())),
        "RUG_PULL_DETECTED" => Ok(Decision::Sell("EMERGENCY_RUG".to_string())),
        "EXCHANGE_ISSUES" => Ok(Decision::Sell("EMERGENCY_EXCHANGE".to_string())),
        "ACCOUNT_COMPROMISE" => Ok(Decision::Sell("EMERGENCY_SECURITY".to_string())),
        _ => Ok(Decision::Sell(format!("EMERGENCY_{}", emergency_reason))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerberus::PositionState;

    #[tokio::test]
    async fn test_timeout_decision() {
        let mut position = PositionState::new(
            "test".to_string(),
            0.001,
            0.1,
            "test".to_string(),
            "test".to_string(),
        );
        
        // Set position as very old
        position.entry_timestamp = 0;
        position.timeout_seconds = 100;
        
        let market_data = MarketData::new("test".to_string(), 0.001);
        let decision = run_decision_tree(&position, &market_data).await.unwrap();
        
        assert!(matches!(decision, Decision::Sell(reason) if reason == "TIMEOUT"));
    }

    #[tokio::test]
    async fn test_stop_loss_decision() {
        let position = PositionState::new(
            "test".to_string(),
            0.001,
            0.1,
            "test".to_string(),
            "test".to_string(),
        );
        
        // Price dropped 30% (below 25% stop loss)
        let market_data = MarketData::new("test".to_string(), 0.0007);
        let decision = run_decision_tree(&position, &market_data).await.unwrap();
        
        assert!(matches!(decision, Decision::Sell(reason) if reason == "STOP_LOSS"));
    }

    #[tokio::test]
    async fn test_take_profit_decision() {
        let position = PositionState::new(
            "test".to_string(),
            0.001,
            0.1,
            "test".to_string(),
            "test".to_string(),
        );
        
        // Price doubled (above 100% take profit)
        let market_data = MarketData::new("test".to_string(), 0.002);
        let decision = run_decision_tree(&position, &market_data).await.unwrap();
        
        assert!(matches!(decision, Decision::Sell(reason) if reason == "TAKE_PROFIT"));
    }

    #[tokio::test]
    async fn test_hold_decision() {
        let position = PositionState::new(
            "test".to_string(),
            0.001,
            0.1,
            "test".to_string(),
            "test".to_string(),
        );
        
        // Price up 10% (within normal range)
        let market_data = MarketData::new("test".to_string(), 0.0011);
        let decision = run_decision_tree(&position, &market_data).await.unwrap();
        
        assert!(matches!(decision, Decision::Hold));
    }
}
