//! Simple Engine with Bridge Integration
//! 
//! A minimal working implementation of the HFT engine with mempool bridge.

use crate::{config::Config, market::MarketData, strategy::Strategy, bridge::*};
use anyhow::Result;
use tokio::sync::broadcast;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, warn, error, debug};

/// Simple HFT Engine with bridge integration
pub struct SimpleEngine {
    config: Config,
    market_data: MarketData,
    strategy: Box<dyn Strategy>,
    dry_run: bool,
    event_processor: SimpleEventProcessor,
    stats: EngineStats,
}

/// Engine statistics
#[derive(Debug, Default)]
pub struct EngineStats {
    pub events_processed: u64,
    pub opportunities_found: u64,
    pub trades_executed: u64,
    pub total_profit_sol: f64,
    pub uptime_seconds: u64,
}

impl SimpleEngine {
    /// Create new simple engine
    pub async fn new(config: Config, dry_run: bool) -> Result<Self> {
        let market_data = MarketData::new(&config.solana).await?;
        let strategy = crate::strategy::create_strategy(&config.strategy)?;
        let event_processor = SimpleEventProcessor::new();
        
        Ok(Self {
            config,
            market_data,
            strategy,
            dry_run,
            event_processor,
            stats: EngineStats::default(),
        })
    }

    /// Run engine with bridge integration
    pub async fn run_with_bridge(&mut self, mut bridge_rx: broadcast::Receiver<Arc<BridgeEvent>>) -> Result<()> {
        info!("🚀 Starting Simple HFT Engine with bridge integration...");
        
        let start_time = Instant::now();
        let mut last_strategy_run = Instant::now();
        let strategy_interval = Duration::from_millis(self.config.strategy.update_interval_ms);
        
        // Print startup info
        info!("⚙️  Configuration:");
        info!("   - Dry run: {}", self.dry_run);
        info!("   - Strategy interval: {}ms", self.config.strategy.update_interval_ms);
        info!("   - Max position size: {} SOL", self.config.trading.max_position_size_sol);
        
        loop {
            tokio::select! {
                // HIGH PRIORITY: Real-time bridge events
                Ok(event) = bridge_rx.recv() => {
                    let processing_start = Instant::now();
                    
                    match self.process_bridge_event(&event).await {
                        Ok(result) => {
                            let latency = processing_start.elapsed();
                            self.stats.events_processed += 1;
                            
                            if result.success {
                                self.stats.opportunities_found += 1;
                                self.stats.total_profit_sol += result.profit_estimate;
                                
                                info!(
                                    "✅ Event processed in {:?} - {} (Profit: {:.6} SOL)", 
                                    latency,
                                    result.action_taken,
                                    result.profit_estimate
                                );
                            } else {
                                debug!("⏭️  Event skipped: {}", result.action_taken);
                            }
                        }
                        Err(e) => {
                            error!("❌ Event processing failed: {}", e);
                        }
                    }
                }
                
                // LOW PRIORITY: Regular strategy execution
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    if last_strategy_run.elapsed() >= strategy_interval {
                        if let Err(e) = self.run_regular_strategy().await {
                            error!("Strategy execution error: {}", e);
                        }
                        last_strategy_run = Instant::now();
                    }
                    
                    // Update uptime
                    self.stats.uptime_seconds = start_time.elapsed().as_secs();
                    
                    // Print stats every 60 seconds
                    if self.stats.uptime_seconds % 60 == 0 && self.stats.uptime_seconds > 0 {
                        self.print_stats();
                    }
                }
            }
        }
    }

    /// Legacy run method for backward compatibility
    pub async fn run(&mut self) -> Result<()> {
        warn!("⚠️  Running engine without bridge integration - MEV opportunities will be missed!");
        
        loop {
            if let Err(e) = self.run_regular_strategy().await {
                error!("Strategy execution error: {}", e);
            }
            
            tokio::time::sleep(Duration::from_millis(self.config.strategy.update_interval_ms)).await;
        }
    }
    
    /// Process bridge event
    async fn process_bridge_event(&mut self, event: &BridgeEvent) -> Result<ProcessingResult, ProcessingError> {
        // Basic risk check
        if event.priority > 3 {
            return Ok(ProcessingResult {
                success: false,
                action_taken: "Rejected due to low priority".to_string(),
                profit_estimate: 0.0,
            });
        }
        
        // Process the event
        let result = self.event_processor.process_event(event).await?;
        
        // If this is a real opportunity and not dry run, we would execute trades here
        if result.success && result.profit_estimate > 0.001 && !self.dry_run {
            // TODO: Implement actual trade execution
            info!("💰 Would execute trade for {:.6} SOL profit", result.profit_estimate);
            self.stats.trades_executed += 1;
        }
        
        Ok(result)
    }
    
    /// Run regular trading strategy (non-MEV)
    async fn run_regular_strategy(&mut self) -> Result<()> {
        let market_snapshot = self.market_data.get_snapshot().await?;
        let orders = self.strategy.generate_orders(&market_snapshot).await?;
        
        for order in orders {
            if !self.dry_run {
                debug!("Executing regular order: {:?}", order);
                // TODO: Implement order execution
            }
        }
        
        Ok(())
    }
    
    /// Print engine statistics
    fn print_stats(&self) {
        info!("📊 Engine Stats ({}s uptime):", self.stats.uptime_seconds);
        info!("   - Events processed: {}", self.stats.events_processed);
        info!("   - Opportunities found: {}", self.stats.opportunities_found);
        info!("   - Trades executed: {}", self.stats.trades_executed);
        info!("   - Total profit: {:.6} SOL", self.stats.total_profit_sol);
        
        if self.stats.events_processed > 0 {
            let success_rate = (self.stats.opportunities_found as f64 / self.stats.events_processed as f64) * 100.0;
            info!("   - Success rate: {:.1}%", success_rate);
        }
        
        let (processor_count, avg_time) = self.event_processor.get_stats();
        info!("   - Avg processing time: {:.1}ms", avg_time);
        info!("   - Processor events: {}", processor_count);
    }
    
    /// Get current statistics
    pub fn get_stats(&self) -> &EngineStats {
        &self.stats
    }

    /// Check if engine is in dry run mode
    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    /// Process event from bridge
    pub async fn process_event(&mut self, event_data: &[u8]) -> Result<()> {
        // Simple event processing
        debug!("Processing event: {} bytes", event_data.len());

        // In a real implementation, would parse and handle different event types
        tokio::time::sleep(Duration::from_millis(1)).await;

        Ok(())
    }
}

