//! 🗄️ DragonflyDB Cache Strategy - Ultra-Fast LLM Response Caching
//! 
//! Intelligent caching to avoid expensive LLM recomputation

use anyhow::Result;
use redis::{Client, Commands, Connection};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tokio::time::{Duration, Instant};
use tracing::{info, warn, debug};
use crate::skeleton_templates::AnalysisResponse;

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub redis_url: String,
    pub default_ttl_seconds: u64,
    pub max_cache_size: usize,
    pub compression_enabled: bool,
    pub metrics_enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            default_ttl_seconds: 300, // 5 minutes
            max_cache_size: 10000,
            compression_enabled: true,
            metrics_enabled: true,
        }
    }
}

/// Cache key components for intelligent invalidation
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    pub wallet_address: String,
    pub token_mint: String,
    pub strategy_hash: String,
    pub time_bucket: u64, // Rounded to 5-minute buckets
}

impl CacheKey {
    pub fn new(wallet: &str, token: &str, strategy: &str, timestamp: u64) -> Self {
        // Round timestamp to 5-minute buckets for better cache hits
        let time_bucket = (timestamp / 300) * 300;
        
        Self {
            wallet_address: wallet.to_string(),
            token_mint: token.to_string(),
            strategy_hash: Self::hash_strategy(strategy),
            time_bucket,
        }
    }

    /// Create hash of strategy parameters for cache key
    fn hash_strategy(strategy: &str) -> String {
        let mut hasher = DefaultHasher::new();
        strategy.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Generate Redis key
    pub fn to_redis_key(&self) -> String {
        format!(
            "cerebro:cache:{}:{}:{}:{}",
            self.wallet_address,
            self.token_mint,
            self.strategy_hash,
            self.time_bucket
        )
    }
}

/// Cached analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAnalysis {
    pub result: AnalysisResponse,
    pub timestamp: u64,
    pub hit_count: u32,
    pub confidence_decay: f64, // Confidence decreases over time
}

impl CachedAnalysis {
    pub fn new(result: AnalysisResponse) -> Self {
        Self {
            result,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            hit_count: 0,
            confidence_decay: 1.0,
        }
    }

    /// Update confidence based on age
    pub fn update_confidence(&mut self) {
        let age_minutes = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() - self.timestamp) / 60;

        // Confidence decays exponentially: 100% -> 90% -> 81% -> ...
        self.confidence_decay = 0.9_f64.powf(age_minutes as f64);
        self.hit_count += 1;
    }

    /// Check if cache entry is still valid
    pub fn is_valid(&self, max_age_seconds: u64) -> bool {
        let age = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() - self.timestamp;
        
        age <= max_age_seconds && self.confidence_decay > 0.5
    }
}

/// Cache performance metrics
#[derive(Debug, Default)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_requests: u64,
    pub avg_lookup_time_ms: f64,
    pub cache_size: usize,
    pub memory_usage_mb: f64,
}

impl CacheMetrics {
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.hits as f64 / self.total_requests as f64
        }
    }

    pub fn miss_rate(&self) -> f64 {
        1.0 - self.hit_rate()
    }
}

/// High-performance DragonflyDB cache
pub struct DragonflyCache {
    config: CacheConfig,
    client: Client,
    metrics: CacheMetrics,
    last_cleanup: Instant,
}

impl DragonflyCache {
    pub fn new(config: CacheConfig) -> Result<Self> {
        let client = Client::open(config.redis_url.clone())?;
        
        Ok(Self {
            config,
            client,
            metrics: CacheMetrics::default(),
            last_cleanup: Instant::now(),
        })
    }

    /// Get cached analysis result
    pub async fn get(&mut self, key: &CacheKey) -> Result<Option<CachedAnalysis>> {
        let start_time = Instant::now();
        let redis_key = key.to_redis_key();
        
        let mut conn = self.client.get_connection()?;
        
        match conn.get::<_, String>(&redis_key) {
            Ok(cached_data) => {
                let mut cached: CachedAnalysis = serde_json::from_str(&cached_data)?;
                
                // Check if still valid
                if cached.is_valid(self.config.default_ttl_seconds) {
                    cached.update_confidence();
                    
                    // Update cache with new hit count and confidence
                    let updated_data = serde_json::to_string(&cached)?;
                    let _: () = conn.set_ex(&redis_key, updated_data, self.config.default_ttl_seconds)?;
                    
                    self.record_hit(start_time);
                    debug!("Cache hit for key: {} (confidence: {:.2})", redis_key, cached.confidence_decay);
                    
                    Ok(Some(cached))
                } else {
                    // Expired, remove from cache
                    let _: () = conn.del(&redis_key)?;
                    self.record_miss(start_time);
                    debug!("Cache expired for key: {}", redis_key);
                    
                    Ok(None)
                }
            },
            Err(_) => {
                self.record_miss(start_time);
                debug!("Cache miss for key: {}", redis_key);
                Ok(None)
            }
        }
    }

    /// Store analysis result in cache
    pub async fn set(&mut self, key: &CacheKey, result: AnalysisResponse) -> Result<()> {
        let redis_key = key.to_redis_key();
        let cached = CachedAnalysis::new(result);
        let cached_data = serde_json::to_string(&cached)?;
        
        let mut conn = self.client.get_connection()?;
        
        // Store with TTL
        let _: () = conn.set_ex(&redis_key, cached_data, self.config.default_ttl_seconds)?;
        
        debug!("Cached analysis for key: {}", redis_key);
        
        // Periodic cleanup
        self.cleanup_if_needed().await?;
        
        Ok(())
    }

