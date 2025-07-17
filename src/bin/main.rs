use anyhow::Result;
use clap::Parser;
use solana_hft_ninja::{config::Config, engine::Engine, mempool::*, mempool::listener::HeliusConfig};
use tokio::sync::mpsc;
use tracing::{info, error, warn};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "hft-ninja")]
#[command(about = "Zero-cost Solana High-Frequency Trading Engine")]
struct Args {
    #[arg(short, long, default_value = "./config")]
    config_path: String,
    
    #[arg(short, long)]
    dry_run: bool,
    
    #[arg(long)]
    enable_mempool: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    info!("Starting Solana HFT Ninja 2025.07...");
    
    // Load configuration
    let config = Config::load(&args.config_path)?;

    // Initialize mempool router channel
    let mempool_rx = init_mempool_channel();
    info!("Mempool router initialized");

    // Initialize engine
    let mut engine = Engine::new(config, args.dry_run).await?;

    // Start mempool listener if enabled
    let mempool_handle = if args.enable_mempool {
        info!("Starting enhanced mempool listener with MEV detection...");
        Some(start_enhanced_mempool_listener().await?)
    } else {
        warn!("Mempool listener disabled - MEV opportunities will be missed!");
        None
    };

    // Start trading engine with mempool integration
    info!("Starting HFT Engine with real-time MEV capabilities...");
    if let Err(e) = engine.run_with_mempool(mempool_rx).await {
        error!("Engine error: {}", e);
        return Err(e);
    }

    // Wait for mempool listener to finish
    if let Some(handle) = mempool_handle {
        handle.await?;
    }
    
    Ok(())
}

/// Start enhanced mempool listener with MEV detection
async fn start_enhanced_mempool_listener() -> Result<tokio::task::JoinHandle<()>> {
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Configure Helius connection (fallback to demo mode if no API key)
    let config = if let Ok(api_key) = std::env::var("HELIUS_KEY") {
        info!("🔑 Using Helius API key for real data");
        crate::mempool::helius::HeliusConfig {
            api_key,
            endpoint: "wss://mainnet.helius-rpc.com".to_string(),
            reconnect_interval: std::time::Duration::from_secs(5),
            ping_interval: std::time::Duration::from_secs(30),
            max_reconnect_attempts: 10,
        }
    } else {
        warn!("⚠️  No HELIUS_KEY found, using demo mode");
        crate::mempool::helius::HeliusConfig::default()
    };

    // Create metrics and parser
    let metrics = MempoolMetrics::new();
    let parser = ZeroCopyParser::new(metrics.clone(), 16 * 1024 * 1024);

    // Create opportunity detector
    let opportunity_detector = OpportunityDetector::new();

    // Create listener
    let listener = MempoolListener::new(config, parser, metrics, tx);

    // Start MEV processing task
    let processing_handle = tokio::spawn(async move {
        let mut transaction_count = 0;
        let mut dex_count = 0;
        let mut mev_opportunities = 0;

        while let Some(parsed_tx) = rx.recv().await {
            transaction_count += 1;
            dex_count += parsed_tx.dex_interactions.len();

            // Detect MEV opportunities
            let opportunities = opportunity_detector.detect_opportunities(&parsed_tx);
            mev_opportunities += opportunities.len();

            // Send opportunities to engine via router
            for opportunity in opportunities {
                if let Err(e) = send_mempool_event(opportunity) {
                    error!("Failed to send MEV opportunity: {}", e);
                }
            }

            // Log progress every 100 transactions
            if transaction_count % 100 == 0 {
                info!(
                    "📊 Processed {} transactions | {} DEX interactions | {} MEV opportunities detected",
                    transaction_count, dex_count, mev_opportunities
                );
            }

            // Log individual DEX interactions for debugging
            for interaction in &parsed_tx.dex_interactions {
                info!(
                    "🔍 DEX: {} - {:?} at slot {} | Accounts: {}",
                    interaction.program.name(),
                    interaction.instruction_type,
                    parsed_tx.slot,
                    interaction.accounts.len()
                );
            }
        }
    });

    // Start listener in background
    let listener_handle = tokio::spawn(async move {
        if let Err(e) = listener.start().await {
            error!("Mempool listener error: {}", e);
        }
    });

    info!("🚀 Enhanced mempool listener started with MEV detection");
    Ok(listener_handle)
}

/// Legacy mempool listener (kept for backward compatibility)
async fn start_mempool_listener() -> Result<tokio::task::JoinHandle<()>> {
    warn!("Using legacy mempool listener - consider upgrading to enhanced version");
    start_enhanced_mempool_listener().await
}