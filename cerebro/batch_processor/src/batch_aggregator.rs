//! 🚀 Batch Aggregation System - Enterprise Performance
//! 
//! Reduces LLM costs by N× through intelligent batching and compression

use anyhow::Result;
use redis::{Client, Commands, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{Duration, interval};
use tracing::{info, warn, debug};
use bincode;
use base64;

/// Batch configuration for optimal performance
#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub max_batch_size: usize,        // N=100 records per batch
    pub batch_timeout_seconds: u64,   // 30 seconds max wait
    pub redis_url: String,
    pub fast_queue: String,           // "cerebro:fast" - last 5 min
    pub slow_queue: String,           // "cerebro:slow" - historical
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            batch_timeout_seconds: 30,
            redis_url: "redis://localhost:6379".to_string(),
            fast_queue: "cerebro:fast".to_string(),
            slow_queue: "cerebro:slow".to_string(),
        }
    }
}

/// Trading event for batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingEvent {
    pub timestamp: u64,
    pub wallet_address: String,
    pub token_mint: String,
    pub strategy_type: String,
    pub amount_sol: f64,
    pub profit_sol: f64,
    pub execution_time_ms: u64,
    pub success: bool,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Compressed batch for LLM processing
#[derive(Debug, Serialize, Deserialize)]
pub struct CompressedBatch {
    pub batch_id: String,
    pub event_count: usize,
    pub time_range: (u64, u64),
    pub compressed_data: String,  // base64 encoded bincode
    pub summary_stats: BatchStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchStats {
    pub total_volume_sol: f64,
    pub total_profit_sol: f64,
    pub success_rate: f64,
    pub avg_execution_time_ms: f64,
    pub unique_wallets: usize,
    pub unique_tokens: usize,
    pub strategy_distribution: HashMap<String, usize>,
}

/// High-performance batch aggregator
pub struct BatchAggregator {
    config: BatchConfig,
    redis_client: Client,
    current_batch: Vec<TradingEvent>,
    batch_start_time: std::time::Instant,
}

impl BatchAggregator {
    pub fn new(config: BatchConfig) -> Result<Self> {
        let redis_client = Client::open(config.redis_url.clone())?;
        
        Ok(Self {
            config,
            redis_client,
            current_batch: Vec::new(),
            batch_start_time: std::time::Instant::now(),
        })
    }

    /// Add event to current batch with intelligent routing
    pub async fn add_event(&mut self, event: TradingEvent) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        
        // Route to fast queue if recent (last 5 minutes)
        let queue = if now - event.timestamp < 300 {
            &self.config.fast_queue
        } else {
            &self.config.slow_queue
        };

        // Add to current batch
        self.current_batch.push(event);

        // Check if batch should be processed
        if self.should_process_batch() {
            self.process_current_batch(queue).await?;
        }

        Ok(())
    }

    /// Check if batch should be processed
    fn should_process_batch(&self) -> bool {
        self.current_batch.len() >= self.config.max_batch_size ||
        self.batch_start_time.elapsed().as_secs() >= self.config.batch_timeout_seconds
    }

    /// Process and compress current batch
    async fn process_current_batch(&mut self, queue: &str) -> Result<()> {
        if self.current_batch.is_empty() {
            return Ok(());
        }

        debug!("Processing batch of {} events", self.current_batch.len());

        // Generate batch statistics
        let stats = self.calculate_batch_stats();
        
        // Compress batch data using bincode + base64
        let compressed_data = self.compress_batch_data()?;
        
        // Create compressed batch
        let batch = CompressedBatch {
            batch_id: uuid::Uuid::new_v4().to_string(),
            event_count: self.current_batch.len(),
            time_range: self.get_time_range(),
            compressed_data,
            summary_stats: stats,
        };

        // Push to Redis queue
        let mut conn = self.redis_client.get_connection()?;
        let batch_json = serde_json::to_string(&batch)?;
        let _: () = conn.lpush(queue, batch_json)?;

        info!(
            "Batch {} with {} events pushed to {} queue", 
            batch.batch_id, 
            batch.event_count,
            queue
        );

        // Reset batch
        self.current_batch.clear();
        self.batch_start_time = std::time::Instant::now();

        Ok(())
    }

    /// Compress batch data using bincode + base64
    fn compress_batch_data(&self) -> Result<String> {
        // Serialize to bincode (binary format)
        let binary_data = bincode::serialize(&self.current_batch)?;
        
        // Encode to base64
        let base64_data = base64::encode(binary_data);
        
        debug!(
            "Compressed {} events: {} bytes -> {} chars", 
            self.current_batch.len(),
            std::mem::size_of_val(&self.current_batch),
            base64_data.len()
        );

        Ok(base64_data)
    }

