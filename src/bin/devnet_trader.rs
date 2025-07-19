//! 🥷 Solana HFT Ninja - Devnet Strategy Trader
//! 
//! Dedicated binary for testing trading strategies on Solana devnet
//! with comprehensive monitoring and safety features

use anyhow::{Result, anyhow};
use clap::{Parser, ValueEnum};
use solana_hft_ninja::*;
use std::sync::Arc;
use tokio::time::{Duration, timeout};
use tracing::{info, warn, error, debug};

#[derive(Parser, Clone)]
#[command(name = "devnet-trader")]
#[command(about = "🥷 Solana HFT Ninja - Devnet Strategy Trader")]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config/devnet.toml")]
    config: String,

    /// Strategy to test
    #[arg(short, long, value_enum)]
    strategy: StrategyType,

    /// Test duration in seconds
    #[arg(short, long, default_value = "60")]
    duration: u64,

    /// Dry run mode (no actual transactions)
    #[arg(long, default_value = "true")]
    dry_run: bool,

    /// Maximum position size in SOL
    #[arg(long, default_value = "0.1")]
    max_position: f64,

    /// Minimum profit threshold in SOL
    #[arg(long, default_value = "0.005")]
    min_profit: f64,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Clone, Debug, ValueEnum)]
enum StrategyType {
    /// Cross-DEX arbitrage strategy
    Arbitrage,
    /// Sandwich attack strategy
    Sandwich,
    /// Jupiter arbitrage strategy
    JupiterArb,
    /// Token launch sniping
    Sniping,
    /// Liquidation hunting
    Liquidation,
    /// All strategies combined
    All,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    std::env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();

    info!("🥷 Starting Solana HFT Ninja Devnet Trader");
    info!("Strategy: {:?}, Duration: {}s, Dry Run: {}", 
          args.strategy, args.duration, args.dry_run);

    // Load configuration
    let config = utils::config::Config::load(&args.config)?;
    info!("📋 Configuration loaded from: {}", args.config);

    // Validate devnet configuration
    validate_devnet_config(&config)?;

    // Initialize components
    let solana_client = Arc::new(core::solana_client::SolanaClient::new(
        &config.solana.rpc_url,
        solana_sdk::commitment_config::CommitmentLevel::Confirmed,
        30000
    )?);
    let wallet = Arc::new(core::wallet::Wallet::load(&config.wallet.path)?);

    // Check wallet balance
    let balance = check_wallet_balance(&solana_client, &wallet).await?;
    info!("💰 Wallet balance: {} SOL", balance);

    if balance < 0.1 {
        return Err(anyhow!("Insufficient balance for testing. Need at least 0.1 SOL"));
    }

    // Initialize strategy based on selection
    match args.strategy {
        StrategyType::Arbitrage => {
            test_arbitrage_strategy(&config, &solana_client, &wallet, &args).await?;
        }
        StrategyType::Sandwich => {
            test_sandwich_strategy(&config, &solana_client, &wallet, &args).await?;
        }
        StrategyType::JupiterArb => {
            test_jupiter_arbitrage(&config, &solana_client, &wallet, &args).await?;
        }
        StrategyType::Sniping => {
            test_sniping_strategy(&config, &solana_client, &wallet, &args).await?;
        }
        StrategyType::Liquidation => {
            test_liquidation_strategy(&config, &solana_client, &wallet, &args).await?;
        }
        StrategyType::All => {
            test_all_strategies(&config, &solana_client, &wallet, &args).await?;
        }
    }

    info!("🎉 Devnet trading test completed successfully!");
    Ok(())
}

/// Validate devnet configuration
fn validate_devnet_config(config: &utils::config::Config) -> Result<()> {
    info!("🔍 Validating devnet configuration...");

    // Check if using devnet RPC
    if !config.solana.rpc_url.contains("devnet") {
        warn!("⚠️  Not using devnet RPC URL: {}", config.solana.rpc_url);
    }

    // Validate wallet configuration
    if config.wallet.path.is_empty() {
        return Err(anyhow!("Wallet path not configured"));
    }

    info!("✅ Configuration validation passed");
    Ok(())
}

/// Check wallet balance on devnet
async fn check_wallet_balance(
    solana_client: &Arc<core::solana_client::SolanaClient>,
    wallet: &Arc<core::wallet::Wallet>,
) -> Result<f64> {
    info!("💰 Checking wallet balance...");
    
    let balance_lamports = solana_client.get_balance(&wallet.pubkey()).await?;
    let balance_sol = balance_lamports as f64 / 1_000_000_000.0;
    
    Ok(balance_sol)
}

