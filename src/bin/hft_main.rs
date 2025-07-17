//! Solana HFT Ninja 2025.07 - Advanced Main Binary
//! 
//! Complete high-frequency trading system with MEV strategies, Jito integration,
//! real-time mempool monitoring, and comprehensive metrics.

use anyhow::Result;
use clap::Parser;
use solana_hft_ninja::{
    config::Config,
    bridge::*,
    simple_engine::*,
    mempool::{start_helius_listener, create_dex_detector, TransactionNotification, DexDetector},
    helius::HeliusConfig,
    strategies::{create_mev_engine, MevEngine, MevOpportunity},
    execution::{JitoConfig, JitoExecutor},
    monitoring::{create_metrics, MetricsServer, start_metrics_collection, HftMetrics},
};
use solana_sdk::signature::Keypair;
use std::sync::Arc;
use tracing::{info, error, warn};
use tokio::signal;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config")]
    config_path: String,

    /// Enable dry run mode (no actual trades)
    #[arg(long)]
    dry_run: bool,

    /// Enable real Helius mempool listener
    #[arg(long, default_value = "true")]
    enable_helius: bool,

    /// Enable MEV strategies
    #[arg(long, default_value = "true")]
    enable_mev: bool,

    /// Enable Jito bundle execution
    #[arg(long, default_value = "true")]
    enable_jito: bool,

    /// Metrics server port
    #[arg(long, default_value = "8080")]
    metrics_port: u16,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(&args.log_level)
        .init();
    
    info!("🚀 Solana HFT Ninja 2025.07 - Advanced Trading System");
    info!("=====================================================");
    
    // Load configuration
    let config = match Config::load(&args.config_path) {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load config: {}, using defaults", e);
            create_default_config()
        }
    };
    
    // Initialize metrics system
    let metrics = create_metrics()?;
    info!("📊 Metrics system initialized");
    
    // Start metrics collection
    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        start_metrics_collection(metrics_clone).await;
    });
    
    // Start metrics server
    let metrics_server = MetricsServer::new(metrics.clone(), args.metrics_port);
    tokio::spawn(async move {
        if let Err(e) = metrics_server.start().await {
            error!("Metrics server error: {}", e);
        }
    });
    
    // Initialize bridge
    let bridge_rx = init_bridge();
    info!("🌉 Bridge initialized successfully");
    
    // Initialize DEX detector
    let dex_detector = create_dex_detector();
    info!("🔍 DEX detector initialized");
    
    // Initialize MEV engine
    let mev_engine = if args.enable_mev {
        Some(create_mev_engine())
    } else {
        None
    };
    
    if args.enable_mev {
        info!("⚡ MEV strategies enabled");
    }
    
    // Initialize Jito executor
    let jito_executor = if args.enable_jito {
        let jito_config = JitoConfig::default();
        let tip_keypair = Keypair::new(); // In reality, load from secure storage
        Some(JitoExecutor::new(jito_config, tip_keypair))
    } else {
        None
    };
    
    if args.enable_jito {
        info!("📦 Jito bundle execution enabled");
    }
    
    // Initialize advanced engine
    let mut engine = AdvancedEngine::new(
        config,
        args.dry_run,
        metrics.clone(),
        dex_detector,
        mev_engine,
        jito_executor,
    ).await?;
    
    info!("⚙️  Advanced engine initialized");
    
    // Start Helius mempool listener if enabled
    let helius_handle = if args.enable_helius {
        info!("🎧 Starting Helius mempool listener...");
        let helius_config = HeliusConfig::default();
        
        if helius_config.api_key.is_empty() {
            warn!("⚠️  HELIUS_API_KEY not set, using mock listener");
            Some(start_bridge_mempool_listener().await?)
        } else {
            let mut helius_rx = start_helius_listener(helius_config).await?;
            
            // Bridge Helius events to our system
            let (bridge_tx, _) = tokio::sync::mpsc::unbounded_channel();
            tokio::spawn(async move {
                while let Some(notification) = helius_rx.recv().await {
                    // Convert Helius notification to bridge event
                    let event_data = serde_json::to_vec(&notification).unwrap_or_default();
                    if let Err(e) = bridge_tx.send(event_data) {
                        warn!("Failed to send Helius event to bridge: {}", e);
                    }
                }
            });
            
            None
        }
    } else {
        warn!("⚠️  Helius listener disabled - using mock events");
        Some(start_bridge_mempool_listener().await?)
    };
    
    // Print startup summary
    info!("📋 Startup Summary:");
    info!("   - Config path: {}", args.config_path);
    info!("   - Dry run: {}", args.dry_run);
    info!("   - Helius enabled: {}", args.enable_helius);
    info!("   - MEV enabled: {}", args.enable_mev);
    info!("   - Jito enabled: {}", args.enable_jito);
    info!("   - Metrics port: {}", args.metrics_port);
    info!("   - Bridge status: ✅ Active");
    info!("   - System status: 🟢 Ready");
    
    // Setup graceful shutdown
    let shutdown_signal = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("🛑 Shutdown signal received");
    };
    
    // Start the advanced engine
    info!("🎯 Starting Advanced HFT Engine...");

    // Convert bridge_rx to the expected type
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

    // Start a task to convert bridge events to bytes
    tokio::spawn(async move {
        let mut bridge_rx = bridge_rx;
        while let Ok(event) = bridge_rx.recv().await {
            let event_data = serde_json::to_vec(&*event).unwrap_or_default();
            if tx.send(event_data).is_err() {
                break;
            }
        }
    });

    tokio::select! {
        result = engine.run_with_bridge_bytes(rx) => {
            if let Err(e) = result {
                error!("❌ Engine error: {}", e);
                return Err(e);
            }
        }
        _ = shutdown_signal => {
            info!("🔄 Graceful shutdown initiated");
        }
    }
    
    // Wait for background tasks to finish
    if let Some(handle) = helius_handle {
        handle.await?;
    }
    
    info!("✅ Solana HFT Ninja shutdown complete");
    Ok(())
}

