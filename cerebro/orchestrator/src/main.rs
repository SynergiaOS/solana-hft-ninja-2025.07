//! 🚀 Cerebro Orchestrator - Enterprise AI Integration
//! 
//! Main orchestrator that ties all optimizations together

use anyhow::Result;
use clap::{Arg, Command};
use std::sync::Arc;
use tokio::signal;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error, debug};
use tracing_subscriber;

// Import our optimized modules
use crate::batch_processor::batch_aggregator::{BatchAggregator, BatchConsumer, BatchConfig};
use crate::prompt_engine::skeleton_templates::{PromptCompressor, PromptOptimizer};
use crate::model_router::model_switching::{ModelRouter, ModelSwitchingConfig, RequestContext, RequestPriority};
use crate::cache_engine::dragonfly_cache::{DragonflyCache, CacheConfig, CacheKey};
use crate::feature_engine::lazy_extraction::{LazyFeatureExtractor, FeatureConfig};

mod batch_processor;
mod prompt_engine;
mod model_router;
mod cache_engine;
mod feature_engine;

/// Cerebro configuration
#[derive(Debug, Clone)]
pub struct CerebroConfig {
    pub batch_config: BatchConfig,
    pub cache_config: CacheConfig,
    pub model_config: ModelSwitchingConfig,
    pub feature_config: FeatureConfig,
    pub webhook_port: u16,
    pub metrics_port: u16,
    pub log_level: String,
    pub enable_chaos_testing: bool,
}

impl Default for CerebroConfig {
    fn default() -> Self {
        Self {
            batch_config: BatchConfig::default(),
            cache_config: CacheConfig::default(),
            model_config: ModelSwitchingConfig::default(),
            feature_config: FeatureConfig::default(),
            webhook_port: 8081,
            metrics_port: 9091,
            log_level: "info".to_string(),
            enable_chaos_testing: false,
        }
    }
}

/// Main Cerebro orchestrator
pub struct CerebroOrchestrator {
    config: CerebroConfig,
    batch_aggregator: Arc<RwLock<BatchAggregator>>,
    batch_consumer: Arc<BatchConsumer>,
    prompt_optimizer: Arc<RwLock<PromptOptimizer>>,
    model_router: Arc<RwLock<ModelRouter>>,
    cache: Arc<RwLock<DragonflyCache>>,
    feature_extractor: Arc<RwLock<LazyFeatureExtractor>>,
}

impl CerebroOrchestrator {
    pub async fn new(config: CerebroConfig) -> Result<Self> {
        info!("🧠 Initializing Cerebro Orchestrator");

        // Initialize components
        let batch_aggregator = Arc::new(RwLock::new(
            BatchAggregator::new(config.batch_config.clone())?
        ));
        
        let batch_consumer = Arc::new(
            BatchConsumer::new(config.batch_config.clone())?
        );
        
        let prompt_optimizer = Arc::new(RwLock::new(
            PromptOptimizer::new()
        ));
        
        let model_router = Arc::new(RwLock::new(
            ModelRouter::new(config.model_config.clone())
        ));
        
        let cache = Arc::new(RwLock::new(
            DragonflyCache::new(config.cache_config.clone())?
        ));
        
        let feature_extractor = Arc::new(RwLock::new(
            LazyFeatureExtractor::new(config.feature_config.clone())
        ));

        Ok(Self {
            config,
            batch_aggregator,
            batch_consumer,
            prompt_optimizer,
            model_router,
            cache,
            feature_extractor,
        })
    }

    /// Start the orchestrator
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting Cerebro Orchestrator");

        // Start background services
        let batch_processor = self.start_batch_processor();
        let ai_inference_engine = self.start_ai_inference_engine();
        let cache_manager = self.start_cache_manager();
        let feature_processor = self.start_feature_processor();
        let metrics_server = self.start_metrics_server();
        let webhook_server = self.start_webhook_server();

