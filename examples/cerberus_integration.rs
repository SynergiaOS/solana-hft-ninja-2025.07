use anyhow::Result;
use solana_hft_ninja::cerberus::{
    CerberusBrain, CerberusConfig, PositionState, PositionStatus
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

/// Example integration of Cerberus with existing HFT strategies
/// 
/// This example shows how to:
/// 1. Initialize Cerberus with premium endpoints
/// 2. Create positions from strategy signals
/// 3. Monitor position lifecycle
/// 4. Handle external commands
/// 5. Integrate with existing risk management

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("cerberus_integration=info,solana_hft_ninja=debug")
        .init();

    info!("🚀 Starting Cerberus Integration Example");

    // Example 1: Initialize Cerberus with premium endpoints
    let cerberus = initialize_cerberus().await?;

    // Example 2: Create positions from strategy signals
    simulate_strategy_signals(&cerberus).await?;

    // Example 3: Monitor positions
    monitor_positions(&cerberus).await?;

    // Example 4: Send external commands
    send_external_commands(&cerberus).await?;

    // Example 5: Emergency procedures
    demonstrate_emergency_procedures(&cerberus).await?;

    info!("✅ Cerberus integration example completed");
    Ok(())
}

/// Initialize Cerberus with production configuration
async fn initialize_cerberus() -> Result<Arc<CerberusBrain>> {
    info!("🧠 Initializing Cerberus Brain");

    let config = CerberusConfig {
        loop_interval_ms: 200,
        quicknode_endpoint: std::env::var("QUICKNODE_ENDPOINT")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
        helius_endpoint: std::env::var("HELIUS_ENDPOINT")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
        redis_url: "redis://127.0.0.1:6379".to_string(),
        jito_endpoint: "https://mainnet.block-engine.jito.wtf".to_string(),
        max_concurrent_positions: 20,
        default_timeout_seconds: 300, // 5 minutes for this example
        emergency_stop_enabled: true,
    };

    let cerberus = CerberusBrain::new(config).await?;
    
    info!("✅ Cerberus initialized successfully");
    Ok(Arc::new(cerberus))
}

/// Simulate strategy signals creating positions
async fn simulate_strategy_signals(cerberus: &Arc<CerberusBrain>) -> Result<()> {
    info!("📈 Simulating strategy signals");

    // Example tokens for demonstration
    let example_positions = vec![
        ("So11111111111111111111111111111111111111112", 0.001, 0.1), // SOL
        ("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", 1.0, 0.05),   // USDC
        ("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", 1.0, 0.05),   // USDT
    ];

    for (mint, entry_price, position_size) in example_positions {
        // Create position from strategy signal
        let position = create_position_from_signal(
            mint,
            entry_price,
            position_size,
            "sandwich_strategy",
        ).await?;

        // Store in Cerberus for autonomous management
        cerberus.store.store_position(&position).await?;
        
        info!("📊 Created position: {} - {} SOL at {} SOL", 
              mint, position_size, entry_price);
        
        // Small delay between positions
        sleep(Duration::from_millis(100)).await;
    }

    info!("✅ Strategy signals processed");
    Ok(())
}

/// Create a position from strategy signal
async fn create_position_from_signal(
    mint: &str,
    entry_price: f64,
    position_size_sol: f64,
    strategy_id: &str,
) -> Result<PositionState> {
    let mut position = PositionState::new(
        mint.to_string(),
        entry_price,
        position_size_sol,
        strategy_id.to_string(),
        "example_wallet".to_string(),
    );

    // Customize position parameters based on strategy
    match strategy_id {
        "sandwich_strategy" => {
            position.take_profit_target_percent = 50.0;  // 50% profit target
            position.stop_loss_target_percent = -15.0;   // 15% stop loss
            position.timeout_seconds = 180;              // 3 minutes timeout
        },
        "arbitrage_strategy" => {
            position.take_profit_target_percent = 20.0;  // 20% profit target
            position.stop_loss_target_percent = -10.0;   // 10% stop loss
            position.timeout_seconds = 60;               // 1 minute timeout
        },
        "market_making" => {
            position.take_profit_target_percent = 5.0;   // 5% profit target
            position.stop_loss_target_percent = -5.0;    // 5% stop loss
            position.timeout_seconds = 3600;             // 1 hour timeout
        },
        _ => {
            // Default parameters
        }
    }

    Ok(position)
}

/// Monitor position lifecycle
async fn monitor_positions(cerberus: &Arc<CerberusBrain>) -> Result<()> {
    info!("👁️ Monitoring positions");

    for i in 0..10 {
        let positions = cerberus.store.get_all_open_positions().await?;
        let position_count = positions.len();
        
        if position_count > 0 {
            info!("📊 Monitoring cycle {}: {} active positions", i + 1, position_count);
            
            // Show position details
            for position in &positions {
                let age = position.age_seconds();
                let pnl = position.pnl_unrealized_percent.unwrap_or(0.0);
                
                info!("  Position {}: Age {}s, PnL {:.2}%", 
                      &position.mint[..8], age, pnl);
            }
        } else {
            info!("📊 Monitoring cycle {}: No active positions", i + 1);
            break;
        }
        
        sleep(Duration::from_secs(2)).await;
    }

    info!("✅ Position monitoring completed");
    Ok(())
}

