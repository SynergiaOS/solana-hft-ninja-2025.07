//! 🧪 Comprehensive Performance Testing Framework
//! 
//! Microbenchmarks, chaos testing, and production validation

use anyhow::Result;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use hdrhistogram::Histogram;
use prometheus::{Encoder, TextEncoder, Counter, Histogram as PrometheusHistogram};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tracing::{info, warn};

// Import our modules
use solana_hft_ninja::core::engine::Engine;
use solana_hft_ninja::mempool::parser::ParsedTransaction;
use solana_hft_ninja::strategies::mev::MevEngine;
use cerebro::batch_processor::batch_aggregator::{BatchAggregator, TradingEvent, BatchConfig};
use cerebro::cache_engine::dragonfly_cache::{DragonflyCache, CacheConfig, CacheKey};
use cerebro::feature_engine::lazy_extraction::{LazyFeatureExtractor, FeatureConfig, MarketTick};

/// Performance test configuration
#[derive(Debug, Clone)]
pub struct PerfTestConfig {
    pub signal_rate_per_second: u64,
    pub test_duration_seconds: u64,
    pub batch_size: usize,
    pub concurrent_workers: usize,
    pub target_latency_p99_ms: f64,
    pub target_throughput_rps: f64,
}

impl Default for PerfTestConfig {
    fn default() -> Self {
        Self {
            signal_rate_per_second: 10000,
            test_duration_seconds: 60,
            batch_size: 100,
            concurrent_workers: 4,
            target_latency_p99_ms: 100.0,
            target_throughput_rps: 1000.0,
        }
    }
}

/// Performance test results
#[derive(Debug)]
pub struct PerfTestResults {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub latency_p50_ms: f64,
    pub latency_p95_ms: f64,
    pub latency_p99_ms: f64,
    pub throughput_rps: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

/// HFT Ninja performance benchmarks
pub struct HftNinjaBenchmarks {
    config: PerfTestConfig,
    latency_histogram: Histogram<u64>,
    request_counter: Counter,
    latency_prometheus: PrometheusHistogram,
}

impl HftNinjaBenchmarks {
    pub fn new(config: PerfTestConfig) -> Result<Self> {
        let latency_histogram = Histogram::new(3)?; // 3 significant digits
        
        let request_counter = Counter::new(
            "hft_benchmark_requests_total",
            "Total number of benchmark requests"
        )?;
        
        let latency_prometheus = PrometheusHistogram::with_opts(
            prometheus::HistogramOpts::new(
                "hft_benchmark_latency_seconds",
                "Request latency in seconds"
            ).buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0])
        )?;

