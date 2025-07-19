// 🥷 Solana HFT Ninja 2025.07 - Unified Architecture
// Single binary with multiple operation modes for maximum performance

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info};

// Use modules from lib
use solana_hft_ninja::*;

use solana_hft_ninja::core::{Engine, EngineConfig};
use solana_hft_ninja::{
    config::Config,
    utils::{logging::init_logging, metrics::MetricsServer},
};

#[derive(Parser)]
#[command(name = "hft-ninja")]
#[command(about = "🥷 Solana HFT Ninja 2025.07 - Ultra-Fast Trading Engine")]
#[command(version = "2025.07")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Enable metrics server
    #[arg(long, default_value = "true")]
    metrics: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the complete trading engine
    Trade {
        /// Trading strategies to enable
        #[arg(short, long, value_delimiter = ',')]
        strategies: Option<Vec<String>>,

        /// Dry run mode (no real trades)
        #[arg(long)]
        dry_run: bool,
    },

    /// Run API server only
    Api {
        /// API server port
        #[arg(short, long, default_value = "8001")]
        port: u16,
    },

    /// Run market data collector
    Data {
        /// Data sources to collect from
        #[arg(short, long, value_delimiter = ',')]
        sources: Option<Vec<String>>,
    },

    /// Run strategy backtesting
    Backtest {
        /// Strategy to backtest
        #[arg(short, long)]
        strategy: String,

        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        start_date: String,

        /// End date (YYYY-MM-DD)
        #[arg(long)]
        end_date: String,
    },

    /// Run performance benchmarks
    Benchmark {
        /// Benchmark type
        #[arg(short, long, default_value = "all")]
        bench_type: String,

        /// Number of iterations
        #[arg(short, long, default_value = "1000")]
        iterations: u32,
    },

    /// Validate configuration and setup
    Validate,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(&cli.log_level)?;

    // Load configuration
    let config = Config::load(&cli.config)?;
    info!("🚀 HFT Ninja starting with config: {}", cli.config);

    // Start metrics server if enabled
    let _metrics_server = if cli.metrics {
        Some(MetricsServer::start(9090).await?)
    } else {
        None
    };

    // Execute command
    match cli.command {
        Commands::Trade {
            strategies,
            dry_run,
        } => run_trading_engine(config, strategies, dry_run).await,
        Commands::Api { port } => run_api_server(config, port).await,
        Commands::Data { sources } => run_data_collector(config, sources).await,
        Commands::Backtest {
            strategy,
            start_date,
            end_date,
        } => run_backtest(config, strategy, start_date, end_date).await,
        Commands::Benchmark {
            bench_type,
            iterations,
        } => run_benchmark(config, bench_type, iterations).await,
        Commands::Validate => validate_setup(config).await,
    }
}

async fn run_trading_engine(
    config: Config,
    strategies: Option<Vec<String>>,
    dry_run: bool,
) -> Result<()> {
    info!("🎯 Starting trading engine (dry_run: {})", dry_run);

    // Create engine configuration
    let engine_config = EngineConfig {
        dry_run,
        strategies: strategies.unwrap_or_else(|| vec!["sandwich".to_string()]),
        solana_rpc_url: config.solana.rpc_url.clone(),
        solana_ws_url: config.solana.ws_url.clone(),
        wallet_path: config.wallet.private_key_path.clone(),
        max_position_size: config.trading.max_position_size_sol,
        risk_limits: solana_hft_ninja::utils::config::RiskConfig {
            max_daily_loss: 1.0,
            max_position_size: config.trading.max_position_size_sol,
            stop_loss_percentage: 0.05,
            take_profit_percentage: 0.1,
        },
    };

    // Initialize trading engine
    let engine = Arc::new(Engine::new(engine_config).await?);

    // Start engine
    let engine_handle = {
        let engine = Arc::clone(&engine);
        tokio::spawn(async move {
            if let Err(e) = engine.run().await {
                error!("Trading engine error: {}", e);
            }
        })
    };

    // Start API server
    let api_handle = {
        let engine = Arc::clone(&engine);
        tokio::spawn(async move {
            if let Err(e) = solana_hft_ninja::api::start_server(engine, 8080).await {
                error!("API server error: {}", e);
            }
        })
    };

    // Wait for shutdown signal
    info!("🥷 HFT Ninja running. Press Ctrl+C to stop.");
    signal::ctrl_c().await?;
    info!("🛑 Shutdown signal received");

    // Graceful shutdown
    engine.shutdown().await?;
    engine_handle.abort();
    api_handle.abort();

    info!("✅ HFT Ninja stopped gracefully");
    Ok(())
}