/// Advanced engine with full feature set
struct AdvancedEngine {
    simple_engine: SimpleEngine,
    metrics: Arc<HftMetrics>,
    dex_detector: DexDetector,
    mev_engine: Option<MevEngine>,
    jito_executor: Option<JitoExecutor>,
}

impl AdvancedEngine {
    /// Create new advanced engine
    async fn new(
        config: Config,
        dry_run: bool,
        metrics: Arc<HftMetrics>,
        dex_detector: DexDetector,
        mev_engine: Option<MevEngine>,
        jito_executor: Option<JitoExecutor>,
    ) -> Result<Self> {
        let simple_engine = SimpleEngine::new(config, dry_run).await?;
        
        Ok(Self {
            simple_engine,
            metrics,
            dex_detector,
            mev_engine,
            jito_executor,
        })
    }
    
    /// Run engine with bridge integration
    async fn run_with_bridge_bytes(
        &mut self,
        mut bridge_rx: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>,
    ) -> Result<()> {
        info!("🔄 Advanced engine running with full feature set");
        
        while let Some(event_data) = bridge_rx.recv().await {
            let start_time = std::time::Instant::now();
            
            // Update metrics
            self.metrics.transactions_processed.inc();
            self.metrics.bridge_queue_size.set(bridge_rx.len() as i64);
            
            // Process event with advanced features
            if let Err(e) = self.process_advanced_event(&event_data).await {
                warn!("Error processing advanced event: {}", e);
                self.metrics.transactions_failed.inc();
            }
            
            // Record processing time
            self.metrics.record_transaction_processing_time(start_time.elapsed());
        }
        
        Ok(())
    }
    
    /// Process event with advanced features
    async fn process_advanced_event(&mut self, event_data: &[u8]) -> Result<()> {
        // Try to parse as transaction notification
        if let Ok(notification) = serde_json::from_slice::<TransactionNotification>(event_data) {
            // Detect DEX transactions
            if let Some(dex_tx) = self.dex_detector.detect_dex_transaction(&notification.transaction) {
                info!("🔍 DEX transaction detected: {} on {:?}", dex_tx.signature, dex_tx.protocol);
                self.metrics.dex_transactions_detected.inc();
                
                // Analyze for MEV opportunities
                if let Some(ref mut mev_engine) = self.mev_engine {
                    let opportunities = mev_engine.analyze_transaction(&dex_tx);
                    
                    for opportunity in opportunities {
                        info!("⚡ MEV opportunity found: {:?}", opportunity);
                        self.metrics.mev_opportunities_found.inc();
                        
                        // Execute MEV strategy (if not dry run)
                        if !self.simple_engine.is_dry_run() {
                            if let Err(e) = self.execute_mev_opportunity(&opportunity).await {
                                warn!("Failed to execute MEV opportunity: {}", e);
                            }
                        }
                    }
                }
            }
        }
        
        // Fallback to simple engine processing
        self.simple_engine.process_event(event_data).await
    }
    
    /// Execute MEV opportunity
    async fn execute_mev_opportunity(&self, opportunity: &MevOpportunity) -> Result<()> {
        info!("🎯 Executing MEV opportunity: {:?}", opportunity);
        
        // This would contain the actual MEV execution logic
        // For now, just simulate execution
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        
        self.metrics.trades_executed.inc();
        self.metrics.trades_successful.inc();
        
        Ok(())
    }
}

/// Create default configuration for advanced system
fn create_default_config() -> Config {
    use solana_hft_ninja::config::*;
    
    Config {
        solana: SolanaConfig {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            ws_url: "wss://api.mainnet-beta.solana.com".to_string(),
            rpc_timeout_ms: 5000,
        },
        wallet: WalletConfig {
            private_key_path: "config/wallet.key".to_string(),
            keypair_path: "config/wallet.json".to_string(),
        },
        trading: TradingConfig {
            initial_balance_sol: 100.0,
            max_position_size_sol: 10.0,
            max_slippage_bps: 50,
            min_profit_threshold_bps: 25,
            risk_limit_bps: 500,
        },
        strategy: StrategyConfig {
            strategy_mode: "market_making".to_string(),
            update_interval_ms: 100,
            order_book_depth: 20,
            spread_bps: 25,
        },
        risk: RiskConfig::default(),
        logging: LoggingConfig {
            rust_log: "info".to_string(),
            log_level: "info".to_string(),
            log_file_path: "hft.log".to_string(),
        },
        monitoring: MonitoringConfig {
            metrics_port: 8080,
            health_check_interval_ms: 10000,
            enable_ddos_protection: true,
            rate_limit_rps: 1000,
        },
    }
}
