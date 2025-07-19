//! ⚡ Model Switching Strategy - Cost-Optimized AI Pipeline
//! 
//! Hot/Warm/Cold model routing for optimal cost vs quality balance

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use tracing::{info, warn, debug};

/// Model tier for different use cases
#[derive(Debug, Clone, PartialEq)]
pub enum ModelTier {
    Hot,    // GPT-4o-mini - fast, cheap, 30s summaries
    Warm,   // GPT-4o - better quality, hourly aggregations  
    Cold,   // Llama-3-70B - fine-tuned, nightly retraining
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub provider: String,
    pub cost_per_1k_tokens: f64,
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout_seconds: u64,
    pub rate_limit_rpm: u32,
}

/// Model switching configuration
#[derive(Debug, Clone)]
pub struct ModelSwitchingConfig {
    pub hot_model: ModelConfig,
    pub warm_model: ModelConfig,
    pub cold_model: ModelConfig,
    pub hot_threshold_minutes: u64,    // Use hot model for data <5 min old
    pub warm_threshold_hours: u64,     // Use warm model for data <24h old
    pub cost_budget_daily: f64,        // Daily cost budget in USD
    pub quality_threshold: f64,        // Min quality score (0-1)
}

impl Default for ModelSwitchingConfig {
    fn default() -> Self {
        Self {
            hot_model: ModelConfig {
                name: "gpt-4o-mini".to_string(),
                provider: "openai".to_string(),
                cost_per_1k_tokens: 0.00015, // $0.15 per 1M tokens
                max_tokens: 4096,
                temperature: 0.3,
                timeout_seconds: 10,
                rate_limit_rpm: 1000,
            },
            warm_model: ModelConfig {
                name: "gpt-4o".to_string(),
                provider: "openai".to_string(),
                cost_per_1k_tokens: 0.005, // $5 per 1M tokens
                max_tokens: 8192,
                temperature: 0.1,
                timeout_seconds: 30,
                rate_limit_rpm: 100,
            },
            cold_model: ModelConfig {
                name: "llama-3-70b-instruct".to_string(),
                provider: "local".to_string(),
                cost_per_1k_tokens: 0.0, // Free on own GPU
                max_tokens: 16384,
                temperature: 0.0,
                timeout_seconds: 60,
                rate_limit_rpm: 10,
            },
            hot_threshold_minutes: 5,
            warm_threshold_hours: 24,
            cost_budget_daily: 10.0, // $10/day budget
            quality_threshold: 0.7,
        }
    }
}

/// Request context for model selection
#[derive(Debug)]
pub struct RequestContext {
    pub data_age_minutes: u64,
    pub batch_size: usize,
    pub priority: RequestPriority,
    pub required_quality: f64,
    pub max_cost: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestPriority {
    Realtime,   // <60s response needed
    Standard,   // <5min response acceptable
    Batch,      // Can wait hours
}

/// Model usage statistics
#[derive(Debug, Default)]
pub struct ModelUsageStats {
    pub requests_count: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub avg_latency_ms: f64,
    pub success_rate: f64,
    pub last_used: Option<Instant>,
}

/// Intelligent model router
pub struct ModelRouter {
    config: ModelSwitchingConfig,
    usage_stats: HashMap<ModelTier, ModelUsageStats>,
    daily_cost: f64,
    last_reset: Instant,
}

impl ModelRouter {
    pub fn new(config: ModelSwitchingConfig) -> Self {
        Self {
            config,
            usage_stats: HashMap::new(),
            daily_cost: 0.0,
            last_reset: Instant::now(),
        }
    }

    /// Select optimal model based on context
    pub fn select_model(&mut self, context: &RequestContext) -> Result<ModelTier> {
        // Reset daily costs if needed
        self.reset_daily_costs_if_needed();

        // Check budget constraints
        if self.daily_cost >= self.config.cost_budget_daily {
            warn!("Daily budget exceeded, forcing cold model");
            return Ok(ModelTier::Cold);
        }

        // Priority-based selection
        let model_tier = match context.priority {
            RequestPriority::Realtime => {
                if context.data_age_minutes <= self.config.hot_threshold_minutes {
                    ModelTier::Hot
                } else {
                    ModelTier::Warm
                }
            },
            RequestPriority::Standard => {
                if context.data_age_minutes <= self.config.hot_threshold_minutes {
                    ModelTier::Hot
                } else if context.data_age_minutes <= self.config.warm_threshold_hours * 60 {
                    ModelTier::Warm
                } else {
                    ModelTier::Cold
                }
            },
            RequestPriority::Batch => {
                // Always use cold model for batch processing
                ModelTier::Cold
            }
        };

        // Quality override - use higher tier if quality requirement is high
        let final_tier = if context.required_quality > self.config.quality_threshold {
            match model_tier {
                ModelTier::Hot => ModelTier::Warm,
                ModelTier::Warm => ModelTier::Warm,
                ModelTier::Cold => ModelTier::Warm,
            }
        } else {
            model_tier
        };

        // Cost override - check if we can afford the selected model
        if let Some(max_cost) = context.max_cost {
            let estimated_cost = self.estimate_request_cost(&final_tier, context.batch_size);
            if estimated_cost > max_cost {
                return Ok(self.find_cheapest_model_within_budget(max_cost, context.batch_size));
            }
        }

        debug!(
            "Selected {:?} model for request (age: {}min, priority: {:?})", 
            final_tier, 
            context.data_age_minutes, 
            context.priority
        );

        Ok(final_tier)
    }

    /// Get model configuration for tier
    pub fn get_model_config(&self, tier: &ModelTier) -> &ModelConfig {
        match tier {
            ModelTier::Hot => &self.config.hot_model,
            ModelTier::Warm => &self.config.warm_model,
            ModelTier::Cold => &self.config.cold_model,
        }
    }

