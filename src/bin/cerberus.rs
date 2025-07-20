use anyhow::Result;
use clap::{Arg, Command};
use solana_hft_ninja::cerberus::{CerberusBrain, CerberusConfig, PositionState};
use std::env;
use tracing::{info, error, warn};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("cerberus=debug,solana_hft_ninja=info")
        .init();

    let matches = Command::new("Cerberus Trade Execution Brain")
        .version("2025.7.0")
        .author("HFT Ninja Team")
        .about("🧠 Autonomous position management system for Solana HFT")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path")
                .default_value("config/cerberus.toml"),
        )
        .arg(
            Arg::new("quicknode")
                .long("quicknode")
                .value_name("URL")
                .help("QuickNode endpoint URL"),
        )
        .arg(
            Arg::new("helius")
                .long("helius")
                .value_name("URL")
                .help("Helius endpoint URL"),
        )
        .arg(
            Arg::new("redis")
                .long("redis")
                .value_name("URL")
                .help("Redis/DragonflyDB URL")
                .default_value("redis://127.0.0.1:6379"),
        )
        .arg(
            Arg::new("jito")
                .long("jito")
                .value_name("URL")
                .help("Jito block engine URL")
                .default_value("https://mainnet.block-engine.jito.wtf"),
        )
        .arg(
            Arg::new("interval")
                .long("interval")
                .value_name("MS")
                .help("Decision loop interval in milliseconds")
                .default_value("200"),
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .help("Dry run mode (no actual transactions)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("test-position")
                .long("test-position")
                .value_name("MINT")
                .help("Create a test position for the given mint"),
        )
        .get_matches();

    // Build configuration
    let mut config = CerberusConfig::default();

    // Check environment variables first, then CLI args
    if let Ok(quicknode) = env::var("QUICKNODE_ENDPOINT") {
        config.quicknode_endpoint = quicknode;
    } else if let Some(quicknode) = matches.get_one::<String>("quicknode") {
        config.quicknode_endpoint = quicknode.clone();
    }

    if let Ok(helius) = env::var("HELIUS_ENDPOINT") {
        config.helius_endpoint = helius;
    } else if let Some(helius) = matches.get_one::<String>("helius") {
        config.helius_endpoint = helius.clone();
    }
    
    if let Some(redis) = matches.get_one::<String>("redis") {
        config.redis_url = redis.clone();
    }
    
    if let Some(jito) = matches.get_one::<String>("jito") {
        config.jito_endpoint = jito.clone();
    }
    
    if let Some(interval) = matches.get_one::<String>("interval") {
        config.loop_interval_ms = interval.parse().unwrap_or(200);
    }

    let dry_run = matches.get_flag("dry-run");

    // Print startup banner
    print_banner(&config, dry_run);

    // Validate environment
    validate_environment().await?;

    // Initialize Cerberus
    let cerberus = CerberusBrain::new(config).await?;

    // Handle test position creation
    if let Some(mint) = matches.get_one::<String>("test-position") {
        create_test_position(&cerberus, mint).await?;
        return Ok(());
    }

    // Start the decision loop
    info!("🚀 Starting Cerberus Trade Execution Brain");
    
    if dry_run {
        warn!("🧪 Running in DRY RUN mode - no actual transactions will be executed");
    }

    // Handle Ctrl+C gracefully
    let cerberus_clone = cerberus.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        warn!("🛑 Received Ctrl+C, initiating graceful shutdown...");
        
        if let Err(e) = cerberus_clone.emergency_stop("USER_SHUTDOWN").await {
            error!("Failed to execute emergency stop: {}", e);
        }
        
        std::process::exit(0);
    });

    // Start the main loop
    cerberus.start_decision_loop().await?;

    Ok(())
}

fn print_banner(config: &CerberusConfig, dry_run: bool) {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                🧠 CERBERUS TRADE EXECUTION BRAIN 🧠           ║");
    println!("║                     Solana HFT Ninja 2025.07                 ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ 🎯 Mission: Autonomous position management                   ║");
    println!("║ ⚡ Speed: Sub-second decision making                         ║");
    println!("║ 🛡️ Safety: Multi-layer risk management                      ║");
    println!("║ 🤖 Intelligence: AI-driven + hard rules                     ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ 📊 Primary RPC: QuickNode                                   ║");
    println!("║ 📊 Fallback RPC: Helius                                     ║");
    println!("║ 🗄️ Store: Redis/DragonflyDB                                 ║");
    println!("║ 🚀 Execution: Jito Bundles                                  ║");
    println!("║ ⏱️ Loop: {}ms intervals                                    ║", config.loop_interval_ms);
    if dry_run {
        println!("║ 🧪 Mode: DRY RUN (Safe Testing)                            ║");
    } else {
        println!("║ 💰 Mode: LIVE TRADING (Real Money)                         ║");
    }
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
}

async fn validate_environment() -> Result<()> {
    info!("🔍 Validating environment...");

    // Check for wallet keypair
    if env::var("SOLANA_PRIVATE_KEY").is_err() {
        let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let keypair_path = format!("{}/.config/solana/id.json", home_dir);
        
        if !std::path::Path::new(&keypair_path).exists() {
            return Err(anyhow::anyhow!(
                "❌ No wallet keypair found. Set SOLANA_PRIVATE_KEY environment variable or ensure ~/.config/solana/id.json exists"
            ));
        }
    }

    // Check for premium endpoints
    if env::var("QUICKNODE_ENDPOINT").is_err() {
        warn!("⚠️ QUICKNODE_ENDPOINT not set, using default mainnet RPC");
    }

    if env::var("HELIUS_ENDPOINT").is_err() {
        warn!("⚠️ HELIUS_ENDPOINT not set, using default mainnet RPC");
    }

    info!("✅ Environment validation passed");
    Ok(())
}

async fn create_test_position(cerberus: &CerberusBrain, mint: &str) -> Result<()> {
    info!("🧪 Creating test position for mint: {}", mint);

    let test_position = PositionState::new(
        mint.to_string(),
        0.001, // Entry price: 0.001 SOL
        0.1,   // Position size: 0.1 SOL
        "test-strategy".to_string(),
        "test-wallet".to_string(),
    );

    cerberus.store.store_position(&test_position).await?;

    info!("✅ Test position created:");
    info!("   Mint: {}", test_position.mint);
    info!("   Entry Price: {} SOL", test_position.entry_price);
    info!("   Position Size: {} SOL", test_position.position_size_sol);
    info!("   Take Profit: {}%", test_position.take_profit_target_percent);
    info!("   Stop Loss: {}%", test_position.stop_loss_target_percent);
    info!("   Timeout: {} seconds", test_position.timeout_seconds);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = CerberusConfig::default();
        assert_eq!(config.loop_interval_ms, 200);
        assert_eq!(config.max_concurrent_positions, 50);
        assert_eq!(config.default_timeout_seconds, 600);
        assert!(config.emergency_stop_enabled);
    }
}
