//! Prometheus Metrics System
//! 
//! Comprehensive monitoring and metrics collection for HFT system

use anyhow::Result;
use prometheus::{
    Counter, Gauge, Histogram, IntCounter, IntGauge, HistogramOpts,
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
        let transactions_processed = IntCounter::new(
            "hft_transactions_processed_total",
            "Total number of transactions processed"
        )?;
        registry.register(Box::new(transactions_processed.clone()))?;

        let transactions_failed = IntCounter::new(
            "hft_transactions_failed_total",
            "Total number of failed transactions"
        )?;
        registry.register(Box::new(transactions_failed.clone()))?;

        let dex_transactions_detected = IntCounter::new(
            "hft_dex_transactions_detected_total",
            "Total number of DEX transactions detected"
        )?;
        registry.register(Box::new(dex_transactions_detected.clone()))?;

        let mev_opportunities_found = IntCounter::new(
            "hft_mev_opportunities_found_total",
            "Total number of MEV opportunities found"
        )?;
        registry.register(Box::new(mev_opportunities_found.clone()))?;
        
        // Performance metrics
        let transaction_processing_time = Histogram::with_opts(
            HistogramOpts::new(
                "hft_transaction_processing_seconds",
                "Time spent processing transactions"
            ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0])
        )?;
        registry.register(Box::new(transaction_processing_time.clone()))?;

        let mempool_latency = Histogram::with_opts(
            HistogramOpts::new(
                "hft_mempool_latency_seconds",
                "Latency from mempool to processing"
            ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0])
        )?;
        registry.register(Box::new(mempool_latency.clone()))?;

        let execution_latency = Histogram::with_opts(
            HistogramOpts::new(
                "hft_execution_latency_seconds",
                "Trade execution latency"
            ).buckets(vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0, 10.0])
        )?;
        registry.register(Box::new(execution_latency.clone()))?;

        let bridge_queue_size = IntGauge::new(
            "hft_bridge_queue_size",
            "Current size of bridge message queue"
        )?;
        registry.register(Box::new(bridge_queue_size.clone()))?;
        
        // Trading metrics
        let trades_executed = IntCounter::new(
            "hft_trades_executed_total",
            "Total number of trades executed"
        )?;
        registry.register(Box::new(trades_executed.clone()))?;

        let trades_successful = IntCounter::new(
            "hft_trades_successful_total",
            "Total number of successful trades"
        )?;
        registry.register(Box::new(trades_successful.clone()))?;

        let trades_failed = IntCounter::new(
            "hft_trades_failed_total",
            "Total number of failed trades"
        )?;
        registry.register(Box::new(trades_failed.clone()))?;

        let total_volume_sol = Gauge::new(
            "hft_total_volume_sol",
            "Total trading volume in SOL"
        )?;
        registry.register(Box::new(total_volume_sol.clone()))?;

        let total_profit_sol = Gauge::new(
            "hft_total_profit_sol",
            "Total profit in SOL"
        )?;
        registry.register(Box::new(total_profit_sol.clone()))?;

        let total_loss_sol = Gauge::new(
            "hft_total_loss_sol",
            "Total loss in SOL"
        )?;
        registry.register(Box::new(total_loss_sol.clone()))?;
        
        // MEV metrics
        let sandwich_opportunities = IntCounter::new(
            "hft_sandwich_opportunities_total",
            "Total sandwich opportunities detected"
        )?;
        registry.register(Box::new(sandwich_opportunities.clone()))?;

        let arbitrage_opportunities = IntCounter::new(
            "hft_arbitrage_opportunities_total",
            "Total arbitrage opportunities detected"
        )?;
        registry.register(Box::new(arbitrage_opportunities.clone()))?;

        let liquidation_opportunities = IntCounter::new(
            "hft_liquidation_opportunities_total",
            "Total liquidation opportunities detected"
        )?;
        registry.register(Box::new(liquidation_opportunities.clone()))?;

        let mev_profit_sol = Gauge::new(
            "hft_mev_profit_sol",
            "Total MEV profit in SOL"
        )?;
        registry.register(Box::new(mev_profit_sol.clone()))?;
        
        // System metrics
        let memory_usage_bytes = IntGauge::new(
            "hft_memory_usage_bytes",
            "Current memory usage in bytes"
        )?;
        registry.register(Box::new(memory_usage_bytes.clone()))?;

        let cpu_usage_percent = Gauge::new(
            "hft_cpu_usage_percent",
            "Current CPU usage percentage"
        )?;
        registry.register(Box::new(cpu_usage_percent.clone()))?;

        let active_connections = IntGauge::new(
            "hft_active_connections",
            "Number of active connections"
        )?;
        registry.register(Box::new(active_connections.clone()))?;

        let error_rate = Gauge::new(
            "hft_error_rate",
            "Current error rate (errors per second)"
        )?;
        registry.register(Box::new(error_rate.clone()))?;
        
        // Jito metrics
        let bundles_submitted = IntCounter::new(
            "hft_bundles_submitted_total",
            "Total number of Jito bundles submitted"
        )?;
        registry.register(Box::new(bundles_submitted.clone()))?;

        let bundles_confirmed = IntCounter::new(
            "hft_bundles_confirmed_total",
            "Total number of Jito bundles confirmed"
        )?;
        registry.register(Box::new(bundles_confirmed.clone()))?;

        let bundles_failed = IntCounter::new(
            "hft_bundles_failed_total",
            "Total number of Jito bundles failed"
        )?;
        registry.register(Box::new(bundles_failed.clone()))?;

        let bundle_confirmation_time = Histogram::with_opts(
            HistogramOpts::new(
                "hft_bundle_confirmation_seconds",
                "Time for bundle confirmation"
            ).buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0])
        )?;
        registry.register(Box::new(bundle_confirmation_time.clone()))?;

        let tip_amount_sol = Gauge::new(
            "hft_tip_amount_sol",
            "Current tip amount in SOL"
        )?;
        registry.register(Box::new(tip_amount_sol.clone()))?;

        // Security metrics
        let circuit_breaker_state = IntGauge::new(
            "hft_circuit_breaker_state",
            "Circuit breaker state (0=closed, 1=open, 2=half-open)"
        )?;
        registry.register(Box::new(circuit_breaker_state.clone()))?;

        let wallet_locked = IntGauge::new(
            "hft_wallet_locked",
            "Wallet lock status (0=unlocked, 1=locked)"
        )?;
        registry.register(Box::new(wallet_locked.clone()))?;

        let daily_loss_ratio = Gauge::new(
            "hft_daily_loss_ratio",
            "Daily loss ratio (0.0-1.0)"
        )?;
        registry.register(Box::new(daily_loss_ratio.clone()))?;

        let position_utilization = Gauge::new(
            "hft_position_utilization",
            "Position utilization ratio (0.0-1.0)"
        )?;
        registry.register(Box::new(position_utilization.clone()))?;

        let consecutive_failures = IntGauge::new(
            "hft_consecutive_failures",
            "Number of consecutive failures"
        )?;
        registry.register(Box::new(consecutive_failures.clone()))?;

        let security_events_total = IntCounter::new(
            "hft_security_events_total",
            "Total number of security events"
        )?;
        registry.register(Box::new(security_events_total.clone()))?;

        let failed_logins_total = IntCounter::new(
            "hft_failed_logins_total",
            "Total number of failed login attempts"
        )?;
        registry.register(Box::new(failed_logins_total.clone()))?;

        let emergency_events_total = IntCounter::new(
            "hft_emergency_events_total",
            "Total number of emergency events"
        )?;
        registry.register(Box::new(emergency_events_total.clone()))?;

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
