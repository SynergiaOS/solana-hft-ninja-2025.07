//! Security & Risk Management Module
//!
//! Production-grade security features for Solana HFT Ninja 2025.07

pub mod access_control;
pub mod audit_logger;
pub mod circuit_breaker;
pub mod risk_limits;
pub mod wallet_security;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable hardware wallet integration
    pub hardware_wallet_enabled: bool,

    /// Maximum position size in SOL
    pub max_position_size_sol: f64,

    /// Maximum daily loss limit in SOL
    pub max_daily_loss_sol: f64,

    /// Circuit breaker threshold (consecutive losses)
    pub circuit_breaker_threshold: u32,

    /// Enable audit logging
    pub audit_logging_enabled: bool,

    /// Encryption key for sensitive data
    pub encryption_key_path: String,

    /// API rate limits
    pub api_rate_limit_per_minute: u32,

    /// Session timeout in minutes
    pub session_timeout_minutes: u32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            hardware_wallet_enabled: false,
            max_position_size_sol: 0.1,
            max_daily_loss_sol: 0.5,
            circuit_breaker_threshold: 5,
            audit_logging_enabled: true,
            encryption_key_path: "keys/encryption.key".to_string(),
            api_rate_limit_per_minute: 100,
            session_timeout_minutes: 30,
        }
    }
}

/// Security manager - central security coordinator
pub struct SecurityManager {
    config: SecurityConfig,
    wallet_security: wallet_security::WalletSecurity,
    risk_limits: risk_limits::RiskLimits,
    circuit_breaker: circuit_breaker::CircuitBreaker,
    audit_logger: audit_logger::AuditLogger,
    access_control: access_control::AccessControl,
}

impl SecurityManager {
    /// Create new security manager
    pub fn new(config: SecurityConfig) -> Result<Self> {
        info!("🔐 Initializing Security Manager...");

        let wallet_security = wallet_security::WalletSecurity::new(&config)?;
        let risk_limits = risk_limits::RiskLimits::new(&config)?;
        let circuit_breaker = circuit_breaker::CircuitBreaker::new(&config)?;
        let audit_logger = audit_logger::AuditLogger::new(&config)?;
        let access_control = access_control::AccessControl::new(&config)?;

        info!("✅ Security Manager initialized successfully");

        Ok(Self {
            config,
            wallet_security,
            risk_limits,
            circuit_breaker,
            audit_logger,
            access_control,
        })
    }

    /// Validate transaction before execution
    pub async fn validate_transaction(
        &mut self,
        transaction: &TransactionRequest,
    ) -> Result<SecurityValidation> {
        let start_time = SystemTime::now();

        // Log security check
        self.audit_logger.log_security_check(transaction).await?;

        // Check circuit breaker
        if self.circuit_breaker.is_open() {
            warn!("🚨 Circuit breaker is OPEN - blocking transaction");
            return Ok(SecurityValidation::Blocked(
                "Circuit breaker active".to_string(),
            ));
        }

        // Check risk limits
        if !self
            .risk_limits
            .check_position_size(transaction.amount_sol)?
        {
            warn!(
                "🚨 Position size exceeds limit: {} SOL",
                transaction.amount_sol
            );
            return Ok(SecurityValidation::Blocked(
                "Position size limit exceeded".to_string(),
            ));
        }

        // Check daily loss limit
        if !self.risk_limits.check_daily_loss_limit().await? {
            warn!("🚨 Daily loss limit exceeded");
            return Ok(SecurityValidation::Blocked(
                "Daily loss limit exceeded".to_string(),
            ));
        }

        // Validate wallet security
        if !self
            .wallet_security
            .validate_transaction(transaction)
            .await?
        {
            error!("🚨 Wallet security validation failed");
            return Ok(SecurityValidation::Blocked(
                "Wallet security validation failed".to_string(),
            ));
        }

        let validation_time = start_time.elapsed().unwrap_or(Duration::from_millis(0));

        info!(
            "✅ Transaction security validation passed ({:.2}ms)",
            validation_time.as_millis()
        );

        Ok(SecurityValidation::Approved {
            validation_time,
            risk_score: self.calculate_risk_score(transaction),
        })
    }

    /// Handle transaction result for risk tracking
    pub async fn handle_transaction_result(&mut self, result: &TransactionResult) -> Result<()> {
        // Update risk limits
        self.risk_limits.update_with_result(result).await?;

        // Update circuit breaker
        self.circuit_breaker.update_with_result(result).await?;

        // Log transaction result
        self.audit_logger.log_transaction_result(result).await?;

        Ok(())
    }

    /// Calculate risk score for transaction
    fn calculate_risk_score(&self, transaction: &TransactionRequest) -> f64 {
        let mut score = 0.0;

        // Position size risk
        let position_ratio = transaction.amount_sol / self.config.max_position_size_sol;
        score += position_ratio * 30.0;

        // Slippage risk
        score += transaction.max_slippage_bps as f64 * 0.01;

        // Market volatility (simplified)
        score += 10.0;

        // Cap at 100
        score.min(100.0)
    }

    /// Emergency shutdown
    pub async fn emergency_shutdown(&mut self, reason: &str) -> Result<()> {
        error!("🚨 EMERGENCY SHUTDOWN: {}", reason);

        // Activate circuit breaker
        self.circuit_breaker.emergency_open(reason).await?;

        // Log emergency event
        self.audit_logger.log_emergency_event(reason).await?;

        // Secure wallet
        self.wallet_security.emergency_lock().await?;

        Ok(())
    }

    /// Get security status
    pub fn get_security_status(&self) -> SecurityStatus {
        SecurityStatus {
            circuit_breaker_open: self.circuit_breaker.is_open(),
            daily_loss_ratio: self.risk_limits.get_daily_loss_ratio(),
            position_utilization: self.risk_limits.get_position_utilization(),
            wallet_locked: self.wallet_security.is_locked(),
            last_security_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Transaction request for security validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub signature: String,
    pub amount_sol: f64,
    pub max_slippage_bps: u32,
    pub transaction_type: String,
    pub timestamp: u64,
}

/// Transaction result for risk tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub signature: String,
    pub success: bool,
    pub profit_loss_sol: f64,
    pub gas_used: u64,
    pub execution_time_ms: u64,
    pub timestamp: u64,
}

/// Security validation result
#[derive(Debug, Clone)]
pub enum SecurityValidation {
    Approved {
        validation_time: Duration,
        risk_score: f64,
    },
    Blocked(String),
}

/// Security status for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStatus {
    pub circuit_breaker_open: bool,
    pub daily_loss_ratio: f64,
    pub position_utilization: f64,
    pub wallet_locked: bool,
    pub last_security_check: u64,
}

// Re-export main types
pub use access_control::AccessControl;
pub use audit_logger::AuditLogger;
pub use circuit_breaker::CircuitBreaker;
pub use risk_limits::RiskLimits;
pub use wallet_security::WalletSecurity;
