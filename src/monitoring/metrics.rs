//! Prometheus Metrics System
//! 
//! Comprehensive monitoring and metrics collection for HFT system

use anyhow::Result;
use prometheus::{
    Counter, Gauge, Histogram, IntCounter, IntGauge, 
    register_counter, register_gauge, register_histogram, 
    register_int_counter, register_int_gauge,
    Encoder, TextEncoder, Registry,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};
use std::time::{Duration, Instant};

/// Metrics collector for HFT system
pub struct HftMetrics {
    // Transaction metrics
    pub transactions_processed: IntCounter,
    pub transactions_failed: IntCounter,
    pub dex_transactions_detected: IntCounter,
    pub mev_opportunities_found: IntCounter,
    
    // Performance metrics
    pub transaction_processing_time: Histogram,
    pub mempool_latency: Histogram,
    pub execution_latency: Histogram,
    pub bridge_queue_size: IntGauge,
    
    // Trading metrics
    pub trades_executed: IntCounter,
    pub trades_successful: IntCounter,
    pub trades_failed: IntCounter,
    pub total_volume_sol: Gauge,
    pub total_profit_sol: Gauge,
    pub total_loss_sol: Gauge,
    
    // MEV metrics
    pub sandwich_opportunities: IntCounter,
    pub arbitrage_opportunities: IntCounter,
    pub liquidation_opportunities: IntCounter,
    pub mev_profit_sol: Gauge,
    
    // System metrics
    pub memory_usage_bytes: IntGauge,
    pub cpu_usage_percent: Gauge,
    pub active_connections: IntGauge,
    pub error_rate: Gauge,

    // Security metrics
    pub circuit_breaker_state: IntGauge,
    pub wallet_locked: IntGauge,
    pub daily_loss_ratio: Gauge,
    pub position_utilization: Gauge,
    pub consecutive_failures: IntGauge,
    pub security_events_total: IntCounter,
    pub failed_logins_total: IntCounter,
    pub emergency_events_total: IntCounter,
    
    // Jito metrics
    pub bundles_submitted: IntCounter,
    pub bundles_confirmed: IntCounter,
    pub bundles_failed: IntCounter,
    pub bundle_confirmation_time: Histogram,
    pub tip_amount_sol: Gauge,
    
    registry: Registry,
}

impl HftMetrics {
    /// Create new metrics collector
    pub fn new() -> Result<Self> {
        let registry = Registry::new();
        
        // Transaction metrics
        let transactions_processed = register_int_counter!(
            "hft_transactions_processed_total",
            "Total number of transactions processed"
        )?;
        
        let transactions_failed = register_int_counter!(
            "hft_transactions_failed_total",
            "Total number of failed transactions"
        )?;
        
        let dex_transactions_detected = register_int_counter!(
            "hft_dex_transactions_detected_total",
            "Total number of DEX transactions detected"
        )?;
        
        let mev_opportunities_found = register_int_counter!(
            "hft_mev_opportunities_found_total",
            "Total number of MEV opportunities found"
        )?;
        
        // Performance metrics
        let transaction_processing_time = register_histogram!(
            "hft_transaction_processing_seconds",
            "Time spent processing transactions",
            vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]
        )?;
        
