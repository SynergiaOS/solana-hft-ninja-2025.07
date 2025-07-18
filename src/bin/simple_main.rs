//! Simple Main - Working Bridge Implementation
//! 
//! This is a minimal working version that demonstrates the mempool→engine bridge.

use anyhow::Result;
use clap::Parser;
use solana_hft_ninja::{config::Config, bridge::*, simple_engine::*, monitoring::{create_metrics, start_metrics_collection, MetricsServer}};
use tracing::{info, error, warn};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config")]
    config_path: String,

    /// Enable dry run mode (no actual trades)
    #[arg(long)]
    dry_run: bool,

    /// Enable mempool listener
    #[arg(long, default_value = "true")]
    enable_mempool: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("🚀 Solana HFT Ninja 2025.07 - Simple Bridge Demo");
    info!("================================================");
    
    let args = Args::parse();
    
    // Load configuration (with fallback)
    info!("📁 Loading configuration from: {}", args.config_path);
    let config = match Config::load(&args.config_path) {
        Ok(config) => {
            info!("✅ Configuration loaded successfully");
            config
        },
        Err(e) => {
            warn!("Failed to load config: {}, using defaults", e);
            create_default_config()
        }
    };

    info!("🔧 Configuration:");
    info!("   - RPC URL: {}", config.solana.rpc_url);
    info!("   - Wallet path: {}", config.wallet.keypair_path);
    
    // Initialize bridge
    let bridge_rx = init_bridge();
    info!("🌉 Bridge initialized successfully");
    
    // Initialize metrics
    info!("📊 Initializing metrics system...");
    let metrics = create_metrics()?;
    let metrics_server = MetricsServer::new(metrics.clone(), 8080);

    // Start metrics collection
    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        start_metrics_collection(metrics_clone).await;
    });

    // Start metrics server
    let metrics_server_clone = metrics_server;
    tokio::spawn(async move {
        if let Err(e) = metrics_server_clone.start().await {
            error!("Failed to start metrics server: {}", e);
        }
    });

    // Initialize simple engine
    info!("🏗️  Creating HFT Engine...");
    let mut engine = SimpleEngine::new(config, args.dry_run).await?;

    // Set metrics in engine
    engine.set_metrics(metrics.clone());

    info!("✅ Engine created successfully");
    info!("🔑 Wallet: {}", engine.wallet_pubkey());
    info!("📊 Metrics server started on http://localhost:8080/metrics");
    
    // Start mempool listener if enabled
    let mempool_handle = if args.enable_mempool {
        info!("🎧 Starting bridge mempool listener...");
        Some(start_bridge_mempool_listener().await?)
    } else {
        warn!("⚠️  Mempool listener disabled - no real-time events");
        None
    };
    
    // Print startup summary
    info!("📋 Startup Summary:");
    info!("   - Config path: {}", args.config_path);
    info!("   - Dry run: {}", args.dry_run);
    info!("   - Mempool enabled: {}", args.enable_mempool);
    info!("   - Bridge status: ✅ Active");
    
    // Start the engine with bridge integration
    info!("🎯 Starting HFT Engine with bridge integration...");
    if let Err(e) = engine.run_with_bridge(bridge_rx).await {
        error!("❌ Engine error: {}", e);
        return Err(e);
    }
    
    // Wait for mempool listener to finish (this won't happen in normal operation)
    if let Some(handle) = mempool_handle {
        handle.await?;
    }
    
    Ok(())
}

/// Create default configuration for testing
fn create_default_config() -> Config {
    use solana_hft_ninja::config::*;

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
        wallet_tracker: None, // Disabled by default
    }
}
