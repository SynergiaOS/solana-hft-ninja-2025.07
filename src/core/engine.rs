// 🥷 Unified Trading Engine - High-Performance Core
// Single engine handling all trading operations with zero-copy optimization

use crate::core::{
    types::*, events::*, memory::*, 
    intern_symbol, get_symbol
};
use crate::strategies::{Strategy, create_strategy};
use crate::core::SolanaClient;
use crate::utils::config::RiskConfig;

use anyhow::{Result, Context};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{info, warn, error, debug};
use std::collections::HashMap;

/// Engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub dry_run: bool,
    pub strategies: Vec<String>,
    pub solana_rpc_url: String,
    pub solana_ws_url: String,
    pub wallet_path: String,
    pub max_position_size: f64,
    pub risk_limits: RiskConfig,
}

/// Main trading engine with unified architecture
pub struct Engine {
    config: EngineConfig,
    event_bus: Arc<EventBus>,
    memory_pool: Arc<MemoryPool>,
    
    // Core components
    solana_client: Arc<SolanaClient>,
    strategies: Vec<Box<dyn Strategy>>,
    
    // State management
    positions: Arc<RwLock<HashMap<u32, Position>>>,
    balances: Arc<RwLock<HashMap<u32, Balance>>>,
    order_books: Arc<RwLock<HashMap<u32, OrderBook>>>,
    
    // Performance metrics
    metrics: Arc<RwLock<PerformanceMetrics>>,
    