        // Wait for shutdown signal
        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("Received shutdown signal");
            }
            result = batch_processor => {
                error!("Batch processor failed: {:?}", result);
            }
            result = ai_inference_engine => {
                error!("AI inference engine failed: {:?}", result);
            }
            result = cache_manager => {
                error!("Cache manager failed: {:?}", result);
            }
            result = feature_processor => {
                error!("Feature processor failed: {:?}", result);
            }
            result = metrics_server => {
                error!("Metrics server failed: {:?}", result);
            }
            result = webhook_server => {
                error!("Webhook server failed: {:?}", result);
            }
        }

        info!("🛑 Shutting down Cerebro Orchestrator");
        Ok(())
    }

    /// Start batch processing service
    fn start_batch_processor(&self) -> tokio::task::JoinHandle<Result<()>> {
        let batch_aggregator = self.batch_aggregator.clone();
        
        tokio::spawn(async move {
            let mut aggregator = batch_aggregator.write().await;
            aggregator.start_background_processor().await
        })
    }

    /// Start AI inference engine
    fn start_ai_inference_engine(&self) -> tokio::task::JoinHandle<Result<()>> {
        let batch_consumer = self.batch_consumer.clone();
        let prompt_optimizer = self.prompt_optimizer.clone();
        let model_router = self.model_router.clone();
        let cache = self.cache.clone();

        tokio::spawn(async move {
            let mut inference_interval = interval(Duration::from_secs(5));
            
            loop {
                inference_interval.tick().await;
                
                // Process next batch
                if let Ok(Some(batch)) = batch_consumer.consume_next_batch().await {
                    debug!("Processing batch: {}", batch.batch_id);
                    
                    // Check cache first
                    let cache_key = CacheKey::new(
                        "batch",
                        &batch.batch_id,
                        "analysis",
                        batch.time_range.0,
                    );
                    
                    let mut cache_guard = cache.write().await;
                    if let Ok(Some(cached_result)) = cache_guard.get(&cache_key).await {
                        info!("Cache hit for batch: {}", batch.batch_id);
                        continue;
                    }
                    
                    // Optimize prompt
                    let optimized_prompt = {
                        let mut optimizer = prompt_optimizer.write().await;
                        optimizer.optimize_prompt(&batch)?
                    };
                    
                    // Select optimal model
                    let context = RequestContext {
                        data_age_minutes: (std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs() - batch.time_range.1) / 60,
                        batch_size: batch.event_count,
                        priority: if batch.event_count > 50 {
                            RequestPriority::Realtime
                        } else {
                            RequestPriority::Standard
                        },
                        required_quality: 0.8,
                        max_cost: Some(0.01), // $0.01 per batch
                    };
                    
                    let model_tier = {
                        let mut router = model_router.write().await;
                        router.select_model(&context)?
                    };
                    
                    info!("Selected {:?} model for batch {}", model_tier, batch.batch_id);
                    
                    // TODO: Actual LLM inference would go here
                    // For now, simulate processing
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    
                    // Record usage
                    {
                        let mut router = model_router.write().await;
                        router.record_usage(model_tier, 1000, 100, true);
                    }
                }
            }
        })
    }

    /// Start cache management service
    fn start_cache_manager(&self) -> tokio::task::JoinHandle<Result<()>> {
        let cache = self.cache.clone();
        
        tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(300)); // 5 minutes
            
            loop {
                cleanup_interval.tick().await;
                
                let mut cache_guard = cache.write().await;
                let (cache_size, memory_usage) = cache_guard.get_cache_info().await?;
                
                info!("Cache status: {} entries, {:.2} MB", cache_size, memory_usage);
                
                // Cleanup if memory usage is too high
                if memory_usage > 100.0 { // 100 MB limit
                    warn!("Cache memory usage high, triggering cleanup");
                    // TODO: Implement intelligent cache eviction
                }
            }
        })
    }

    /// Start feature processing service
    fn start_feature_processor(&self) -> tokio::task::JoinHandle<Result<()>> {
        let feature_extractor = self.feature_extractor.clone();
        
        tokio::spawn(async move {
            let mut processing_interval = interval(Duration::from_millis(100));
            
            loop {
                processing_interval.tick().await;
                
                // TODO: Receive market ticks from HFT Ninja
                // For now, simulate with mock data
                let mock_tick = feature_engine::lazy_extraction::MarketTick {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    price: 100.0 + rand::random::<f64>() * 10.0,
                    volume: 1000.0 + rand::random::<f64>() * 500.0,
                    token_mint: "SOL".to_string(),
                    dex: "raydium".to_string(),
                };
                
                let mut extractor = feature_extractor.write().await;
                if let Err(e) = extractor.add_tick(mock_tick).await {
                    error!("Feature extraction failed: {}", e);
                }
            }
        })
    }

    /// Start metrics server
    fn start_metrics_server(&self) -> tokio::task::JoinHandle<Result<()>> {
        let port = self.config.metrics_port;
        let model_router = self.model_router.clone();
        let cache = self.cache.clone();
        
        tokio::spawn(async move {
            use warp::Filter;
            
            let metrics = warp::path("metrics")
                .and(warp::get())
                .and_then(move || {
                    let router = model_router.clone();
                    let cache = cache.clone();
                    
                    async move {
                        // Generate Prometheus metrics
                        let mut metrics_output = String::new();
                        
                        // Model router metrics
                        {
                            let router_guard = router.read().await;
                            let report = router_guard.get_cost_efficiency_report();
                            
                            metrics_output.push_str(&format!(
                                "cerebro_total_requests {}\n",
                                report.total_requests
                            ));
                            metrics_output.push_str(&format!(
                                "cerebro_total_cost {}\n",
                                report.total_cost
                            ));
                            metrics_output.push_str(&format!(
                                "cerebro_avg_latency_ms {}\n",
                                report.avg_latency_ms
                            ));
                        }
                        
                        // Cache metrics
                        {
                            let cache_guard = cache.read().await;
                            let cache_metrics = cache_guard.get_metrics();
                            
                            metrics_output.push_str(&format!(
                                "cerebro_cache_hits {}\n",
                                cache_metrics.hits
                            ));
                            metrics_output.push_str(&format!(
                                "cerebro_cache_misses {}\n",
                                cache_metrics.misses
                            ));
                            metrics_output.push_str(&format!(
                                "cerebro_cache_hit_rate {}\n",
                                cache_metrics.hit_rate()
                            ));
                        }
                        
                        Ok::<_, warp::Rejection>(warp::reply::with_header(
                            metrics_output,
                            "content-type",
                            "text/plain; version=0.0.4"
                        ))
                    }
                });
            
            info!("📊 Starting metrics server on port {}", port);
            warp::serve(metrics)
                .run(([0, 0, 0, 0], port))
                .await;
            
            Ok(())
        })
    }

    /// Start webhook server for HFT Ninja integration
    fn start_webhook_server(&self) -> tokio::task::JoinHandle<Result<()>> {
        let port = self.config.webhook_port;
        let batch_aggregator = self.batch_aggregator.clone();
        
        tokio::spawn(async move {
            use warp::Filter;
            use serde_json::Value;
            
            let webhook = warp::path("webhook")
                .and(warp::post())
                .and(warp::body::json())
                .and_then(move |event: Value| {
                    let aggregator = batch_aggregator.clone();
                    
                    async move {
                        // Parse trading event from HFT Ninja
                        if let Ok(trading_event) = serde_json::from_value::<batch_processor::batch_aggregator::TradingEvent>(event) {
                            let mut aggregator_guard = aggregator.write().await;
                            if let Err(e) = aggregator_guard.add_event(trading_event).await {
                                error!("Failed to add trading event: {}", e);
                                return Ok::<_, warp::Rejection>(warp::reply::with_status(
                                    "Error processing event",
                                    warp::http::StatusCode::INTERNAL_SERVER_ERROR
                                ));
                            }
                        }
                        
                        Ok(warp::reply::with_status(
                            "Event processed",
                            warp::http::StatusCode::OK
                        ))
                    }
                });
            
            info!("🔗 Starting webhook server on port {}", port);
            warp::serve(webhook)
                .run(([0, 0, 0, 0], port))
                .await;
            
            Ok(())
        })
    }

    /// Get comprehensive status report
    pub async fn get_status_report(&self) -> Result<CerebroStatusReport> {
        let model_report = {
            let router = self.model_router.read().await;
            router.get_cost_efficiency_report()
        };
        
        let cache_metrics = {
            let cache = self.cache.read().await;
            cache.get_metrics().clone()
        };
        
        Ok(CerebroStatusReport {
            uptime_seconds: 0, // TODO: Track actual uptime
            total_batches_processed: model_report.total_requests,
            total_cost_usd: model_report.total_cost,
            cache_hit_rate: cache_metrics.hit_rate(),
            avg_processing_latency_ms: model_report.avg_latency_ms,
            daily_budget_used: model_report.daily_budget_used,
            system_healthy: true, // TODO: Implement health checks
        })
    }
}

