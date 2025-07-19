//! 🌪️ Chaos Testing Framework - Resilience Validation
//! 
//! Tests system behavior under failure conditions

use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::{sleep, timeout};
use tracing::{info, warn, error};
use rand::Rng;

/// Chaos test configuration
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    pub test_duration_seconds: u64,
    pub failure_injection_rate: f64, // 0.0 - 1.0
    pub recovery_timeout_seconds: u64,
    pub max_concurrent_failures: usize,
    pub enable_network_failures: bool,
    pub enable_memory_pressure: bool,
    pub enable_disk_failures: bool,
    pub enable_dependency_failures: bool,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            test_duration_seconds: 300, // 5 minutes
            failure_injection_rate: 0.1, // 10% failure rate
            recovery_timeout_seconds: 30,
            max_concurrent_failures: 3,
            enable_network_failures: true,
            enable_memory_pressure: true,
            enable_disk_failures: false, // Dangerous in production
            enable_dependency_failures: true,
        }
    }
}

/// Types of failures to inject
#[derive(Debug, Clone, PartialEq)]
pub enum FailureType {
    NetworkPartition,
    HighLatency,
    MemoryPressure,
    CpuStarvation,
    DiskFull,
    DatabaseDown,
    RedisDown,
    SolanaRpcDown,
    HeliusDown,
}

/// Failure injection result
#[derive(Debug)]
pub struct FailureInjection {
    pub failure_type: FailureType,
    pub start_time: Instant,
    pub duration: Duration,
    pub recovered: bool,
    pub recovery_time: Option<Duration>,
}

/// System health metrics during chaos
#[derive(Debug, Default)]
pub struct ChaosMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_latency_ms: f64,
    pub max_latency_ms: f64,
    pub failures_injected: u64,
    pub failures_recovered: u64,
    pub system_downtime_ms: u64,
}

/// Chaos testing engine
pub struct ChaosEngine {
    config: ChaosConfig,
    active_failures: Arc<RwLock<Vec<FailureInjection>>>,
    metrics: Arc<RwLock<ChaosMetrics>>,
    system_under_test: Arc<dyn SystemUnderTest>,
}

/// Trait for systems that can be chaos tested
#[async_trait::async_trait]
pub trait SystemUnderTest: Send + Sync {
    async fn health_check(&self) -> Result<bool>;
    async fn process_request(&self) -> Result<Duration>;
    async fn get_metrics(&self) -> Result<SystemMetrics>;
    async fn inject_failure(&self, failure_type: FailureType) -> Result<()>;
    async fn recover_from_failure(&self, failure_type: FailureType) -> Result<()>;
}

#[derive(Debug)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_latency: f64,
    pub active_connections: u64,
}