    /// Record model usage for cost tracking
    pub fn record_usage(
        &mut self, 
        tier: ModelTier, 
        tokens_used: u64, 
        latency_ms: u64, 
        success: bool
    ) {
        let stats = self.usage_stats.entry(tier.clone()).or_default();
        let config = self.get_model_config(&tier);
        
        stats.requests_count += 1;
        stats.total_tokens += tokens_used;
        stats.last_used = Some(Instant::now());
        
        // Calculate cost
        let cost = (tokens_used as f64 / 1000.0) * config.cost_per_1k_tokens;
        stats.total_cost += cost;
        self.daily_cost += cost;
        
        // Update latency (exponential moving average)
        let alpha = 0.1;
        stats.avg_latency_ms = stats.avg_latency_ms * (1.0 - alpha) + latency_ms as f64 * alpha;
        
        // Update success rate
        let success_value = if success { 1.0 } else { 0.0 };
        stats.success_rate = stats.success_rate * (1.0 - alpha) + success_value * alpha;

        info!(
            "Model usage recorded: {:?} - {} tokens, ${:.4} cost, {}ms latency", 
            tier, tokens_used, cost, latency_ms
        );
    }

    /// Estimate cost for a request
    fn estimate_request_cost(&self, tier: &ModelTier, batch_size: usize) -> f64 {
        let config = self.get_model_config(tier);
        let estimated_tokens = batch_size * 50; // ~50 tokens per event
        (estimated_tokens as f64 / 1000.0) * config.cost_per_1k_tokens
    }

    /// Find cheapest model within budget
    fn find_cheapest_model_within_budget(&self, max_cost: f64, batch_size: usize) -> ModelTier {
        let tiers = [ModelTier::Cold, ModelTier::Hot, ModelTier::Warm];
        
        for tier in &tiers {
            let cost = self.estimate_request_cost(tier, batch_size);
            if cost <= max_cost {
                return tier.clone();
            }
        }
        
        // Fallback to cheapest (cold)
        ModelTier::Cold
    }

    /// Reset daily costs at midnight
    fn reset_daily_costs_if_needed(&mut self) {
        if self.last_reset.elapsed() >= Duration::from_secs(24 * 60 * 60) {
            self.daily_cost = 0.0;
            self.last_reset = Instant::now();
            info!("Daily cost budget reset");
        }
    }

    /// Get comprehensive usage statistics
    pub fn get_usage_stats(&self) -> &HashMap<ModelTier, ModelUsageStats> {
        &self.usage_stats
    }

    /// Get current daily cost
    pub fn get_daily_cost(&self) -> f64 {
        self.daily_cost
    }

    /// Get cost efficiency report
    pub fn get_cost_efficiency_report(&self) -> CostEfficiencyReport {
        let mut total_requests = 0;
        let mut total_cost = 0.0;
        let mut total_tokens = 0;
        let mut weighted_latency = 0.0;
        let mut weighted_success_rate = 0.0;

        for (tier, stats) in &self.usage_stats {
            total_requests += stats.requests_count;
            total_cost += stats.total_cost;
            total_tokens += stats.total_tokens;
            
            let weight = stats.requests_count as f64 / total_requests as f64;
            weighted_latency += stats.avg_latency_ms * weight;
            weighted_success_rate += stats.success_rate * weight;
        }

        CostEfficiencyReport {
            total_requests,
            total_cost,
            total_tokens,
            cost_per_request: if total_requests > 0 { 
                total_cost / total_requests as f64 
            } else { 
                0.0 
            },
            tokens_per_dollar: if total_cost > 0.0 { 
                total_tokens as f64 / total_cost 
            } else { 
                0.0 
            },
            avg_latency_ms: weighted_latency,
            overall_success_rate: weighted_success_rate,
            daily_budget_used: self.daily_cost / self.config.cost_budget_daily,
        }
    }
}

#[derive(Debug)]
pub struct CostEfficiencyReport {
    pub total_requests: u64,
    pub total_cost: f64,
    pub total_tokens: u64,
    pub cost_per_request: f64,
    pub tokens_per_dollar: f64,
    pub avg_latency_ms: f64,
    pub overall_success_rate: f64,
    pub daily_budget_used: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_selection_realtime() {
        let config = ModelSwitchingConfig::default();
        let mut router = ModelRouter::new(config);

        let context = RequestContext {
            data_age_minutes: 2,
            batch_size: 10,
            priority: RequestPriority::Realtime,
            required_quality: 0.5,
            max_cost: None,
        };

        let model = router.select_model(&context).unwrap();
        assert_eq!(model, ModelTier::Hot);
    }

    #[test]
    fn test_model_selection_high_quality() {
        let config = ModelSwitchingConfig::default();
        let mut router = ModelRouter::new(config);

        let context = RequestContext {
            data_age_minutes: 2,
            batch_size: 10,
            priority: RequestPriority::Standard,
            required_quality: 0.9, // High quality requirement
            max_cost: None,
        };

        let model = router.select_model(&context).unwrap();
        assert_eq!(model, ModelTier::Warm); // Should upgrade to warm for quality
    }

    #[test]
    fn test_cost_tracking() {
        let config = ModelSwitchingConfig::default();
        let mut router = ModelRouter::new(config);

        router.record_usage(ModelTier::Hot, 1000, 50, true);
        router.record_usage(ModelTier::Warm, 2000, 100, true);

        let report = router.get_cost_efficiency_report();
        assert!(report.total_cost > 0.0);
        assert_eq!(report.total_requests, 2);
        assert_eq!(report.total_tokens, 3000);
    }
}