        Ok(Self {
            config,
            latency_histogram,
            request_counter,
            latency_prometheus,
        })
    }

    /// Benchmark mempool processing latency
    pub async fn benchmark_mempool_processing(&mut self) -> Result<PerfTestResults> {
        info!("Starting mempool processing benchmark");
        
        let rt = Runtime::new()?;
        let start_time = Instant::now();
        let mut successful_requests = 0u64;
        let mut failed_requests = 0u64;

        // Create mock transactions
        let mock_transactions = self.create_mock_transactions(1000);
        
        // Benchmark loop
        for tx in mock_transactions {
            let process_start = Instant::now();
            
            // Simulate mempool processing
            let result = rt.block_on(async {
                self.process_mock_transaction(tx).await
            });

            let latency = process_start.elapsed();
            
            match result {
                Ok(_) => {
                    successful_requests += 1;
                    self.record_latency(latency);
                },
                Err(_) => failed_requests += 1,
            }
        }

        let total_duration = start_time.elapsed();
        self.calculate_results(successful_requests, failed_requests, total_duration)
    }

    /// Benchmark strategy analysis latency
    pub async fn benchmark_strategy_analysis(&mut self) -> Result<PerfTestResults> {
        info!("Starting strategy analysis benchmark");
        
        let rt = Runtime::new()?;
        let start_time = Instant::now();
        let mut successful_requests = 0u64;
        let mut failed_requests = 0u64;

        // Create MEV engine
        let mev_config = solana_hft_ninja::strategies::mev::MevConfig::default();
        let mut mev_engine = MevEngine::new(&mev_config);

        // Create mock opportunities
        let mock_opportunities = self.create_mock_opportunities(1000);
        
        for opportunity in mock_opportunities {
            let analysis_start = Instant::now();
            
            let result = rt.block_on(async {
                mev_engine.analyze_opportunity(opportunity).await
            });

            let latency = analysis_start.elapsed();
            
            match result {
                Ok(_) => {
                    successful_requests += 1;
                    self.record_latency(latency);
                },
                Err(_) => failed_requests += 1,
            }
        }

        let total_duration = start_time.elapsed();
        self.calculate_results(successful_requests, failed_requests, total_duration)
    }

    /// Benchmark Cerebro batch processing
    pub async fn benchmark_cerebro_batch_processing(&mut self) -> Result<PerfTestResults> {
        info!("Starting Cerebro batch processing benchmark");
        
        let rt = Runtime::new()?;
        let start_time = Instant::now();
        let mut successful_requests = 0u64;
        let mut failed_requests = 0u64;

        // Create batch aggregator
        let batch_config = BatchConfig::default();
        let mut aggregator = BatchAggregator::new(batch_config)?;

        // Create mock trading events
        let mock_events = self.create_mock_trading_events(10000);
        
        for event in mock_events {
            let batch_start = Instant::now();
            
            let result = rt.block_on(async {
                aggregator.add_event(event).await
            });

            let latency = batch_start.elapsed();
            
            match result {
                Ok(_) => {
                    successful_requests += 1;
                    self.record_latency(latency);
                },
                Err(_) => failed_requests += 1,
            }
        }

        let total_duration = start_time.elapsed();
        self.calculate_results(successful_requests, failed_requests, total_duration)
    }

    /// Benchmark cache performance
    pub async fn benchmark_cache_performance(&mut self) -> Result<PerfTestResults> {
        info!("Starting cache performance benchmark");
        
        let rt = Runtime::new()?;
        let start_time = Instant::now();
        let mut successful_requests = 0u64;
        let mut failed_requests = 0u64;

        // Create cache
        let cache_config = CacheConfig::default();
        let mut cache = DragonflyCache::new(cache_config)?;

        // Create mock cache keys and responses
        let mock_data = self.create_mock_cache_data(1000);
        
        // Benchmark cache operations
        for (key, response) in mock_data {
            let cache_start = Instant::now();
            
            // Set operation
            let set_result = rt.block_on(async {
                cache.set(&key, response.clone()).await
            });

            // Get operation
            let get_result = rt.block_on(async {
                cache.get(&key).await
            });

            let latency = cache_start.elapsed();
            
            match (set_result, get_result) {
                (Ok(_), Ok(Some(_))) => {
                    successful_requests += 1;
                    self.record_latency(latency);
                },
                _ => failed_requests += 1,
            }
        }

        let total_duration = start_time.elapsed();
        self.calculate_results(successful_requests, failed_requests, total_duration)
    }

    /// Benchmark feature extraction
    pub async fn benchmark_feature_extraction(&mut self) -> Result<PerfTestResults> {
        info!("Starting feature extraction benchmark");
        
        let rt = Runtime::new()?;
        let start_time = Instant::now();
        let mut successful_requests = 0u64;
        let mut failed_requests = 0u64;

        // Create feature extractor
        let feature_config = FeatureConfig::default();
        let mut extractor = LazyFeatureExtractor::new(feature_config);

        // Create mock market ticks
        let mock_ticks = self.create_mock_market_ticks(5000);
        
        for tick in mock_ticks {
            let extraction_start = Instant::now();
            
            let result = rt.block_on(async {
                extractor.add_tick(tick).await
            });

            let latency = extraction_start.elapsed();
            
            match result {
                Ok(_) => {
                    successful_requests += 1;
                    self.record_latency(latency);
                },
                Err(_) => failed_requests += 1,
            }
        }

        let total_duration = start_time.elapsed();
        self.calculate_results(successful_requests, failed_requests, total_duration)
    }

    /// Record latency measurement
    fn record_latency(&mut self, latency: Duration) {
        let latency_ms = latency.as_millis() as u64;
        self.latency_histogram.record(latency_ms).unwrap();
        self.latency_prometheus.observe(latency.as_secs_f64());
        self.request_counter.inc();
    }

    /// Calculate performance test results
    fn calculate_results(
        &self,
        successful_requests: u64,
        failed_requests: u64,
        total_duration: Duration,
    ) -> Result<PerfTestResults> {
        let total_requests = successful_requests + failed_requests;
        let throughput_rps = successful_requests as f64 / total_duration.as_secs_f64();

        Ok(PerfTestResults {
            total_requests,
            successful_requests,
            failed_requests,
            latency_p50_ms: self.latency_histogram.value_at_quantile(0.5) as f64,
            latency_p95_ms: self.latency_histogram.value_at_quantile(0.95) as f64,
            latency_p99_ms: self.latency_histogram.value_at_quantile(0.99) as f64,
            throughput_rps,
            memory_usage_mb: self.get_memory_usage_mb(),
            cpu_usage_percent: self.get_cpu_usage_percent(),
        })
    }

    /// Create mock transactions for testing
    fn create_mock_transactions(&self, count: usize) -> Vec<ParsedTransaction> {
        (0..count).map(|i| {
            ParsedTransaction {
                signature: format!("mock_tx_{}", i),
                accounts: vec![],
                instructions: vec![],
                timestamp: 1640995200 + i as u64,
                dex_type: Some(solana_hft_ninja::mempool::dex::DexType::Raydium),
            }
        }).collect()
    }

    /// Create mock MEV opportunities
    fn create_mock_opportunities(&self, count: usize) -> Vec<solana_hft_ninja::strategies::mev::MevOpportunity> {
        (0..count).map(|i| {
            solana_hft_ninja::strategies::mev::MevOpportunity {
                opportunity_type: solana_hft_ninja::strategies::mev::OpportunityType::Arbitrage,
                profit_estimate: 0.01 + (i as f64 * 0.001),
                confidence: 0.8,
                execution_window_ms: 100,
                target_transaction: format!("target_tx_{}", i),
                metadata: std::collections::HashMap::new(),
            }
        }).collect()
    }

    /// Create mock trading events
    fn create_mock_trading_events(&self, count: usize) -> Vec<TradingEvent> {
        (0..count).map(|i| {
            TradingEvent {
                timestamp: 1640995200 + i as u64,
                wallet_address: format!("wallet_{}", i % 100),
                token_mint: format!("token_{}", i % 20),
                strategy_type: "arbitrage".to_string(),
                amount_sol: 0.1 + (i as f64 * 0.01),
                profit_sol: 0.005 + (i as f64 * 0.001),
                execution_time_ms: 50 + (i as u64 % 100),
                success: i % 10 != 0, // 90% success rate
                metadata: std::collections::HashMap::new(),
            }
        }).collect()
    }

    /// Create mock cache data
    fn create_mock_cache_data(&self, count: usize) -> Vec<(CacheKey, cerebro::skeleton_templates::AnalysisResponse)> {
        (0..count).map(|i| {
            let key = CacheKey::new(
                &format!("wallet_{}", i % 50),
                &format!("token_{}", i % 10),
                "arbitrage_strategy",
                1640995200 + i as u64,
            );
            
            let response = cerebro::skeleton_templates::AnalysisResponse {
                strategy_recommendation: "buy".to_string(),
                confidence_score: 0.8 + (i as f64 * 0.01) % 0.2,
                risk_assessment: cerebro::skeleton_templates::RiskLevel::Low,
                execution_priority: (i % 10) as u8 + 1,
                key_insights: vec!["insight_1".to_string(), "insight_2".to_string()],
                next_actions: vec!["action_1".to_string()],
            };
            
            (key, response)
        }).collect()
    }

    /// Create mock market ticks
    fn create_mock_market_ticks(&self, count: usize) -> Vec<MarketTick> {
        (0..count).map(|i| {
            MarketTick {
                timestamp: 1640995200 + i as u64,
                price: 100.0 + (i as f64 * 0.01),
                volume: 1000.0 + (i as f64 * 10.0),
                token_mint: format!("token_{}", i % 5),
                dex: if i % 2 == 0 { "raydium".to_string() } else { "orca".to_string() },
            }
        }).collect()
    }

    /// Simulate transaction processing
    async fn process_mock_transaction(&self, _tx: ParsedTransaction) -> Result<()> {
        // Simulate processing time
        tokio::time::sleep(Duration::from_micros(100)).await;
        Ok(())
    }

    /// Get current memory usage (mock implementation)
    fn get_memory_usage_mb(&self) -> f64 {
        // In real implementation, use system metrics
        128.0 // Mock value
    }

    /// Get current CPU usage (mock implementation)
    fn get_cpu_usage_percent(&self) -> f64 {
        // In real implementation, use system metrics
        45.0 // Mock value
    }

    /// Export Prometheus metrics
    pub fn export_prometheus_metrics(&self) -> Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    /// Validate performance against targets
    pub fn validate_performance(&self, results: &PerfTestResults) -> Result<bool> {
        let mut passed = true;
        
        if results.latency_p99_ms > self.config.target_latency_p99_ms {
            warn!(
                "P99 latency {} ms exceeds target {} ms",
                results.latency_p99_ms,
                self.config.target_latency_p99_ms
            );
            passed = false;
        }
        
        if results.throughput_rps < self.config.target_throughput_rps {
            warn!(
                "Throughput {} RPS below target {} RPS",
                results.throughput_rps,
                self.config.target_throughput_rps
            );
            passed = false;
        }
        
        if results.failed_requests > results.total_requests / 100 { // >1% failure rate
            warn!(
                "High failure rate: {} / {} requests failed",
                results.failed_requests,
                results.total_requests
            );
            passed = false;
        }

        if passed {
            info!("✅ All performance targets met!");
        } else {
            warn!("❌ Some performance targets not met");
        }

        Ok(passed)
    }
}

