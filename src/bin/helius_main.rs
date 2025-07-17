//! Solana HFT Ninja 2025.07 - Helius Real Data Integration
//! 
//! Real-time mempool monitoring with Helius WebSocket API

use anyhow::Result;
use clap::Parser;
use solana_hft_ninja::{
    config::*,
    mempool::helius::{HeliusConfig, start_helius_listener, TransactionNotification},
    bridge::*,
    simple_engine::SimpleEngine
};
use tracing::{info, warn, error, debug};

#[derive(Parser, Debug)]
#[command(name = "helius-hft")]
#[command(about = "Solana HFT Ninja with Helius real data integration")]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config")]
    config: String,
    
    /// Enable dry run mode (no actual trading)
    #[arg(long)]
    dry_run: bool,
    
    /// Helius API key (can also be set via HELIUS_KEY env var)
    #[arg(long)]
    helius_key: Option<String>,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("solana_hft_ninja={},helius_main={}", log_level, log_level))
        .init();

    info!("🚀 Solana HFT Ninja 2025.07 - Helius Real Data Integration");
    info!("================================================================");

    // Get Helius API key
    let api_key = args.helius_key
        .or_else(|| std::env::var("HELIUS_KEY").ok())
        .or_else(|| std::env::var("HELIUS_API_KEY").ok());

    if api_key.is_none() {
        error!("❌ No Helius API key provided!");
        error!("   Set HELIUS_KEY environment variable or use --helius-key flag");
        error!("   Get your free API key at: https://helius.xyz");
        std::process::exit(1);
    }

    let api_key = api_key.unwrap();
    info!("🔑 Helius API key configured");

    // Load configuration
    let config = Config::load(&args.config).unwrap_or_else(|e| {
        warn!("⚠️  Could not load config ({}), using defaults", e);
        Config {
            solana: SolanaConfig {
                rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                ws_url: "wss://api.mainnet-beta.solana.com".to_string(),
                rpc_timeout_ms: 5000,
            },
            wallet: WalletConfig {
                private_key_path: "wallet.json".to_string(),
                keypair_path: "keypair.json".to_string(),
            },
            trading: TradingConfig {
                initial_balance_sol: 1.0,
                max_position_size_sol: 0.1,
                max_slippage_bps: 100,
                min_profit_threshold_bps: 10,
                risk_limit_bps: 500,
            },
            strategy: StrategyConfig {
                strategy_mode: "market_making".to_string(),
                update_interval_ms: 100,
                order_book_depth: 10,
                spread_bps: 50,
            },
            risk: RiskConfig {
                stop_loss_bps: 200,
                take_profit_bps: 100,
                max_drawdown_bps: 1000,
                risk_limit_bps: 500,
            },
            logging: LoggingConfig {
                rust_log: "info".to_string(),
                log_level: "info".to_string(),
                log_file_path: "hft.log".to_string(),
            },
            monitoring: MonitoringConfig {
                metrics_port: 8080,
                health_check_interval_ms: 1000,
                enable_ddos_protection: false,
                rate_limit_rps: 1000,
            },
        }
    });

    // Configure Helius
    let helius_config = HeliusConfig {
        api_key,
        endpoint: "wss://mainnet.helius-rpc.com".to_string(),
        reconnect_interval: std::time::Duration::from_secs(5),
        ping_interval: std::time::Duration::from_secs(30),
        max_reconnect_attempts: 10,
    };

    // Initialize bridge
    let bridge_rx = init_bridge();
    info!("🌉 Bridge initialized successfully");

    // Initialize simple engine
    let mut engine = SimpleEngine::new(config.clone(), args.dry_run).await?;
    info!("⚙️  Simple engine initialized");

    // Start Helius listener
    info!("📡 Starting Helius WebSocket listener...");
    let mut helius_rx = start_helius_listener(helius_config).await?;
    info!("✅ Helius listener started");

    // Start bridge event processing in background
    info!("🎧 Starting bridge event processing...");
    tokio::spawn(async move {
        if let Err(e) = engine.run_with_bridge(bridge_rx).await {
            error!("Bridge processing failed: {}", e);
        }
    });

    info!("📋 Startup Summary:");
    info!("   - Config path: {}", args.config);
    info!("   - Dry run: {}", args.dry_run);
    info!("   - Helius integration: ✅ Active");
    info!("   - Bridge status: ✅ Active");

    info!("🎯 Starting real-time Solana mempool monitoring...");

    // Main event loop - process Helius transactions
    let mut transaction_count = 0u64;
    let start_time = std::time::Instant::now();

    while let Some(notification) = helius_rx.recv().await {
        transaction_count += 1;
        
        // Process transaction through bridge
        if let Err(e) = process_helius_transaction(notification).await {
            debug!("Error processing transaction: {}", e);
        }

        // Log progress every 100 transactions
        if transaction_count % 100 == 0 {
            let elapsed = start_time.elapsed();
            let tps = transaction_count as f64 / elapsed.as_secs_f64();
            info!("📊 Processed {} real transactions ({:.1} TPS)", transaction_count, tps);
        }
    }

    Ok(())
}

/// Process Helius transaction notification through bridge
async fn process_helius_transaction(
    notification: TransactionNotification,
) -> Result<()> {
    debug!("Processing Helius transaction: {}", notification.signature);

    // Convert Helius notification to bridge event
    let bridge_event = BridgeEvent {
        event_type: EventType::DexTransaction {
            signature: notification.signature,
            program: "Unknown".to_string(), // We'll detect this later
            accounts: vec![], // Extract from transaction data
        },
        timestamp: notification.block_time.unwrap_or(0) as u64,
        priority: 1, // Normal priority
    };

    // Send through bridge
    send_bridge_event(bridge_event)?;

    Ok(())
}
