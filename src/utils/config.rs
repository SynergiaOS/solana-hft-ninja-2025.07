// 🥷 Configuration Management - Unified Config System
// High-performance configuration with validation

use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use std::path::Path;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub solana: SolanaConfig,
    pub wallet: WalletConfig,
    pub trading: TradingConfig,
    pub risk: RiskConfig,
    pub api: ApiConfig,
    pub metrics: MetricsConfig,
    pub strategies: StrategiesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub ws_url: String,
    pub commitment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub path: String,
    pub auto_approve: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    pub max_position_size: f64,
    pub default_slippage: f64,
    pub execution_timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    pub max_daily_loss: f64,
    pub max_position_size: f64,
    pub stop_loss_percentage: f64,
    pub take_profit_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub port: u16,
    pub cors_enabled: bool,
    pub rate_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub port: u16,
    pub enabled: bool,
    pub export_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategiesConfig {
    pub enabled: Vec<String>,
    pub config_path: String,
}

impl Config {
    /// Load configuration from file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context("Failed to read config file")?;
        
        let config: Config = toml::from_str(&content)
            .context("Failed to parse config file")?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate Solana config
        if self.solana.rpc_url.is_empty() {
            return Err(anyhow::anyhow!("Solana RPC URL cannot be empty"));
        }
        
        // Validate wallet config
        if self.wallet.path.is_empty() {
            return Err(anyhow::anyhow!("Wallet path cannot be empty"));
        }
        
        // Validate trading config
        if self.trading.max_position_size <= 0.0 {
            return Err(anyhow::anyhow!("Max position size must be positive"));
        }
        
        // Validate risk config
        if self.risk.max_daily_loss <= 0.0 {
            return Err(anyhow::anyhow!("Max daily loss must be positive"));
        }
        
        Ok(())
    }
    
    /// Get default configuration
    pub fn default() -> Self {
        Self {
            solana: SolanaConfig {
                rpc_url: "https://api.devnet.solana.com".to_string(),
                ws_url: "wss://api.devnet.solana.com".to_string(),
                commitment: "confirmed".to_string(),
            },
            wallet: WalletConfig {
                path: "wallet.json".to_string(),
                auto_approve: false,
            },
            trading: TradingConfig {
                max_position_size: 1.0,
                default_slippage: 0.01,
                execution_timeout_ms: 5000,
            },
            risk: RiskConfig {
                max_daily_loss: 0.1,
                max_position_size: 1.0,
                stop_loss_percentage: 0.05,
                take_profit_percentage: 0.1,
            },
            api: ApiConfig {
                port: 8001,
                cors_enabled: true,
                rate_limit: 100,
            },
            metrics: MetricsConfig {
                port: 9090,
                enabled: true,
                export_interval_ms: 1000,
            },
            strategies: StrategiesConfig {
                enabled: vec!["sandwich".to_string()],
                config_path: "strategies.toml".to_string(),
            },
        }
    }
}