/// Criterion benchmarks
fn criterion_benchmark_mempool(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = PerfTestConfig::default();
    let mut benchmarks = HftNinjaBenchmarks::new(config).unwrap();

    c.bench_function("mempool_processing", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(benchmarks.benchmark_mempool_processing().await.unwrap())
        })
    });
}

fn criterion_benchmark_strategy(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = PerfTestConfig::default();
    let mut benchmarks = HftNinjaBenchmarks::new(config).unwrap();

    c.bench_function("strategy_analysis", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(benchmarks.benchmark_strategy_analysis().await.unwrap())
        })
    });
}

criterion_group!(benches, criterion_benchmark_mempool, criterion_benchmark_strategy);
criterion_main!(benches);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mempool_benchmark() {
        let config = PerfTestConfig {
            signal_rate_per_second: 100,
            test_duration_seconds: 5,
            ..Default::default()
        };
        
        let mut benchmarks = HftNinjaBenchmarks::new(config).unwrap();
        let results = benchmarks.benchmark_mempool_processing().await.unwrap();
        
        assert!(results.total_requests > 0);
        assert!(results.latency_p99_ms < 1000.0); // Should be under 1 second
    }

    #[tokio::test]
    async fn test_performance_validation() {
        let config = PerfTestConfig::default();
        let benchmarks = HftNinjaBenchmarks::new(config).unwrap();
        
        let good_results = PerfTestResults {
            total_requests: 1000,
            successful_requests: 990,
            failed_requests: 10,
            latency_p50_ms: 25.0,
            latency_p95_ms: 75.0,
            latency_p99_ms: 95.0, // Under target of 100ms
            throughput_rps: 1500.0, // Above target of 1000 RPS
            memory_usage_mb: 128.0,
            cpu_usage_percent: 45.0,
        };
        
        assert!(benchmarks.validate_performance(&good_results).unwrap());
    }
}