        let mempool_latency = register_histogram!(
            "hft_mempool_latency_seconds",
            "Latency from mempool to processing",
            vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]
        )?;
        
        let execution_latency = register_histogram!(
            "hft_execution_latency_seconds",
            "Trade execution latency",
            vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0, 10.0]
        )?;
        
        let bridge_queue_size = register_int_gauge!(
            "hft_bridge_queue_size",
            "Current size of bridge message queue"
        )?;
        
        // Trading metrics
        let trades_executed = register_int_counter!(
            "hft_trades_executed_total",
            "Total number of trades executed"
        )?;
        
        let trades_successful = register_int_counter!(
            "hft_trades_successful_total",
            "Total number of successful trades"
        )?;
        
        let trades_failed = register_int_counter!(
            "hft_trades_failed_total",
            "Total number of failed trades"
        )?;
        
        let total_volume_sol = register_gauge!(
            "hft_total_volume_sol",
            "Total trading volume in SOL"
        )?;
        
        let total_profit_sol = register_gauge!(
            "hft_total_profit_sol",
            "Total profit in SOL"
        )?;
        
        let total_loss_sol = register_gauge!(
            "hft_total_loss_sol",
            "Total loss in SOL"
        )?;
        
        // MEV metrics
        let sandwich_opportunities = register_int_counter!(
            "hft_sandwich_opportunities_total",
            "Total sandwich opportunities detected"
        )?;
        
        let arbitrage_opportunities = register_int_counter!(
            "hft_arbitrage_opportunities_total",
            "Total arbitrage opportunities detected"
        )?;
        
        let liquidation_opportunities = register_int_counter!(
            "hft_liquidation_opportunities_total",
            "Total liquidation opportunities detected"
        )?;
        
        let mev_profit_sol = register_gauge!(
            "hft_mev_profit_sol",
            "Total MEV profit in SOL"
        )?;
        
        // System metrics
        let memory_usage_bytes = register_int_gauge!(
            "hft_memory_usage_bytes",
            "Current memory usage in bytes"
        )?;
        
        let cpu_usage_percent = register_gauge!(
            "hft_cpu_usage_percent",
            "Current CPU usage percentage"
        )?;
        
        let active_connections = register_int_gauge!(
            "hft_active_connections",
            "Number of active connections"
        )?;
        
        let error_rate = register_gauge!(
            "hft_error_rate",
            "Current error rate (errors per second)"
        )?;
        
        // Jito metrics
        let bundles_submitted = register_int_counter!(
            "hft_bundles_submitted_total",
            "Total number of Jito bundles submitted"
        )?;
        
        let bundles_confirmed = register_int_counter!(
            "hft_bundles_confirmed_total",
            "Total number of Jito bundles confirmed"
        )?;
        
        let bundles_failed = register_int_counter!(
            "hft_bundles_failed_total",
            "Total number of Jito bundles failed"
        )?;
        
        let bundle_confirmation_time = register_histogram!(
            "hft_bundle_confirmation_seconds",
            "Time for bundle confirmation",
            vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0]
        )?;
        
        let tip_amount_sol = register_gauge!(
            "hft_tip_amount_sol",
            "Current tip amount in SOL"
        )?;

        // Security metrics
        let circuit_breaker_state = register_int_gauge!(
            "hft_circuit_breaker_state",
            "Circuit breaker state (0=closed, 1=open, 2=half-open)"
        )?;

        let wallet_locked = register_int_gauge!(
            "hft_wallet_locked",
            "Wallet lock status (0=unlocked, 1=locked)"
        )?;

        let daily_loss_ratio = register_gauge!(
            "hft_daily_loss_ratio",
            "Daily loss ratio (0.0-1.0)"
        )?;

        let position_utilization = register_gauge!(
            "hft_position_utilization",
            "Position utilization ratio (0.0-1.0)"
        )?;

        let consecutive_failures = register_int_gauge!(
            "hft_consecutive_failures",
            "Number of consecutive failures"
        )?;

        let security_events_total = register_int_counter!(
            "hft_security_events_total",
            "Total number of security events"
        )?;

        let failed_logins_total = register_int_counter!(
            "hft_failed_logins_total",
            "Total number of failed login attempts"
        )?;

        let emergency_events_total = register_int_counter!(
            "hft_emergency_events_total",
            "Total number of emergency events"
        )?;

        Ok(Self {
            transactions_processed,
            transactions_failed,
            dex_transactions_detected,
            mev_opportunities_found,
            transaction_processing_time,
            mempool_latency,
            execution_latency,
            bridge_queue_size,
            trades_executed,
            trades_successful,
            trades_failed,
            total_volume_sol,
            total_profit_sol,
            total_loss_sol,
            sandwich_opportunities,
            arbitrage_opportunities,
            liquidation_opportunities,
            mev_profit_sol,
            memory_usage_bytes,
            cpu_usage_percent,
            active_connections,
            error_rate,
            bundles_submitted,
            bundles_confirmed,
            bundles_failed,
            bundle_confirmation_time,
            tip_amount_sol,
            circuit_breaker_state,
            wallet_locked,
            daily_loss_ratio,
            position_utilization,
            consecutive_failures,
            security_events_total,
            failed_logins_total,
            emergency_events_total,
            registry,
        })
    }
    
    /// Record transaction processing time
    pub fn record_transaction_processing_time(&self, duration: Duration) {
        self.transaction_processing_time.observe(duration.as_secs_f64());
    }
    
    /// Record mempool latency
    pub fn record_mempool_latency(&self, duration: Duration) {
        self.mempool_latency.observe(duration.as_secs_f64());
    }
    
    /// Record execution latency
    pub fn record_execution_latency(&self, duration: Duration) {
        self.execution_latency.observe(duration.as_secs_f64());
    }
    
    /// Record bundle confirmation time
    pub fn record_bundle_confirmation_time(&self, duration: Duration) {
        self.bundle_confirmation_time.observe(duration.as_secs_f64());
    }
    
    /// Update system metrics
    pub fn update_system_metrics(&self) {
        // Update memory usage
        if let Ok(memory) = self.get_memory_usage() {
            self.memory_usage_bytes.set(memory as i64);
        }
        
        // Update CPU usage
        if let Ok(cpu) = self.get_cpu_usage() {
            self.cpu_usage_percent.set(cpu);
        }
    }
    
    /// Get current memory usage
    fn get_memory_usage(&self) -> Result<u64> {
        // Simplified - in reality would use system APIs
        Ok(1024 * 1024 * 100) // 100MB
    }
    
    /// Get current CPU usage
    fn get_cpu_usage(&self) -> Result<f64> {
        // Simplified - in reality would use system APIs
        Ok(25.0) // 25%
    }
    
    /// Update circuit breaker state
    pub fn update_circuit_breaker_state(&self, state: i64) {
        self.circuit_breaker_state.set(state);
    }

    /// Update wallet lock status
    pub fn update_wallet_locked(&self, locked: bool) {
        self.wallet_locked.set(if locked { 1 } else { 0 });
    }

    /// Update daily loss ratio
    pub fn update_daily_loss_ratio(&self, ratio: f64) {
        self.daily_loss_ratio.set(ratio);
    }

    /// Update position utilization
    pub fn update_position_utilization(&self, ratio: f64) {
        self.position_utilization.set(ratio);
    }

    /// Update consecutive failures
    pub fn update_consecutive_failures(&self, count: i64) {
        self.consecutive_failures.set(count);
    }

    /// Record security event
    pub fn record_security_event(&self) {
        self.security_events_total.inc();
    }

    /// Record failed login
    pub fn record_failed_login(&self) {
        self.failed_logins_total.inc();
    }

    /// Record emergency event
    pub fn record_emergency_event(&self) {
        self.emergency_events_total.inc();
    }

    /// Export metrics in Prometheus format
    pub fn export_metrics(&self) -> Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
}