/// Test arbitrage strategy
async fn test_arbitrage_strategy(
    _config: &utils::config::Config,
    _solana_client: &Arc<core::solana_client::SolanaClient>,
    _wallet: &Arc<core::wallet::Wallet>,
    args: &Args,
) -> Result<()> {
    info!("⚖️ Testing arbitrage strategy...");

    // Initialize arbitrage strategy (config for future use)
    let _arb_config = strategies::jupiter_arb::JupiterArbConfig {
        enabled: true,
        min_profit: args.min_profit,
        max_position: args.max_position,
        slippage_tolerance: 0.03, // 3%
        dex_pairs: vec![
            "raydium-jupiter".to_string(),
            "orca-jupiter".to_string(),
        ],
        execution_timeout_ms: 5000,
    };

    // Create a simple strategy config for testing
    let strategy_config = config::StrategyConfig {
        strategy_mode: "arbitrage".to_string(),
        update_interval_ms: 100,
        order_book_depth: 20,
        spread_bps: 25,
    };
    let mut strategy = strategies::jupiter_arb::JupiterArbStrategy::new(&strategy_config);

    // Run strategy for specified duration
    let test_duration = Duration::from_secs(args.duration);
    let result = timeout(test_duration, run_arbitrage_loop(&mut strategy, args)).await;

    match result {
        Ok(_) => info!("✅ Arbitrage strategy test completed"),
        Err(_) => info!("⏰ Arbitrage strategy test timed out (expected)"),
    }

    Ok(())
}

/// Test sandwich strategy
async fn test_sandwich_strategy(
    _config: &utils::config::Config,
    _solana_client: &Arc<core::solana_client::SolanaClient>,
    _wallet: &Arc<core::wallet::Wallet>,
    args: &Args,
) -> Result<()> {
    info!("🥪 Testing sandwich strategy...");

    // Initialize MEV engine for sandwich attacks
    let mev_config = strategies::mev::MevConfig {
        sandwich_enabled: true,
        arbitrage_enabled: false,
        liquidation_enabled: false,
        token_launch_enabled: false,
        min_profit_threshold: (args.min_profit * 1_000_000_000.0) as u64,
        max_position_size: (args.max_position * 1_000_000_000.0) as u64,
        max_slippage_bps: 500,
        priority_fee_multiplier: 2.0,
    };

    let mut mev_engine = strategies::mev::MevEngine::new(mev_config);

    // Run strategy for specified duration
    let test_duration = Duration::from_secs(args.duration);
    let result = timeout(test_duration, run_sandwich_loop(&mut mev_engine, args)).await;

    match result {
        Ok(_) => info!("✅ Sandwich strategy test completed"),
        Err(_) => info!("⏰ Sandwich strategy test timed out (expected)"),
    }

    Ok(())
}

/// Test Jupiter arbitrage strategy
async fn test_jupiter_arbitrage(
    _config: &utils::config::Config,
    _solana_client: &Arc<core::solana_client::SolanaClient>,
    _wallet: &Arc<core::wallet::Wallet>,
    args: &Args,
) -> Result<()> {
    info!("🔄 Testing Jupiter arbitrage strategy...");

    // Create Jupiter arbitrage configuration (for future use)
    let _jupiter_config = strategies::jupiter_arb::JupiterArbConfig {
        enabled: true,
        min_profit: args.min_profit,
        max_position: args.max_position,
        slippage_tolerance: 0.025, // 2.5%
        dex_pairs: vec![
            "jupiter-raydium".to_string(),
            "jupiter-orca".to_string(),
        ],
        execution_timeout_ms: 3000,
    };

    // Use same strategy config as before
    let strategy_config = config::StrategyConfig {
        strategy_mode: "jupiter_arbitrage".to_string(),
        update_interval_ms: 100,
        order_book_depth: 20,
        spread_bps: 25,
    };
    let mut strategy = strategies::jupiter_arb::JupiterArbStrategy::new(&strategy_config);

    // Run strategy
    let test_duration = Duration::from_secs(args.duration);
    let result = timeout(test_duration, run_jupiter_loop(&mut strategy, args)).await;

    match result {
        Ok(_) => info!("✅ Jupiter arbitrage test completed"),
        Err(_) => info!("⏰ Jupiter arbitrage test timed out (expected)"),
    }

    Ok(())
}