#[derive(Debug)]
pub struct CerebroStatusReport {
    pub uptime_seconds: u64,
    pub total_batches_processed: u64,
    pub total_cost_usd: f64,
    pub cache_hit_rate: f64,
    pub avg_processing_latency_ms: f64,
    pub daily_budget_used: f64,
    pub system_healthy: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let matches = Command::new("cerebro-orchestrator")
        .version("1.0.0")
        .about("🧠 Cerebro AI Orchestrator for Solana HFT Ninja")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path")
        )
        .arg(
            Arg::new("log-level")
                .short('l')
                .long("log-level")
                .value_name("LEVEL")
                .help("Log level (trace, debug, info, warn, error)")
                .default_value("info")
        )
        .arg(
            Arg::new("chaos-testing")
                .long("enable-chaos-testing")
                .help("Enable chaos testing mode")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    // Create configuration
    let mut config = CerebroConfig::default();
    config.log_level = matches.get_one::<String>("log-level").unwrap().clone();
    config.enable_chaos_testing = matches.get_flag("chaos-testing");

    // Initialize and start orchestrator
    let orchestrator = CerebroOrchestrator::new(config).await?;
    orchestrator.start().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_initialization() {
        let config = CerebroConfig::default();
        let orchestrator = CerebroOrchestrator::new(config).await;
        assert!(orchestrator.is_ok());
    }

    #[tokio::test]
    async fn test_status_report() {
        let config = CerebroConfig::default();
        let orchestrator = CerebroOrchestrator::new(config).await.unwrap();
        let report = orchestrator.get_status_report().await.unwrap();
        
        assert!(report.cache_hit_rate >= 0.0);
        assert!(report.cache_hit_rate <= 1.0);
        assert!(report.total_cost_usd >= 0.0);
    }
}