/// Send external commands to Cerberus
async fn send_external_commands(cerberus: &Arc<CerberusBrain>) -> Result<()> {
    info!("📡 Sending external commands");

    // Get active positions to send commands for
    let positions = cerberus.store.get_all_open_positions().await?;
    
    if positions.is_empty() {
        info!("No active positions to send commands for");
        return Ok(());
    }

    let first_position = &positions[0];

    // Example 1: Cerebro AI signal to sell
    send_cerebro_sell_signal(&first_position.mint).await?;
    sleep(Duration::from_secs(1)).await;

    // Example 2: Cerebro AI signal to buy more
    if positions.len() > 1 {
        send_cerebro_buy_more_signal(&positions[1].mint, 0.05).await?;
        sleep(Duration::from_secs(1)).await;
    }

    // Example 3: Update position targets
    send_cerebro_update_targets(&first_position.mint, 75.0, -20.0).await?;

    info!("✅ External commands sent");
    Ok(())
}

/// Send Cerebro sell signal
async fn send_cerebro_sell_signal(mint: &str) -> Result<()> {
    let command = serde_json::json!({
        "action": "SELL",
        "mint": mint,
        "reason": "AI_BEARISH_SIGNAL",
        "confidence": 0.85
    });

    // In a real implementation, this would use Redis pubsub
    info!("🤖 Cerebro SELL signal: {}", command);
    Ok(())
}

/// Send Cerebro buy more signal
async fn send_cerebro_buy_more_signal(mint: &str, amount_sol: f64) -> Result<()> {
    let command = serde_json::json!({
        "action": "BUY_MORE",
        "mint": mint,
        "amount_sol": amount_sol,
        "reason": "AI_DCA_SIGNAL",
        "confidence": 0.75
    });

    info!("🤖 Cerebro BUY_MORE signal: {}", command);
    Ok(())
}

/// Send Cerebro update targets signal
async fn send_cerebro_update_targets(
    mint: &str, 
    take_profit: f64, 
    stop_loss: f64
) -> Result<()> {
    let command = serde_json::json!({
        "action": "UPDATE_TARGETS",
        "mint": mint,
        "take_profit_percent": take_profit,
        "stop_loss_percent": stop_loss,
        "reason": "AI_RISK_ADJUSTMENT"
    });

    info!("🤖 Cerebro UPDATE_TARGETS signal: {}", command);
    Ok(())
}

/// Demonstrate emergency procedures
async fn demonstrate_emergency_procedures(cerberus: &Arc<CerberusBrain>) -> Result<()> {
    info!("🚨 Demonstrating emergency procedures");

    // Check if there are any positions to close
    let positions = cerberus.store.get_all_open_positions().await?;
    
    if !positions.is_empty() {
        warn!("🚨 Triggering emergency stop for {} positions", positions.len());
        
        // Emergency stop all positions
        cerberus.emergency_stop("EXAMPLE_EMERGENCY").await?;
        
        // Verify all positions are closed
        sleep(Duration::from_secs(1)).await;
        let remaining_positions = cerberus.store.get_all_open_positions().await?;
        
        if remaining_positions.is_empty() {
            info!("✅ Emergency stop successful - all positions closed");
        } else {
            warn!("⚠️ {} positions still open after emergency stop", remaining_positions.len());
        }
    } else {
        info!("No positions to emergency stop");
    }

    // Example Guardian alerts
    send_guardian_alerts().await?;

    info!("✅ Emergency procedures demonstrated");
    Ok(())
}

/// Send Guardian alert examples
async fn send_guardian_alerts() -> Result<()> {
    let alerts = vec![
        serde_json::json!({
            "action": "PAUSE_TRADING",
            "reason": "HIGH_VOLATILITY_DETECTED"
        }),
        serde_json::json!({
            "action": "RESUME_TRADING",
            "reason": "VOLATILITY_NORMALIZED"
        }),
    ];

    for alert in alerts {
        info!("🛡️ Guardian alert: {}", alert);
        sleep(Duration::from_millis(500)).await;
    }

    Ok(())
}

/// Example of integrating Cerberus with existing strategy
pub struct StrategyWithCerberus {
    cerberus: Arc<CerberusBrain>,
    strategy_id: String,
}

impl StrategyWithCerberus {
    pub async fn new(cerberus: Arc<CerberusBrain>, strategy_id: String) -> Self {
        Self {
            cerberus,
            strategy_id,
        }
    }

    /// Execute a trade and hand over to Cerberus for management
    pub async fn execute_trade(
        &self,
        mint: &str,
        entry_price: f64,
        position_size_sol: f64,
    ) -> Result<()> {
        info!("🎯 Executing trade: {} - {} SOL", mint, position_size_sol);

        // 1. Execute the actual trade (placeholder)
        self.execute_market_order(mint, position_size_sol).await?;

        // 2. Create position for Cerberus management
        let position = create_position_from_signal(
            mint,
            entry_price,
            position_size_sol,
            &self.strategy_id,
        ).await?;

        // 3. Hand over to Cerberus
        self.cerberus.store.store_position(&position).await?;

        info!("✅ Trade executed and handed over to Cerberus");
        Ok(())
    }

    /// Placeholder for actual market order execution
    async fn execute_market_order(&self, _mint: &str, _amount: f64) -> Result<()> {
        // In real implementation, this would:
        // 1. Build Jupiter swap transaction
        // 2. Create Jito bundle
        // 3. Send to network
        // 4. Confirm execution
        
        info!("📈 Market order executed (placeholder)");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_position_creation() {
        let position = create_position_from_signal(
            "So11111111111111111111111111111111111111112",
            0.001,
            0.1,
            "test_strategy",
        ).await.unwrap();

        assert_eq!(position.mint, "So11111111111111111111111111111111111111112");
        assert_eq!(position.entry_price, 0.001);
        assert_eq!(position.position_size_sol, 0.1);
        assert_eq!(position.strategy_id, "test_strategy");
        assert_eq!(position.status, PositionStatus::Open);
    }
}
