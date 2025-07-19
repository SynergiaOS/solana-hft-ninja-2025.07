use crate::ai::{AIConfig, LMCacheConfig, OpenSearchConfig, OumiConfig};
use crate::strategies::wallet_tracker::WalletTrackerConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub solana: SolanaConfig,
    pub wallet: WalletConfig,
    pub trading: TradingConfig,
    pub strategy: StrategyConfig,
    pub risk: RiskConfig,
    pub logging: LoggingConfig,
    pub monitoring: MonitoringConfig,
    pub wallet_tracker: Option<WalletTrackerConfig>,
    pub oumi_ai: Option<OumiConfig>,
    pub opensearch_ai: Option<OpenSearchConfig>,
    pub lmcache: Option<LMCacheConfig>,
    pub ai: Option<AIConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub ws_url: String,
    pub rpc_timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub private_key_path: String,
    pub keypair_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    pub initial_balance_sol: f64,
    pub max_position_size_sol: f64,
    pub max_slippage_bps: u64,
    pub min_profit_threshold_bps: u64,
    pub risk_limit_bps: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub strategy_mode: String,
    pub update_interval_ms: u64,
    pub order_book_depth: usize,
    pub spread_bps: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    pub stop_loss_bps: u64,
    pub take_profit_bps: u64,
    pub max_drawdown_bps: u64,
    pub risk_limit_bps: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub rust_log: String,
    pub log_level: String,
    pub log_file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_port: u16,
    pub health_check_interval_ms: u64,
    pub enable_ddos_protection: bool,
    pub rate_limit_rps: u64,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            stop_loss_bps: 200,    // 2%
            take_profit_bps: 300,  // 3%
            max_drawdown_bps: 500, // 5%
            risk_limit_bps: 1000,  // 10% max position size
        }
    }
}

impl Config {
    pub fn load(config_path: &str) -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(
                config::File::with_name(config_path)
                    .format(config::FileFormat::Toml)
                    .required(true),
            )
            .add_source(config::Environment::with_prefix("SOLANA_HFT"))
            .build()?;

        settings.try_deserialize()
    }
}