async fn run_api_server(config: Config, port: u16) -> Result<()> {
    info!("🌐 Starting API server on port {}", port);

    // Create minimal engine for API
    let engine_config = EngineConfig {
        dry_run: true,
        strategies: vec![],
        solana_rpc_url: config.solana.rpc_url.clone(),
        solana_ws_url: config.solana.ws_url.clone(),
        wallet_path: config.wallet.private_key_path.clone(),
        max_position_size: config.trading.max_position_size_sol,
        risk_limits: solana_hft_ninja::utils::config::RiskConfig {
            max_daily_loss: 1.0,
            max_position_size: config.trading.max_position_size_sol,
            stop_loss_percentage: 0.05,
            take_profit_percentage: 0.1,
        },
    };

    let engine = Arc::new(Engine::new(engine_config).await?);

    // Start API server
    solana_hft_ninja::api::start_server(engine, port).await?;

    Ok(())
}

async fn run_data_collector(config: Config, sources: Option<Vec<String>>) -> Result<()> {
    info!("📊 Starting data collector");

    let sources = sources.unwrap_or_else(|| vec!["solana".to_string()]);

    // Initialize data collector
    // Create a dummy utils config for DataCollector
    let utils_config = solana_hft_ninja::utils::config::Config::default();
    let collector = solana_hft_ninja::network::DataCollector::new(utils_config).await?;

    // Start collection
    collector.start(sources).await?;

    // Wait for shutdown
    signal::ctrl_c().await?;
    info!("🛑 Data collector stopped");

    Ok(())
}

async fn run_backtest(
    config: Config,
    strategy: String,
    start_date: String,
    end_date: String,
) -> Result<()> {
    info!("📈 Running backtest for strategy: {}", strategy);
    info!("📅 Period: {} to {}", start_date, end_date);

    // Initialize backtesting engine
    // Create a dummy utils config for Backtester
    let utils_config = solana_hft_ninja::utils::config::Config::default();
    let backtester = solana_hft_ninja::strategies::Backtester::new(utils_config).await?;

    // Run backtest
    let results = backtester.run(&strategy, &start_date, &end_date).await?;

    // Display results
    info!("📊 Backtest Results:");
    info!("  Total Trades: {}", results.total_trades);
    info!("  Successful: {}", results.successful_trades);
    info!("  Success Rate: {:.2}%", results.success_rate * 100.0);
    info!("  Total Profit: {:.6} SOL", results.total_profit);
    info!("  Max Drawdown: {:.2}%", results.max_drawdown * 100.0);
    info!("  Sharpe Ratio: {:.2}", results.sharpe_ratio);

    Ok(())
}

async fn run_benchmark(config: Config, bench_type: String, iterations: u32) -> Result<()> {
    info!(
        "⚡ Running {} benchmark with {} iterations",
        bench_type, iterations
    );

    // Initialize benchmark suite
    // Create a dummy utils config for Benchmarker
    let utils_config = solana_hft_ninja::utils::config::Config::default();
    let benchmarker = solana_hft_ninja::utils::metrics::Benchmarker::new(utils_config).await?;

    // Run benchmark
    let results = benchmarker.run(&bench_type, iterations).await?;

    // Display results
    info!("📊 Benchmark Results:");
    info!("  Average Latency: {:.2}ms", results.avg_latency_ms);
    info!("  95th Percentile: {:.2}ms", results.p95_latency_ms);
    info!("  99th Percentile: {:.2}ms", results.p99_latency_ms);
    info!("  Throughput: {:.0} ops/sec", results.throughput);
    info!("  Memory Usage: {:.2}MB", results.memory_usage_mb);

    Ok(())
}

async fn validate_setup(config: Config) -> Result<()> {
    info!("🔍 Validating HFT Ninja setup...");

    // Validate configuration
    // Config validation - basic checks
    if config.trading.max_position_size_sol <= 0.0 {
        return Err(anyhow::anyhow!("Max position size must be positive"));
    }
    info!("✅ Configuration valid");

    // Test Solana connection
    let solana_client =
        solana_hft_ninja::network::SolanaClient::new(&config.solana.rpc_url).await?;
    let health = solana_client.get_health().await?;
    info!("✅ Solana connection: {}", health);

    // Test wallet
    let wallet = solana_hft_ninja::core::Wallet::load(&config.wallet.private_key_path)?;
    info!("✅ Wallet loaded: {}", wallet.pubkey());

    // Test strategies
    // Default strategies for now
    let enabled_strategies = vec!["arbitrage".to_string()];
    for strategy_name in &enabled_strategies {
        let strategy = solana_hft_ninja::strategies::create_strategy(strategy_name)?;
        info!("✅ Strategy '{}' loaded", strategy.name());
    }

    info!("🎉 All validations passed! HFT Ninja is ready to trade.");
    Ok(())
}