impl ChaosEngine {
    pub fn new(config: ChaosConfig, system: Arc<dyn SystemUnderTest>) -> Self {
        Self {
            config,
            active_failures: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(ChaosMetrics::default())),
            system_under_test: system,
        }
    }

    /// Run comprehensive chaos test
    pub async fn run_chaos_test(&self) -> Result<ChaosTestReport> {
        info!("🌪️ Starting chaos test for {} seconds", self.config.test_duration_seconds);
        
        let start_time = Instant::now();
        let test_duration = Duration::from_secs(self.config.test_duration_seconds);
        
        // Start background tasks
        let failure_injector = self.start_failure_injector();
        let load_generator = self.start_load_generator();
        let health_monitor = self.start_health_monitor();
        let recovery_manager = self.start_recovery_manager();

        // Wait for test completion
        sleep(test_duration).await;

        // Stop background tasks
        failure_injector.abort();
        load_generator.abort();
        health_monitor.abort();
        recovery_manager.abort();

        // Generate report
        let total_duration = start_time.elapsed();
        self.generate_chaos_report(total_duration).await
    }

    /// Start failure injection background task
    fn start_failure_injector(&self) -> tokio::task::JoinHandle<()> {
        let config = self.config.clone();
        let active_failures = self.active_failures.clone();
        let metrics = self.metrics.clone();
        let system = self.system_under_test.clone();

        tokio::spawn(async move {
            let mut rng = rand::thread_rng();
            let injection_interval = Duration::from_secs(10); // Check every 10 seconds

            loop {
                sleep(injection_interval).await;

                // Check if we should inject a failure
                if rng.gen::<f64>() < config.failure_injection_rate {
                    let active_count = active_failures.read().await.len();
                    
                    if active_count < config.max_concurrent_failures {
                        if let Some(failure_type) = Self::select_random_failure(&config, &mut rng) {
                            if let Err(e) = Self::inject_failure(
                                &system,
                                failure_type,
                                &active_failures,
                                &metrics,
                            ).await {
                                error!("Failed to inject failure: {}", e);
                            }
                        }
                    }
                }
            }
        })
    }

    /// Start load generation background task
    fn start_load_generator(&self) -> tokio::task::JoinHandle<()> {
        let system = self.system_under_test.clone();
        let metrics = self.metrics.clone();

        tokio::spawn(async move {
            let request_interval = Duration::from_millis(100); // 10 RPS

            loop {
                let start_time = Instant::now();
                
                match system.process_request().await {
                    Ok(latency) => {
                        let mut metrics_guard = metrics.write().await;
                        metrics_guard.total_requests += 1;
                        metrics_guard.successful_requests += 1;
                        
                        let latency_ms = latency.as_millis() as f64;
                        metrics_guard.avg_latency_ms = 
                            (metrics_guard.avg_latency_ms * (metrics_guard.successful_requests - 1) as f64 + latency_ms) 
                            / metrics_guard.successful_requests as f64;
                        metrics_guard.max_latency_ms = metrics_guard.max_latency_ms.max(latency_ms);
                    },
                    Err(_) => {
                        let mut metrics_guard = metrics.write().await;
                        metrics_guard.total_requests += 1;
                        metrics_guard.failed_requests += 1;
                    }
                }

                sleep(request_interval).await;
            }
        })
    }

    /// Start health monitoring background task
    fn start_health_monitor(&self) -> tokio::task::JoinHandle<()> {
        let system = self.system_under_test.clone();
        let metrics = self.metrics.clone();

        tokio::spawn(async move {
            let health_check_interval = Duration::from_secs(5);
            let mut last_health_check = Instant::now();

            loop {
                sleep(health_check_interval).await;

                match system.health_check().await {
                    Ok(healthy) => {
                        if !healthy {
                            let downtime = last_health_check.elapsed();
                            let mut metrics_guard = metrics.write().await;
                            metrics_guard.system_downtime_ms += downtime.as_millis() as u64;
                        }
                        last_health_check = Instant::now();
                    },
                    Err(_) => {
                        // Health check failed, system is down
                        let downtime = last_health_check.elapsed();
                        let mut metrics_guard = metrics.write().await;
                        metrics_guard.system_downtime_ms += downtime.as_millis() as u64;
                    }
                }
            }
        })
    }

    /// Start recovery management background task
    fn start_recovery_manager(&self) -> tokio::task::JoinHandle<()> {
        let config = self.config.clone();
        let active_failures = self.active_failures.clone();
        let metrics = self.metrics.clone();
        let system = self.system_under_test.clone();

        tokio::spawn(async move {
            let recovery_check_interval = Duration::from_secs(5);

            loop {
                sleep(recovery_check_interval).await;

                let mut failures_to_recover = Vec::new();
                
                // Check for failures that should be recovered
                {
                    let failures = active_failures.read().await;
                    for (index, failure) in failures.iter().enumerate() {
                        if failure.start_time.elapsed() > Duration::from_secs(config.recovery_timeout_seconds) {
                            failures_to_recover.push((index, failure.failure_type.clone()));
                        }
                    }
                }

                // Recover failures
                for (index, failure_type) in failures_to_recover {
                    if let Err(e) = Self::recover_failure(
                        &system,
                        failure_type,
                        index,
                        &active_failures,
                        &metrics,
                    ).await {
                        error!("Failed to recover from failure: {}", e);
                    }
                }
            }
        })
    }

    /// Select random failure type based on configuration
    fn select_random_failure(config: &ChaosConfig, rng: &mut impl Rng) -> Option<FailureType> {
        let mut available_failures = Vec::new();

        if config.enable_network_failures {
            available_failures.extend_from_slice(&[
                FailureType::NetworkPartition,
                FailureType::HighLatency,
                FailureType::SolanaRpcDown,
                FailureType::HeliusDown,
            ]);
        }

        if config.enable_memory_pressure {
            available_failures.push(FailureType::MemoryPressure);
        }

        if config.enable_disk_failures {
            available_failures.push(FailureType::DiskFull);
        }

        if config.enable_dependency_failures {
            available_failures.extend_from_slice(&[
                FailureType::DatabaseDown,
                FailureType::RedisDown,
            ]);
        }

        if available_failures.is_empty() {
            return None;
        }

        let index = rng.gen_range(0..available_failures.len());
        Some(available_failures[index].clone())
    }

    /// Inject a specific failure
    async fn inject_failure(
        system: &Arc<dyn SystemUnderTest>,
        failure_type: FailureType,
        active_failures: &Arc<RwLock<Vec<FailureInjection>>>,
        metrics: &Arc<RwLock<ChaosMetrics>>,
    ) -> Result<()> {
        info!("💥 Injecting failure: {:?}", failure_type);

        system.inject_failure(failure_type.clone()).await?;

        let failure_injection = FailureInjection {
            failure_type,
            start_time: Instant::now(),
            duration: Duration::from_secs(0), // Will be updated on recovery
            recovered: false,
            recovery_time: None,
        };

        active_failures.write().await.push(failure_injection);
        metrics.write().await.failures_injected += 1;

        Ok(())
    }

    /// Recover from a specific failure
    async fn recover_failure(
        system: &Arc<dyn SystemUnderTest>,
        failure_type: FailureType,
        failure_index: usize,
        active_failures: &Arc<RwLock<Vec<FailureInjection>>>,
        metrics: &Arc<RwLock<ChaosMetrics>>,
    ) -> Result<()> {
        info!("🔧 Recovering from failure: {:?}", failure_type);

        system.recover_from_failure(failure_type).await?;

        // Update failure record
        {
            let mut failures = active_failures.write().await;
            if failure_index < failures.len() {
                let failure = &mut failures[failure_index];
                failure.recovered = true;
                failure.duration = failure.start_time.elapsed();
                failure.recovery_time = Some(failure.duration);
            }
        }

        // Remove recovered failure
        active_failures.write().await.remove(failure_index);
        metrics.write().await.failures_recovered += 1;

        Ok(())
    }

    /// Generate comprehensive chaos test report
    async fn generate_chaos_report(&self, total_duration: Duration) -> Result<ChaosTestReport> {
        let metrics = self.metrics.read().await.clone();
        let active_failures = self.active_failures.read().await.clone();

        let success_rate = if metrics.total_requests > 0 {
            metrics.successful_requests as f64 / metrics.total_requests as f64
        } else {
            0.0
        };

        let availability = if total_duration.as_millis() > 0 {
            1.0 - (metrics.system_downtime_ms as f64 / total_duration.as_millis() as f64)
        } else {
            1.0
        };

        let recovery_rate = if metrics.failures_injected > 0 {
            metrics.failures_recovered as f64 / metrics.failures_injected as f64
        } else {
            1.0
        };

        Ok(ChaosTestReport {
            test_duration: total_duration,
            total_requests: metrics.total_requests,
            successful_requests: metrics.successful_requests,
            failed_requests: metrics.failed_requests,
            success_rate,
            avg_latency_ms: metrics.avg_latency_ms,
            max_latency_ms: metrics.max_latency_ms,
            failures_injected: metrics.failures_injected,
            failures_recovered: metrics.failures_recovered,
            recovery_rate,
            system_availability: availability,
            active_failures_at_end: active_failures.len() as u64,
            passed: success_rate > 0.95 && availability > 0.99 && recovery_rate > 0.9,
        })
    }
}