    /// Check if analysis should be cached (avoid caching low-confidence results)
    pub fn should_cache(&self, result: &AnalysisResponse) -> bool {
        result.confidence_score > 0.7 && 
        result.execution_priority >= 5
    }

    /// Invalidate cache entries for specific wallet/token
    pub async fn invalidate_pattern(&mut self, pattern: &str) -> Result<u64> {
        let mut conn = self.client.get_connection()?;
        
        // Get all keys matching pattern
        let keys: Vec<String> = conn.keys(format!("cerebro:cache:{}*", pattern))?;
        
        if !keys.is_empty() {
            let deleted: u64 = conn.del(&keys)?;
            info!("Invalidated {} cache entries for pattern: {}", deleted, pattern);
            self.metrics.evictions += deleted;
            return Ok(deleted);
        }
        
        Ok(0)
    }

    /// Warm cache with predicted queries
    pub async fn warm_cache(&mut self, predictions: Vec<(CacheKey, AnalysisResponse)>) -> Result<()> {
        let mut conn = self.client.get_connection()?;
        
        for (key, result) in predictions {
            if self.should_cache(&result) {
                let redis_key = key.to_redis_key();
                let cached = CachedAnalysis::new(result);
                let cached_data = serde_json::to_string(&cached)?;
                
                let _: () = conn.set_ex(&redis_key, cached_data, self.config.default_ttl_seconds)?;
            }
        }
        
        info!("Cache warmed with {} predictions", predictions.len());
        Ok(())
    }

    /// Get cache statistics
    pub fn get_metrics(&self) -> &CacheMetrics {
        &self.metrics
    }

    /// Record cache hit
    fn record_hit(&mut self, start_time: Instant) {
        self.metrics.hits += 1;
        self.metrics.total_requests += 1;
        self.update_avg_lookup_time(start_time);
    }

    /// Record cache miss
    fn record_miss(&mut self, start_time: Instant) {
        self.metrics.misses += 1;
        self.metrics.total_requests += 1;
        self.update_avg_lookup_time(start_time);
    }

    /// Update average lookup time
    fn update_avg_lookup_time(&mut self, start_time: Instant) {
        let lookup_time = start_time.elapsed().as_millis() as f64;
        let alpha = 0.1; // Exponential moving average
        self.metrics.avg_lookup_time_ms = 
            self.metrics.avg_lookup_time_ms * (1.0 - alpha) + lookup_time * alpha;
    }

    /// Periodic cleanup of expired entries
    async fn cleanup_if_needed(&mut self) -> Result<()> {
        if self.last_cleanup.elapsed() >= Duration::from_secs(300) { // Every 5 minutes
            self.cleanup_expired_entries().await?;
            self.last_cleanup = Instant::now();
        }
        Ok(())
    }

    /// Clean up expired cache entries
    async fn cleanup_expired_entries(&mut self) -> Result<()> {
        let mut conn = self.client.get_connection()?;
        
        // Get all cache keys
        let keys: Vec<String> = conn.keys("cerebro:cache:*")?;
        let mut expired_keys = Vec::new();
        
        for key in keys {
            if let Ok(cached_data) = conn.get::<_, String>(&key) {
                if let Ok(cached) = serde_json::from_str::<CachedAnalysis>(&cached_data) {
                    if !cached.is_valid(self.config.default_ttl_seconds) {
                        expired_keys.push(key);
                    }
                }
            }
        }
        
        if !expired_keys.is_empty() {
            let deleted: u64 = conn.del(&expired_keys)?;
            info!("Cleaned up {} expired cache entries", deleted);
            self.metrics.evictions += deleted;
        }
        
        Ok(())
    }

    /// Get cache size and memory usage
    pub async fn get_cache_info(&mut self) -> Result<(usize, f64)> {
        let mut conn = self.client.get_connection()?;
        
        let keys: Vec<String> = conn.keys("cerebro:cache:*")?;
        let cache_size = keys.len();
        
        // Estimate memory usage (rough calculation)
        let mut total_memory = 0;
        for key in keys.iter().take(100) { // Sample first 100 keys
            if let Ok(data) = conn.get::<_, String>(key) {
                total_memory += data.len();
            }
        }
        
        let estimated_memory_mb = if keys.len() > 0 {
            (total_memory * keys.len() / 100.min(keys.len())) as f64 / 1024.0 / 1024.0
        } else {
            0.0
        };
        
        self.metrics.cache_size = cache_size;
        self.metrics.memory_usage_mb = estimated_memory_mb;
        
        Ok((cache_size, estimated_memory_mb))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skeleton_templates::{AnalysisResponse, RiskLevel};

    #[test]
    fn test_cache_key_generation() {
        let key1 = CacheKey::new("wallet1", "token1", "strategy1", 1640995200);
        let key2 = CacheKey::new("wallet1", "token1", "strategy1", 1640995250); // 50s later
        
        // Should have same time bucket (5-minute buckets)
        assert_eq!(key1.time_bucket, key2.time_bucket);
        assert_eq!(key1.to_redis_key(), key2.to_redis_key());
    }

    #[test]
    fn test_cached_analysis_confidence_decay() {
        let response = AnalysisResponse {
            strategy_recommendation: "test".to_string(),
            confidence_score: 0.9,
            risk_assessment: RiskLevel::Low,
            execution_priority: 8,
            key_insights: vec![],
            next_actions: vec![],
        };
        
        let mut cached = CachedAnalysis::new(response);
        assert_eq!(cached.confidence_decay, 1.0);
        
        // Simulate aging
        cached.timestamp -= 300; // 5 minutes ago
        cached.update_confidence();
        
        assert!(cached.confidence_decay < 1.0);
        assert!(cached.confidence_decay > 0.8); // Should still be high
    }
}
