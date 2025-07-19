//! Audit Logger Module
//!
//! Comprehensive audit logging for security events and transactions

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use super::{SecurityConfig, TransactionRequest, TransactionResult};

/// Audit logger for security events
pub struct AuditLogger {
    config: SecurityConfig,
    log_file: Mutex<std::fs::File>,
}

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    SecurityCheck,
    TransactionResult,
    WalletAccess,
    CircuitBreakerEvent,
    EmergencyEvent,
    LoginAttempt,
    ConfigChange,
    SystemError,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub timestamp: u64,
    pub event_type: AuditEventType,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub event_data: serde_json::Value,
    pub severity: AuditSeverity,
    pub session_id: Option<String>,
}

/// Audit severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl AuditLogger {
    /// Create new audit logger
    pub fn new(config: &SecurityConfig) -> Result<Self> {
        info!("📝 Initializing Audit Logger...");

        // Create audit log directory if it doesn't exist
        let log_dir = Path::new("logs/audit");
        std::fs::create_dir_all(log_dir)?;

        // Open audit log file
        let log_file_path = log_dir.join("security_audit.jsonl");
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)?;

        info!("✅ Audit Logger initialized");

        Ok(Self {
            config: config.clone(),
            log_file: Mutex::new(log_file),
        })
    }

    /// Log security check event
    pub async fn log_security_check(&self, transaction: &TransactionRequest) -> Result<()> {
        let event_data = serde_json::json!({
            "transaction_signature": transaction.signature,
            "amount_sol": transaction.amount_sol,
            "transaction_type": transaction.transaction_type,
            "max_slippage_bps": transaction.max_slippage_bps,
        });

        self.log_event(
            AuditEventType::SecurityCheck,
            event_data,
            AuditSeverity::Info,
            None,
            None,
        )
        .await?;

        Ok(())
    }

    /// Log transaction result
    pub async fn log_transaction_result(&self, result: &TransactionResult) -> Result<()> {
        let severity = if result.success {
            AuditSeverity::Info
        } else {
            AuditSeverity::Warning
        };

        let event_data = serde_json::json!({
            "transaction_signature": result.signature,
            "success": result.success,
            "profit_loss_sol": result.profit_loss_sol,
            "gas_used": result.gas_used,
            "execution_time_ms": result.execution_time_ms,
        });

        self.log_event(
            AuditEventType::TransactionResult,
            event_data,
            severity,
            None,
            None,
        )
        .await?;

        Ok(())
    }

    /// Log emergency event
    pub async fn log_emergency_event(&self, reason: &str) -> Result<()> {
        let event_data = serde_json::json!({
            "reason": reason,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        self.log_event(
            AuditEventType::EmergencyEvent,
            event_data,
            AuditSeverity::Critical,
            None,
            None,
        )
        .await?;

        Ok(())
    }

    /// Log wallet access event
    pub async fn log_wallet_access(
        &self,
        action: &str,
        success: bool,
        user_id: Option<&str>,
    ) -> Result<()> {
        let severity = if success {
            AuditSeverity::Info
        } else {
            AuditSeverity::Warning
        };

        let event_data = serde_json::json!({
            "action": action,
            "success": success,
        });

        self.log_event(
            AuditEventType::WalletAccess,
            event_data,
            severity,
            user_id.map(|s| s.to_string()),
            None,
        )
        .await?;

        Ok(())
    }

    /// Log circuit breaker event
    pub async fn log_circuit_breaker_event(&self, state: &str, reason: Option<&str>) -> Result<()> {
        let severity = match state {
            "open" => AuditSeverity::Error,
            "closed" => AuditSeverity::Info,
            "half_open" => AuditSeverity::Warning,
            _ => AuditSeverity::Info,
        };

        let event_data = serde_json::json!({
            "state": state,
            "reason": reason,
        });

        self.log_event(
            AuditEventType::CircuitBreakerEvent,
            event_data,
            severity,
            None,
            None,
        )
        .await?;

        Ok(())
    }

    /// Log login attempt
    pub async fn log_login_attempt(
        &self,
        user_id: &str,
        success: bool,
        ip_address: Option<&str>,
    ) -> Result<()> {
        let severity = if success {
            AuditSeverity::Info
        } else {
            AuditSeverity::Warning
        };

        let event_data = serde_json::json!({
            "user_id": user_id,
            "success": success,
            "ip_address": ip_address,
        });

        self.log_event(
            AuditEventType::LoginAttempt,
            event_data,
            severity,
            Some(user_id.to_string()),
            ip_address.map(|s| s.to_string()),
        )
        .await?;

        Ok(())
    }

    /// Log configuration change
    pub async fn log_config_change(
        &self,
        config_key: &str,
        old_value: &str,
        new_value: &str,
        user_id: Option<&str>,
    ) -> Result<()> {
        let event_data = serde_json::json!({
            "config_key": config_key,
            "old_value": old_value,
            "new_value": new_value,
        });

        self.log_event(
            AuditEventType::ConfigChange,
            event_data,
            AuditSeverity::Warning,
            user_id.map(|s| s.to_string()),
            None,
        )
        .await?;

        Ok(())
    }

    /// Log system error
    pub async fn log_system_error(
        &self,
        error_message: &str,
        error_code: Option<&str>,
    ) -> Result<()> {
        let event_data = serde_json::json!({
            "error_message": error_message,
            "error_code": error_code,
        });

        self.log_event(
            AuditEventType::SystemError,
            event_data,
            AuditSeverity::Error,
            None,
            None,
        )
        .await?;

        Ok(())
    }

    /// Core logging function
    async fn log_event(
        &self,
        event_type: AuditEventType,
        event_data: serde_json::Value,
        severity: AuditSeverity,
        user_id: Option<String>,
        ip_address: Option<String>,
    ) -> Result<()> {
        let entry = AuditLogEntry {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            event_type,
            user_id,
            ip_address,
            event_data,
            severity,
            session_id: None, // TODO: Implement session tracking
        };

        // Serialize to JSON
        let json_line = serde_json::to_string(&entry)?;

        // Write to file
        {
            let mut file = self.log_file.lock().await;
            writeln!(file, "{}", json_line)?;
            file.flush()?;
        }

        // Also log to tracing for immediate visibility
        match entry.severity {
            AuditSeverity::Info => debug!("AUDIT: {:?}", entry.event_type),
            AuditSeverity::Warning => warn!("AUDIT: {:?}", entry.event_type),
            AuditSeverity::Error => error!("AUDIT: {:?}", entry.event_type),
            AuditSeverity::Critical => error!("AUDIT CRITICAL: {:?}", entry.event_type),
        }

        Ok(())
    }

    /// Get audit statistics
    pub async fn get_audit_stats(&self) -> AuditStats {
        // In a real implementation, this would read and analyze the log file
        // For now, return placeholder stats
        AuditStats {
            total_events: 0,
            security_checks: 0,
            failed_transactions: 0,
            emergency_events: 0,
            failed_logins: 0,
            last_event_timestamp: 0,
        }
    }
}

/// Audit statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_events: u64,
    pub security_checks: u64,
    pub failed_transactions: u64,
    pub emergency_events: u64,
    pub failed_logins: u64,
    pub last_event_timestamp: u64,
}
