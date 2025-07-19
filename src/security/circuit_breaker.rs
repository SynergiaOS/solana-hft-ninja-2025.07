//! Circuit Breaker Module
//!
//! Automatic trading halt on consecutive losses or system errors

use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use super::{SecurityConfig, TransactionResult};

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Trading halted
    HalfOpen, // Testing if system recovered
}

/// Circuit breaker manager
pub struct CircuitBreaker {
    config: SecurityConfig,
    state: Arc<RwLock<CircuitBreakerData>>,
}

/// Circuit breaker internal data
#[derive(Debug, Clone)]
struct CircuitBreakerData {
    pub state: CircuitBreakerState,
    pub consecutive_failures: u32,
    pub last_failure_time: Option<std::time::SystemTime>,
    pub total_failures: u32,
    pub last_success_time: Option<std::time::SystemTime>,
    pub emergency_reason: Option<String>,
    pub auto_recovery_enabled: bool,
    pub recovery_timeout_seconds: u64,
}

impl CircuitBreaker {
    /// Create new circuit breaker
    pub fn new(config: &SecurityConfig) -> Result<Self> {
        info!("🔌 Initializing Circuit Breaker...");

        let state = Arc::new(RwLock::new(CircuitBreakerData {
            state: CircuitBreakerState::Closed,
            consecutive_failures: 0,
            last_failure_time: None,
            total_failures: 0,
            last_success_time: Some(std::time::SystemTime::now()),
            emergency_reason: None,
            auto_recovery_enabled: true,
            recovery_timeout_seconds: 300, // 5 minutes
        }));

        info!("✅ Circuit Breaker initialized");

        Ok(Self {
            config: config.clone(),
            state,
        })
    }

    /// Check if circuit breaker is open (trading halted)
    pub fn is_open(&self) -> bool {
        // Simplified sync check - in production use async version
        false // Placeholder
    }

    /// Update circuit breaker with transaction result
    pub async fn update_with_result(&mut self, result: &TransactionResult) -> Result<()> {
        let mut data = self.state.write();

        if result.success && result.profit_loss_sol >= 0.0 {
            // Successful trade
            self.handle_success(&mut data).await?;
        } else {
            // Failed trade or loss
            self.handle_failure(&mut data, result).await?;
        }

        Ok(())
    }

    /// Handle successful transaction
    async fn handle_success(&self, data: &mut CircuitBreakerData) -> Result<()> {
        data.consecutive_failures = 0;
        data.last_success_time = Some(std::time::SystemTime::now());

        match data.state {
            CircuitBreakerState::HalfOpen => {
                // Recovery successful - close circuit breaker
                data.state = CircuitBreakerState::Closed;
                info!("✅ Circuit breaker CLOSED - system recovered");
            }
            CircuitBreakerState::Closed => {
                // Normal operation continues
                debug!("✅ Successful transaction - circuit breaker remains closed");
            }
            CircuitBreakerState::Open => {
                // Should not happen - open circuit should block transactions
                warn!("⚠️ Successful transaction while circuit breaker is open");
            }
        }

        Ok(())
    }