/// Enhanced mempool listener that uses the bridge
pub async fn start_bridge_mempool_listener() -> Result<tokio::task::JoinHandle<()>> {
    use tokio::sync::mpsc;
    
    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
    
    // Create event detector
    let event_detector = SimpleEventDetector::new();
    
    // Simulate mempool listener (in real implementation, this would connect to Helius)
    let listener_handle = tokio::spawn(async move {
        info!("🎧 Starting bridge mempool listener...");
        
        let mut transaction_count = 0;
        let mut event_count = 0;
        
        // Simulate receiving transactions
        loop {
            // Simulate transaction data
            let tx_data = vec![0u8; 1500]; // Simulate large transaction
            transaction_count += 1;
            
            // Detect events
            let events = event_detector.detect_events(&tx_data);
            event_count += events.len();
            
            // Send events through bridge
            for event in events {
                if let Err(e) = send_bridge_event(event) {
                    error!("Failed to send bridge event: {}", e);
                }
            }
            
            // Log progress
            if transaction_count % 100 == 0 {
                info!("📈 Processed {} transactions, detected {} events", transaction_count, event_count);
            }
            
            // Simulate transaction arrival rate (10 TPS)
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
    
    info!("🌉 Bridge mempool listener started");
    Ok(listener_handle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;
    
    fn create_test_config() -> Config {
        Config {
            solana: SolanaConfig {
                rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                ws_url: "wss://api.mainnet-beta.solana.com".to_string(),
                rpc_timeout_ms: 5000,
            },
            wallet: WalletConfig {
                private_key_path: "test.key".to_string(),
                keypair_path: "test.json".to_string(),
            },
            trading: TradingConfig {
                initial_balance_sol: 10.0,
                max_position_size_sol: 1.0,
                max_slippage_bps: 100,
                min_profit_threshold_bps: 50,
                risk_limit_bps: 1000,
            },
            strategy: StrategyConfig {
                strategy_mode: "market_making".to_string(),
                update_interval_ms: 1000,
                order_book_depth: 10,
                spread_bps: 50,
            },
            risk: RiskConfig::default(),
            logging: LoggingConfig {
                rust_log: "info".to_string(),
                log_level: "info".to_string(),
                log_file_path: "test.log".to_string(),
            },
            monitoring: MonitoringConfig {
                metrics_port: 8080,
                health_check_interval_ms: 30000,
                enable_ddos_protection: false,
                rate_limit_rps: 100,
            },
        }
    }
    
    #[tokio::test]
    async fn test_simple_engine_creation() {
        let config = create_test_config();
        let engine = SimpleEngine::new(config, true).await;
        assert!(engine.is_ok());
    }
    
    #[tokio::test]
    async fn test_bridge_event_processing() {
        let config = create_test_config();
        let mut engine = SimpleEngine::new(config, true).await.unwrap();
        
        let event = BridgeEvent {
            event_type: EventType::DexTransaction {
                signature: "test".to_string(),
                program: "Raydium".to_string(),
                accounts: vec!["acc1".to_string()],
            },
            timestamp: 1234567890,
            priority: 0,
        };
        
        let result = engine.process_bridge_event(&event).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }
}