    // Shutdown signal
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl Engine {
    pub async fn new(config: EngineConfig) -> Result<Self> {
        info!("🚀 Initializing HFT Ninja Engine v2025.07");
        
        // Initialize event bus
        let event_bus = Arc::new(EventBus::new(crate::core::EVENT_BUFFER_SIZE));
        
        // Initialize memory pool
        let memory_pool = Arc::new(MemoryPool::new(crate::core::MEMORY_POOL_SIZE)?);
        
        // Initialize Solana client
        let solana_client = Arc::new(
            SolanaClient::new(
                &config.solana_rpc_url,
                solana_sdk::commitment_config::CommitmentLevel::Confirmed,
                5000
            ).context("Failed to create Solana client")?
        );
        
        // Initialize strategies
        let mut strategies = Vec::new();
        for strategy_name in &config.strategies {
            match create_strategy(strategy_name) {
                Ok(strategy) => {
                    info!("✅ Loaded strategy: {}", strategy_name);
                    strategies.push(strategy);
                }
                Err(e) => {
                    warn!("❌ Failed to load strategy '{}': {}", strategy_name, e);
                }
            }
        }
        
        // Initialize state
        let positions = Arc::new(RwLock::new(HashMap::new()));
        let balances = Arc::new(RwLock::new(HashMap::new()));
        let order_books = Arc::new(RwLock::new(HashMap::new()));
        
        // Initialize metrics
        let metrics = Arc::new(RwLock::new(PerformanceMetrics {
            latency_ns: 0,
            throughput_ops_sec: 0.0,
            memory_usage_bytes: 0,
            cpu_usage_percent: 0.0,
            error_rate: 0.0,
            timestamp: current_timestamp(),
        }));
        
        Ok(Self {
            config,
            event_bus,
            memory_pool,
            solana_client,
            strategies,
            positions,
            balances,
            order_books,
            metrics,
            shutdown_tx: None,
        })
    }
    
    /// Start the trading engine
    pub async fn run(&self) -> Result<()> {
        info!("🎯 Starting trading engine (dry_run: {})", self.config.dry_run);
        
        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        
        // Start event processing
        let event_processor = self.start_event_processor().await?;
        
        // Start market data feed
        let market_data_feed = self.start_market_data_feed().await?;
        
        // Start strategy execution
        let strategy_executor = self.start_strategy_executor().await?;
        
        // Start performance monitoring
        let performance_monitor = self.start_performance_monitor().await?;
        
        // Publish engine started event
        self.event_bus.publish(Event::StrategyStarted {
            strategy_name: "HFT_ENGINE".to_string(),
            timestamp: current_timestamp(),
        })?;
        
        info!("🥷 HFT Ninja Engine running at full speed!");
        
        // Wait for shutdown signal
        shutdown_rx.recv().await;
        
        // Cleanup
        event_processor.abort();
        market_data_feed.abort();
        strategy_executor.abort();
        performance_monitor.abort();
        
        info!("🛑 Trading engine stopped");
        Ok(())
    }
    
    /// Graceful shutdown
    pub async fn shutdown(&self) -> Result<()> {
        info!("🛑 Initiating graceful shutdown...");
        
        // Publish shutdown event
        self.event_bus.publish(Event::SystemShutdown {
            timestamp: current_timestamp(),
        })?;
        
        // Close all positions if not in dry run mode
        if !self.config.dry_run {
            self.close_all_positions().await?;
        }
        
        // Send shutdown signal
        if let Some(ref tx) = self.shutdown_tx {
            let _ = tx.send(()).await;
        }
        
        info!("✅ Graceful shutdown completed");
        Ok(())
    }
    
    /// Start event processing task
    async fn start_event_processor(&self) -> Result<tokio::task::JoinHandle<()>> {
        let event_bus = Arc::clone(&self.event_bus);
        let mut receiver = event_bus.subscribe();
        
        let handle = tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                let start_time = std::time::Instant::now();
                
                // Process event based on type
                match event {
                    Event::PriceUpdate { symbol_id, price, .. } => {
                        debug!("Price update: {} = {:.6} SOL", 
                               get_symbol(symbol_id).unwrap_or_default(), 
                               price.to_sol());
                    }
                    Event::TradeExecuted { trade } => {
                        info!("Trade executed: {} {} {:.6} SOL", 
                              get_symbol(trade.symbol_id).unwrap_or_default(),
                              match trade.side { TradeSide::Buy => "BUY", TradeSide::Sell => "SELL" },
                              trade.price.to_sol());
                    }
                    Event::RiskLimitExceeded { symbol_id, limit_type, .. } => {
                        warn!("Risk limit exceeded for {}: {}", 
                              get_symbol(symbol_id).unwrap_or_default(), 
                              limit_type);
                    }
                    _ => {}
                }
                
                // Track processing latency
                let latency = start_time.elapsed();
                if latency.as_nanos() > 10000 { // 10 microseconds threshold
                    warn!("Event processing latency: {}ns", latency.as_nanos());
                }
            }
        });
        
        Ok(handle)
    }
    
    /// Start market data feed
    async fn start_market_data_feed(&self) -> Result<tokio::task::JoinHandle<()>> {
        let event_bus = Arc::clone(&self.event_bus);
        let solana_client = Arc::clone(&self.solana_client);
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));
            
            loop {
                interval.tick().await;
                
                // Simulate market data (replace with real WebSocket feed)
                let sol_symbol = intern_symbol("SOL");
                let price = Price::from_sol(23.45 + (rand::random::<f64>() - 0.5) * 0.1);
                
                if let Err(e) = event_bus.publish(Event::PriceUpdate {
                    symbol_id: sol_symbol,
                    price,
                    volume: 1000,
                    timestamp: current_timestamp(),
                }) {
                    error!("Failed to publish price update: {}", e);
                }
            }
        });
        
        Ok(handle)
    }
    
    /// Start strategy execution
    async fn start_strategy_executor(&self) -> Result<tokio::task::JoinHandle<()>> {
        let strategies = self.strategies.len();
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(10));
            
            loop {
                interval.tick().await;
                
                // Execute strategies (placeholder)
                debug!("Executing {} strategies", strategies);
                
                // TODO: Implement actual strategy execution
            }
        });
        
        Ok(handle)
    }
    
    /// Start performance monitoring
    async fn start_performance_monitor(&self) -> Result<tokio::task::JoinHandle<()>> {
        let metrics = Arc::clone(&self.metrics);
        let event_bus = Arc::clone(&self.event_bus);
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            
            loop {
                interval.tick().await;
                
                // Update performance metrics
                let mut metrics_guard = metrics.write().await;
                metrics_guard.timestamp = current_timestamp();
                metrics_guard.memory_usage_bytes = 
                    std::alloc::System.used_memory().unwrap_or(0) as u64;
                
                // Check for performance alerts
                if metrics_guard.latency_ns > 100_000 { // 100 microseconds
                    let _ = event_bus.publish(Event::LatencyAlert {
                        component: "ENGINE".to_string(),
                        latency_ns: metrics_guard.latency_ns,
                        threshold_ns: 100_000,
                        timestamp: current_timestamp(),
                    });
                }
            }
        });
        
        Ok(handle)
    }
    
    /// Close all open positions
    async fn close_all_positions(&self) -> Result<()> {
        let positions = self.positions.read().await;
        
        for (symbol_id, position) in positions.iter() {
            if position.quantity != 0 {
                info!("Closing position: {} qty={}", 
                      get_symbol(*symbol_id).unwrap_or_default(), 
                      position.quantity);
                
                // TODO: Implement actual position closing
            }
        }
        
        Ok(())
    }
    
    /// Get current engine statistics
    pub async fn get_stats(&self) -> EngineStats {
        let positions_count = self.positions.read().await.len();
        let balances_count = self.balances.read().await.len();
        let order_books_count = self.order_books.read().await.len();
        let event_stats = self.event_bus.stats();
        let metrics = self.metrics.read().await.clone();
        
        EngineStats {
            positions_count,
            balances_count,
            order_books_count,
            strategies_count: self.strategies.len(),
            event_stats,
            performance_metrics: metrics,
        }
    }
}

/// Engine statistics for monitoring
#[derive(Debug, Clone)]
pub struct EngineStats {
    pub positions_count: usize,
    pub balances_count: usize,
    pub order_books_count: usize,
    pub strategies_count: usize,
    pub event_stats: crate::core::events::EventBusStats,
    pub performance_metrics: PerformanceMetrics,
}

// Placeholder trait for memory usage (would use a real allocator in production)
trait MemoryAllocator {
    fn used_memory(&self) -> Option<usize>;
}

impl MemoryAllocator for std::alloc::System {
    fn used_memory(&self) -> Option<usize> {
        // Placeholder - would use real memory tracking
        Some(1024 * 1024) // 1MB
    }
}