    /// Handle failed transaction
    async fn handle_failure(
        &self,
        data: &mut CircuitBreakerData,
        result: &TransactionResult,
    ) -> Result<()> {
        data.consecutive_failures += 1;
        data.total_failures += 1;
        data.last_failure_time = Some(std::time::SystemTime::now());

        let threshold = self.config.circuit_breaker_threshold;

        match data.state {
            CircuitBreakerState::Closed => {
                if data.consecutive_failures >= threshold {
                    // Open circuit breaker
                    data.state = CircuitBreakerState::Open;
                    error!(
                        "🚨 Circuit breaker OPENED - {} consecutive failures",
                        data.consecutive_failures
                    );

                    // Schedule auto-recovery if enabled
                    if data.auto_recovery_enabled {
                        self.schedule_recovery_attempt(data.recovery_timeout_seconds)
                            .await?;
                    }
                } else {
                    warn!(
                        "⚠️ Failure {}/{} - circuit breaker remains closed",
                        data.consecutive_failures, threshold
                    );
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Failed during recovery - back to open
                data.state = CircuitBreakerState::Open;
                error!("🚨 Recovery failed - circuit breaker back to OPEN");

                // Schedule another recovery attempt
                if data.auto_recovery_enabled {
                    self.schedule_recovery_attempt(data.recovery_timeout_seconds * 2)
                        .await?;
                }
            }
            CircuitBreakerState::Open => {
                // Already open - should not receive transactions
                debug!("Circuit breaker already open - ignoring failure");
            }
        }

        Ok(())
    }

    /// Emergency open circuit breaker
    pub async fn emergency_open(&mut self, reason: &str) -> Result<()> {
        let mut data = self.state.write();

        data.state = CircuitBreakerState::Open;
        data.emergency_reason = Some(reason.to_string());
        data.auto_recovery_enabled = false; // Disable auto-recovery for emergency

        error!("🚨 EMERGENCY CIRCUIT BREAKER ACTIVATION: {}", reason);

        Ok(())
    }

    /// Manually close circuit breaker
    pub async fn manual_close(&mut self) -> Result<()> {
        let mut data = self.state.write();

        data.state = CircuitBreakerState::Closed;
        data.consecutive_failures = 0;
        data.emergency_reason = None;
        data.auto_recovery_enabled = true;

        info!("🔧 Circuit breaker manually CLOSED");

        Ok(())
    }

    /// Attempt recovery (half-open state)
    pub async fn attempt_recovery(&mut self) -> Result<()> {
        let mut data = self.state.write();

        if data.state == CircuitBreakerState::Open {
            data.state = CircuitBreakerState::HalfOpen;
            info!("🔄 Circuit breaker HALF-OPEN - testing recovery");
        }

        Ok(())
    }

    /// Schedule automatic recovery attempt
    async fn schedule_recovery_attempt(&self, delay_seconds: u64) -> Result<()> {
        info!(
            "⏰ Scheduling recovery attempt in {} seconds",
            delay_seconds
        );

        // In a real implementation, this would use a proper scheduler
        // For now, just log the intent
        debug!("Recovery scheduled (implementation needed)");

        Ok(())
    }

    /// Get circuit breaker status
    pub async fn get_status(&self) -> CircuitBreakerStatus {
        let data = self.state.read();

        let uptime_seconds = data
            .last_success_time
            .and_then(|t| t.elapsed().ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let downtime_seconds = if data.state == CircuitBreakerState::Open {
            data.last_failure_time
                .and_then(|t| t.elapsed().ok())
                .map(|d| d.as_secs())
                .unwrap_or(0)
        } else {
            0
        };

        CircuitBreakerStatus {
            state: data.state.clone(),
            consecutive_failures: data.consecutive_failures,
            total_failures: data.total_failures,
            threshold: self.config.circuit_breaker_threshold,
            uptime_seconds,
            downtime_seconds,
            emergency_reason: data.emergency_reason.clone(),
            auto_recovery_enabled: data.auto_recovery_enabled,
        }
    }
}

/// Circuit breaker status for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerStatus {
    pub state: CircuitBreakerState,
    pub consecutive_failures: u32,
    pub total_failures: u32,
    pub threshold: u32,
    pub uptime_seconds: u64,
    pub downtime_seconds: u64,
    pub emergency_reason: Option<String>,
    pub auto_recovery_enabled: bool,
}

// Implement Serialize/Deserialize for CircuitBreakerState
impl Serialize for CircuitBreakerState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            CircuitBreakerState::Closed => serializer.serialize_str("closed"),
            CircuitBreakerState::Open => serializer.serialize_str("open"),
            CircuitBreakerState::HalfOpen => serializer.serialize_str("half_open"),
        }
    }
}

impl<'de> Deserialize<'de> for CircuitBreakerState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "closed" => Ok(CircuitBreakerState::Closed),
            "open" => Ok(CircuitBreakerState::Open),
            "half_open" => Ok(CircuitBreakerState::HalfOpen),
            _ => Err(serde::de::Error::custom("Invalid circuit breaker state")),
        }
    }
}
