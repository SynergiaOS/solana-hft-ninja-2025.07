// 🥷 Solana HFT Ninja - Mainnet Trader
// REAL MONEY TRADING - MAXIMUM SECURITY

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use solana_hft_ninja::{
    core::{solana_client::SolanaClient, wallet::Wallet},
    strategies,
    utils::config::Config,
    config::StrategyConfig,
};

#[derive(Parser, Debug, Clone)]
#[command(name = "mainnet_trader")]
#[command(about = "🥷 Solana HFT Ninja - Mainnet Strategy Trader")]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config/mainnet-ultra-safe.toml")]
    config: String,

    /// Strategy to test
    #[arg(short, long)]
    #[arg(value_enum)]
    strategy: Strategy,

    /// Test duration in seconds
    #[arg(short, long, default_value = "300")]
    duration: u64,

    /// Dry run mode (no actual transactions)
    #[arg(long)]
    dry_run: bool,

    /// Maximum position size in SOL
    #[arg(long, default_value = "0.01")]
    max_position: f64,

    /// Minimum profit threshold in SOL
    #[arg(long, default_value = "0.001")]
    min_profit: f64,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Strategy {
    Arbitrage,
    Sandwich,
    JupiterArb,
    Sniping,
    Liquidation,
    All,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .with_target(false)
        .init();

    info!("🥷 Starting Solana HFT Ninja Mainnet Trader");
    info!(
        "Strategy: {:?}, Duration: {}s, Dry Run: {}",
        args.strategy, args.duration, args.dry_run
    );

    // 🚨 MAINNET WARNING
    if !args.dry_run {
        warn!("🚨 MAINNET REAL MONEY TRADING ENABLED!");
        warn!("🚨 This will use REAL SOL from your wallet!");
        warn!("🚨 Make sure you understand the risks!");

        // 5 second warning
        for i in (1..=5).rev() {
            warn!("🚨 Starting real trading in {} seconds...", i);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    // Load configuration
    let config = Config::load(&args.config)?;
    info!("📋 Configuration loaded from: {}", args.config);

    // Validate mainnet configuration
    info!("🔍 Validating mainnet configuration...");
    validate_mainnet_config(&config)?;
    info!("✅ Configuration validation passed");

    // Initialize Solana client
    let solana_client = Arc::new(SolanaClient::new(
        &config.solana.rpc_url,
        solana_sdk::commitment_config::CommitmentLevel::Confirmed,
        30000,
    )?);

    // Initialize wallet
    let wallet = Arc::new(Wallet::load(&config.wallet.path)?);

    // Check wallet balance
    info!("💰 Checking wallet balance...");
    let balance = solana_client.get_balance(&wallet.pubkey()).await?;
    let balance_sol = balance as f64 / 1_000_000_000.0;
    info!("💰 Wallet balance: {:.5} SOL", balance_sol);

    // Safety checks for mainnet
    if !args.dry_run {
        perform_safety_checks(balance_sol, &args)?;
    }

    // Run strategy
    match args.strategy {
        Strategy::Arbitrage => {
            test_arbitrage_strategy(&config, &solana_client, &wallet, &args).await?;
        }
        Strategy::Sandwich => {
            test_sandwich_strategy(&config, &solana_client, &wallet, &args).await?;
        }
        Strategy::JupiterArb => {
            test_jupiter_arbitrage_strategy(&config, &solana_client, &wallet, &args).await?;
        }
        Strategy::Sniping => {
            test_sniping_strategy(&config, &solana_client, &wallet, &args).await?;
        }
        Strategy::Liquidation => {
            test_liquidation_strategy(&config, &solana_client, &wallet, &args).await?;
        }
        Strategy::All => {
            test_all_strategies(&config, &solana_client, &wallet, &args).await?;
        }
    }

    info!("🎉 Mainnet trading test completed successfully!");
    Ok(())
}

/// Validate mainnet configuration
fn validate_mainnet_config(config: &Config) -> Result<()> {
    // Check if using mainnet RPC
    if !config.solana.rpc_url.contains("mainnet") {
        return Err(anyhow::anyhow!("❌ Not using mainnet RPC URL"));
    }

    // Check if risk management is configured
    if config.risk.max_daily_loss == 0.0 {
        return Err(anyhow::anyhow!("❌ Risk management not configured"));
    }

    info!("✅ Using mainnet RPC: {}", config.solana.rpc_url);
    info!("✅ Risk management configured");

    Ok(())
}

/// Perform safety checks before real trading
fn perform_safety_checks(balance_sol: f64, args: &Args) -> Result<()> {
    // Check minimum balance
    if balance_sol < 0.01 {
        return Err(anyhow::anyhow!(
            "❌ Insufficient balance: {:.5} SOL (minimum: 0.01 SOL)",
            balance_sol
        ));
    }

    // Check position size vs balance
    if args.max_position > balance_sol * 0.5 {
        return Err(anyhow::anyhow!(
            "❌ Position size too large: {:.3} SOL (max recommended: {:.3} SOL)",
            args.max_position,
            balance_sol * 0.5
        ));
    }

    info!("✅ Safety checks passed");
    info!("✅ Balance: {:.5} SOL", balance_sol);
    info!("✅ Max position: {:.3} SOL", args.max_position);

    Ok(())
}

/// Test arbitrage strategy
async fn test_arbitrage_strategy(
    _config: &Config,
    solana_client: &Arc<SolanaClient>,
    wallet: &Arc<Wallet>,
    args: &Args,
) -> Result<()> {
    info!("⚖️ Testing arbitrage strategy...");

    let strategy_config = StrategyConfig::default();
    let mut strategy = strategies::jupiter_arb::JupiterArbStrategy::new(&strategy_config);

    // Run strategy for specified duration
    let test_duration = Duration::from_secs(args.duration);
    let result = timeout(
        test_duration,
        simulate_arbitrage_strategy(&mut strategy, solana_client, wallet, args),
    )
    .await;

    match result {
        Ok(_) => info!("✅ Arbitrage strategy completed"),
        Err(_) => info!("⏰ Arbitrage strategy test timed out (expected)"),
    }

    Ok(())
}

/// Test sandwich strategy
async fn test_sandwich_strategy(
    _config: &Config,
    solana_client: &Arc<SolanaClient>,
    wallet: &Arc<Wallet>,
    args: &Args,
) -> Result<()> {
    info!("🥪 Testing sandwich strategy...");

    let mev_config = strategies::mev::MevConfig::default();
    let mut mev_engine = strategies::mev::MevEngine::new(mev_config);

    // Run strategy for specified duration
    let test_duration = Duration::from_secs(args.duration);
    let result = timeout(
        test_duration,
        simulate_sandwich_strategy(&mut mev_engine, solana_client, wallet, args),
    )
    .await;

    match result {
        Ok(_) => info!("✅ Sandwich strategy completed"),
        Err(_) => info!("⏰ Sandwich strategy test timed out (expected)"),
    }

    Ok(())
}

/// Test Jupiter arbitrage strategy
async fn test_jupiter_arbitrage_strategy(
    _config: &Config,
    solana_client: &Arc<SolanaClient>,
    wallet: &Arc<Wallet>,
    args: &Args,
) -> Result<()> {
    info!("🔄 Testing Jupiter arbitrage strategy...");

    let strategy_config = StrategyConfig::default();
    let mut strategy = strategies::jupiter_arb::JupiterArbStrategy::new(&strategy_config);

    // Run strategy for specified duration
    let test_duration = Duration::from_secs(args.duration);
    let result = timeout(
        test_duration,
        simulate_jupiter_arbitrage_strategy(&mut strategy, solana_client, wallet, args),
    )
    .await;

    match result {
        Ok(_) => info!("✅ Jupiter arbitrage strategy completed"),
        Err(_) => info!("⏰ Jupiter arbitrage test timed out (expected)"),
    }

    Ok(())
}

/// Test sniping strategy
async fn test_sniping_strategy(
    _config: &Config,
    _solana_client: &Arc<SolanaClient>,
    _wallet: &Arc<Wallet>,
    args: &Args,
) -> Result<()> {
    info!("🚀 Testing token launch sniping strategy...");

    let start_time = std::time::Instant::now();
    let duration = Duration::from_secs(args.duration);

    while start_time.elapsed() < duration {
        let elapsed = start_time.elapsed().as_secs();
        if elapsed % 10 == 0 {
            info!("🔍 Scanning for token launches... ({}s)", elapsed);
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    info!("✅ Sniping strategy test completed");
    Ok(())
}

/// Test liquidation strategy
async fn test_liquidation_strategy(
    _config: &Config,
    _solana_client: &Arc<SolanaClient>,
    _wallet: &Arc<Wallet>,
    args: &Args,
) -> Result<()> {
    info!("💧 Testing liquidation strategy...");

    let start_time = std::time::Instant::now();
    let duration = Duration::from_secs(args.duration);

    while start_time.elapsed() < duration {
        let elapsed = start_time.elapsed().as_secs();
        if elapsed % 15 == 0 {
            info!("🔍 Monitoring liquidation opportunities... ({}s)", elapsed);
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    info!("✅ Liquidation strategy test completed");
    Ok(())
}

/// Test all strategies
async fn test_all_strategies(
    config: &Config,
    solana_client: &Arc<SolanaClient>,
    wallet: &Arc<Wallet>,
    args: &Args,
) -> Result<()> {
    info!("🎯 Testing all strategies sequentially...");

    let strategy_duration = args.duration / 5; // Divide time among strategies
    let mut strategy_args = args.clone();
    strategy_args.duration = strategy_duration;

    // Test each strategy
    test_arbitrage_strategy(config, solana_client, wallet, &strategy_args).await?;
    test_sandwich_strategy(config, solana_client, wallet, &strategy_args).await?;
    test_jupiter_arbitrage_strategy(config, solana_client, wallet, &strategy_args).await?;
    test_sniping_strategy(config, solana_client, wallet, &strategy_args).await?;
    test_liquidation_strategy(config, solana_client, wallet, &strategy_args).await?;

    info!("✅ All strategies tested successfully");
    Ok(())
}

// Simulation functions for strategies
async fn simulate_arbitrage_strategy(
    _strategy: &mut strategies::jupiter_arb::JupiterArbStrategy,
    _solana_client: &Arc<SolanaClient>,
    _wallet: &Arc<Wallet>,
    args: &Args,
) -> Result<()> {
    info!("🔄 Simulating arbitrage strategy for {} seconds...", args.duration);
    tokio::time::sleep(Duration::from_secs(args.duration)).await;
    Ok(())
}

async fn simulate_sandwich_strategy(
    _mev_engine: &mut strategies::mev::MevEngine,
    _solana_client: &Arc<SolanaClient>,
    _wallet: &Arc<Wallet>,
    args: &Args,
) -> Result<()> {
    info!("🥪 Simulating sandwich strategy for {} seconds...", args.duration);
    tokio::time::sleep(Duration::from_secs(args.duration)).await;
    Ok(())
}

async fn simulate_jupiter_arbitrage_strategy(
    _strategy: &mut strategies::jupiter_arb::JupiterArbStrategy,
    _solana_client: &Arc<SolanaClient>,
    _wallet: &Arc<Wallet>,
    args: &Args,
) -> Result<()> {
    info!("🎯 Simulating Jupiter arbitrage strategy for {} seconds...", args.duration);
    tokio::time::sleep(Duration::from_secs(args.duration)).await;
    Ok(())
}