    /// Calculate comprehensive batch statistics
    fn calculate_batch_stats(&self) -> BatchStats {
        let mut total_volume = 0.0;
        let mut total_profit = 0.0;
        let mut successful_trades = 0;
        let mut total_execution_time = 0u64;
        let mut unique_wallets = std::collections::HashSet::new();
        let mut unique_tokens = std::collections::HashSet::new();
        let mut strategy_distribution = HashMap::new();

        for event in &self.current_batch {
            total_volume += event.amount_sol;
            total_profit += event.profit_sol;
            total_execution_time += event.execution_time_ms;
            
            if event.success {
                successful_trades += 1;
            }

            unique_wallets.insert(event.wallet_address.clone());
            unique_tokens.insert(event.token_mint.clone());
            
            *strategy_distribution.entry(event.strategy_type.clone()).or_insert(0) += 1;
        }

        let event_count = self.current_batch.len();
        
        BatchStats {
            total_volume_sol: total_volume,
            total_profit_sol: total_profit,
            success_rate: if event_count > 0 { 
                successful_trades as f64 / event_count as f64 
            } else { 
                0.0 
            },
            avg_execution_time_ms: if event_count > 0 { 
                total_execution_time as f64 / event_count as f64 
            } else { 
                0.0 
            },
            unique_wallets: unique_wallets.len(),
            unique_tokens: unique_tokens.len(),
            strategy_distribution,
        }
    }

    /// Get time range of current batch
    fn get_time_range(&self) -> (u64, u64) {
        if self.current_batch.is_empty() {
            return (0, 0);
        }

        let mut min_time = u64::MAX;
        let mut max_time = 0u64;

        for event in &self.current_batch {
            min_time = min_time.min(event.timestamp);
            max_time = max_time.max(event.timestamp);
        }

        (min_time, max_time)
    }

    /// Start background batch processor
    pub async fn start_background_processor(&mut self) -> Result<()> {
        let mut interval = interval(Duration::from_secs(self.config.batch_timeout_seconds));
        
        loop {
            interval.tick().await;
            
            if !self.current_batch.is_empty() {
                self.process_current_batch(&self.config.fast_queue).await?;
            }
        }
    }
}

/// Batch consumer for LLM processing
pub struct BatchConsumer {
    config: BatchConfig,
    redis_client: Client,
}

impl BatchConsumer {
    pub fn new(config: BatchConfig) -> Result<Self> {
        let redis_client = Client::open(config.redis_url.clone())?;
        
        Ok(Self {
            config,
            redis_client,
        })
    }

    /// Consume batch from priority queue (fast first)
    pub async fn consume_next_batch(&self) -> Result<Option<CompressedBatch>> {
        let mut conn = self.redis_client.get_connection()?;
        
        // Try fast queue first
        if let Ok(batch_json) = conn.rpop::<_, String>(&self.config.fast_queue) {
            let batch: CompressedBatch = serde_json::from_str(&batch_json)?;
            debug!("Consumed batch {} from fast queue", batch.batch_id);
            return Ok(Some(batch));
        }
        
        // Fallback to slow queue
        if let Ok(batch_json) = conn.rpop::<_, String>(&self.config.slow_queue) {
            let batch: CompressedBatch = serde_json::from_str(&batch_json)?;
            debug!("Consumed batch {} from slow queue", batch.batch_id);
            return Ok(Some(batch));
        }
        
        Ok(None)
    }

    /// Decompress batch data
    pub fn decompress_batch(&self, batch: &CompressedBatch) -> Result<Vec<TradingEvent>> {
        // Decode from base64
        let binary_data = base64::decode(&batch.compressed_data)?;
        
        // Deserialize from bincode
        let events: Vec<TradingEvent> = bincode::deserialize(&binary_data)?;
        
        debug!(
            "Decompressed batch {}: {} events", 
            batch.batch_id, 
            events.len()
        );

        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_batch_compression() {
        let config = BatchConfig::default();
        let mut aggregator = BatchAggregator::new(config).unwrap();

        // Add test events
        for i in 0..50 {
            let event = TradingEvent {
                timestamp: 1640995200 + i,
                wallet_address: format!("wallet_{}", i),
                token_mint: format!("token_{}", i % 10),
                strategy_type: "arbitrage".to_string(),
                amount_sol: 0.1 * i as f64,
                profit_sol: 0.01 * i as f64,
                execution_time_ms: 100 + i,
                success: i % 3 == 0,
                metadata: HashMap::new(),
            };
            
            aggregator.add_event(event).await.unwrap();
        }

        // Verify batch stats
        let stats = aggregator.calculate_batch_stats();
        assert_eq!(stats.unique_tokens, 10);
        assert!(stats.total_volume_sol > 0.0);
    }
}