/// Metrics server for Prometheus scraping
pub struct MetricsServer {
    metrics: Arc<HftMetrics>,
    port: u16,
}

impl MetricsServer {
    /// Create new metrics server
    pub fn new(metrics: Arc<HftMetrics>, port: u16) -> Self {
        Self { metrics, port }
    }
    
    /// Start metrics server
    pub async fn start(&self) -> Result<()> {
        use warp::Filter;
        
        let metrics = self.metrics.clone();
        let metrics_route = warp::path("metrics")
            .map(move || {
                match metrics.export_metrics() {
                    Ok(metrics_text) => {
                        warp::reply::with_header(
                            metrics_text,
                            "content-type",
                            "text/plain; version=0.0.4; charset=utf-8"
                        )
                    }
                    Err(e) => {
                        error!("Failed to export metrics: {}", e);
                        warp::reply::with_header(
                            "Error exporting metrics".to_string(),
                            "content-type",
                            "text/plain"
                        )
                    }
                }
            });
        
        let health_route = warp::path("health")
            .map(|| "OK");
        
        let routes = metrics_route.or(health_route);
        
        info!("🔍 Starting metrics server on port {}", self.port);
        
        warp::serve(routes)
            .run(([0, 0, 0, 0], self.port))
            .await;
        
        Ok(())
    }
}

/// Create metrics instance
pub fn create_metrics() -> Result<Arc<HftMetrics>> {
    let metrics = HftMetrics::new()?;
    Ok(Arc::new(metrics))
}

/// Start metrics collection background task
pub async fn start_metrics_collection(metrics: Arc<HftMetrics>) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    
    loop {
        interval.tick().await;
        metrics.update_system_metrics();
    }
}