/// Chaos test report
#[derive(Debug)]
pub struct ChaosTestReport {
    pub test_duration: Duration,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub max_latency_ms: f64,
    pub failures_injected: u64,
    pub failures_recovered: u64,
    pub recovery_rate: f64,
    pub system_availability: f64,
    pub active_failures_at_end: u64,
    pub passed: bool,
}

impl ChaosTestReport {
    pub fn print_summary(&self) {
        println!("\n🌪️ CHAOS TEST REPORT");
        println!("==================");
        println!("Test Duration: {:.2}s", self.test_duration.as_secs_f64());
        println!("Total Requests: {}", self.total_requests);
        println!("Success Rate: {:.2}%", self.success_rate * 100.0);
        println!("Avg Latency: {:.2}ms", self.avg_latency_ms);
        println!("Max Latency: {:.2}ms", self.max_latency_ms);
        println!("Failures Injected: {}", self.failures_injected);
        println!("Recovery Rate: {:.2}%", self.recovery_rate * 100.0);
        println!("System Availability: {:.2}%", self.system_availability * 100.0);
        println!("Active Failures at End: {}", self.active_failures_at_end);
        
        if self.passed {
            println!("✅ CHAOS TEST PASSED");
        } else {
            println!("❌ CHAOS TEST FAILED");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockSystem {
        healthy: Arc<RwLock<bool>>,
        injected_failures: Arc<RwLock<Vec<FailureType>>>,
    }

    impl MockSystem {
        fn new() -> Self {
            Self {
                healthy: Arc::new(RwLock::new(true)),
                injected_failures: Arc::new(RwLock::new(Vec::new())),
            }
        }
    }

    #[async_trait::async_trait]
    impl SystemUnderTest for MockSystem {
        async fn health_check(&self) -> Result<bool> {
            Ok(*self.healthy.read().await)
        }

        async fn process_request(&self) -> Result<Duration> {
            if *self.healthy.read().await {
                Ok(Duration::from_millis(50))
            } else {
                Err(anyhow::anyhow!("System unhealthy"))
            }
        }

        async fn get_metrics(&self) -> Result<SystemMetrics> {
            Ok(SystemMetrics {
                cpu_usage: 50.0,
                memory_usage: 60.0,
                disk_usage: 30.0,
                network_latency: 10.0,
                active_connections: 100,
            })
        }

        async fn inject_failure(&self, failure_type: FailureType) -> Result<()> {
            *self.healthy.write().await = false;
            self.injected_failures.write().await.push(failure_type);
            Ok(())
        }

        async fn recover_from_failure(&self, failure_type: FailureType) -> Result<()> {
            *self.healthy.write().await = true;
            let mut failures = self.injected_failures.write().await;
            if let Some(pos) = failures.iter().position(|f| *f == failure_type) {
                failures.remove(pos);
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_chaos_engine() {
        let config = ChaosConfig {
            test_duration_seconds: 10,
            failure_injection_rate: 0.5,
            recovery_timeout_seconds: 5,
            ..Default::default()
        };

        let mock_system = Arc::new(MockSystem::new());
        let chaos_engine = ChaosEngine::new(config, mock_system);

        let report = chaos_engine.run_chaos_test().await.unwrap();
        
        assert!(report.total_requests > 0);
        assert!(report.test_duration.as_secs() >= 10);
        // System should recover from failures
        assert!(report.recovery_rate > 0.0);
    }
}