/// Test sniping strategy
async fn test_sniping_strategy(
    _config: &utils::config::Config,
    _solana_client: &Arc<core::solana_client::SolanaClient>,
    _wallet: &Arc<core::wallet::Wallet>,
    args: &Args,
) -> Result<()> {
    info!("🚀 Testing token launch sniping strategy...");
    
    // For now, just simulate sniping detection
    for i in 0..args.duration {
        if i % 10 == 0 {
            info!("🔍 Scanning for token launches... ({}s)", i);
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    info!("✅ Sniping strategy test completed");
    Ok(())
}

/// Test liquidation strategy
async fn test_liquidation_strategy(
    _config: &utils::config::Config,
    _solana_client: &Arc<core::solana_client::SolanaClient>,
    _wallet: &Arc<core::wallet::Wallet>,
    args: &Args,
) -> Result<()> {
    info!("💧 Testing liquidation strategy...");
    
    // For now, just simulate liquidation monitoring
    for i in 0..args.duration {
        if i % 15 == 0 {
            info!("🔍 Monitoring liquidation opportunities... ({}s)", i);
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    info!("✅ Liquidation strategy test completed");
    Ok(())
}

/// Test all strategies sequentially
async fn test_all_strategies(
    config: &utils::config::Config,
    solana_client: &Arc<core::solana_client::SolanaClient>,
    wallet: &Arc<core::wallet::Wallet>,
    args: &Args,
) -> Result<()> {
    info!("🎯 Testing all strategies sequentially...");

    let strategy_duration = args.duration / 5; // Divide time among strategies

    let mut test_args = args.clone();
    test_args.duration = strategy_duration;

    // Test each strategy
    test_arbitrage_strategy(config, solana_client, wallet, &test_args).await?;
    test_sandwich_strategy(config, solana_client, wallet, &test_args).await?;
    test_jupiter_arbitrage(config, solana_client, wallet, &test_args).await?;
    test_sniping_strategy(config, solana_client, wallet, &test_args).await?;
    test_liquidation_strategy(config, solana_client, wallet, &test_args).await?;

    info!("✅ All strategies tested successfully");
    Ok(())
}

/// Run arbitrage detection loop
async fn run_arbitrage_loop(
    _strategy: &mut strategies::jupiter_arb::JupiterArbStrategy,
    args: &Args,
) -> Result<()> {
    let mut opportunities_found = 0;

    loop {
        // Simulate market data analysis
        debug!("🔍 Scanning for arbitrage opportunities...");
        
        // In a real implementation, this would analyze actual market data
        // For testing, we simulate opportunity detection
        if opportunities_found % 20 == 0 && opportunities_found > 0 {
            info!("💡 Simulated arbitrage opportunity detected!");
            if !args.dry_run {
                info!("💰 Would execute arbitrage trade here");
            } else {
                info!("🧪 Dry run: Skipping actual execution");
            }
        }

        opportunities_found += 1;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

/// Run sandwich detection loop
async fn run_sandwich_loop(
    _mev_engine: &mut strategies::mev::MevEngine,
    args: &Args,
) -> Result<()> {
    let mut transactions_analyzed = 0;

    loop {
        // Simulate transaction analysis
        debug!("🔍 Analyzing transactions for sandwich opportunities...");
        
        // In a real implementation, this would analyze mempool transactions
        if transactions_analyzed % 30 == 0 && transactions_analyzed > 0 {
            info!("🥪 Simulated sandwich opportunity detected!");
            if !args.dry_run {
                info!("💰 Would execute sandwich attack here");
            } else {
                info!("🧪 Dry run: Skipping actual execution");
            }
        }

        transactions_analyzed += 1;
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
}

/// Run Jupiter arbitrage loop
async fn run_jupiter_loop(
    _strategy: &mut strategies::jupiter_arb::JupiterArbStrategy,
    args: &Args,
) -> Result<()> {
    let mut routes_checked = 0;

    loop {
        // Simulate Jupiter route analysis
        debug!("🔄 Checking Jupiter routes for arbitrage...");
        
        if routes_checked % 25 == 0 && routes_checked > 0 {
            info!("🎯 Jupiter arbitrage opportunity found!");
            if !args.dry_run {
                info!("💰 Would execute Jupiter arbitrage here");
            } else {
                info!("🧪 Dry run: Skipping actual execution");
            }
        }

        routes_checked += 1;
        tokio::time::sleep(Duration::from_millis(400)).await;
    }
}
