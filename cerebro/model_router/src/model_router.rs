//! ⚡ Model Switching Router - Cost-Optimized AI Selection
//! 
//! Routes requests to Hot/Warm/Cold model tiers based on data age and quality requirements

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

/// Model tier for cost optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelTier {
    Hot,   // GPT-4o-mini - $0.15/1M tokens - <5min data
    Warm,  // GPT-4o - $5/1M tokens - <24h data
    Cold,  // Llama-3-70B - $0 - historical data
}

/// Model configuration
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub daily_budget_usd: f64,
    pub hot_threshold_minutes: u64,
    pub warm_threshold_hours: u64,
    pub quality_threshold: f64,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            daily_budget_usd: 30.0,
            hot_threshold_minutes: 5,
            warm_threshold_hours: 24,
            quality_threshold: 0.8,
        }
    }
}

/// Request context for model selection
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub data_age_seconds: u64,
    pub quality_requirement: f64,
    pub batch_size: usize,
    pub priority: RequestPriority,
}

/// Request priority levels
#[derive(Debug, Clone)]
pub enum RequestPriority {
    Critical,  // Always use best model
    High,      // Prefer quality over cost
    Normal,    // Balance cost and quality
    Low,       // Minimize cost
}

/// Model router for intelligent selection
pub struct ModelRouter {
    config: ModelConfig,
    daily_cost: f64,
    request_count: u64,
}

impl ModelRouter {
    /// Create new model router
    pub fn new(config: ModelConfig) -> Self {
        info!("🚀 Initializing ModelRouter with ${} daily budget", config.daily_budget_usd);
        
        Self {
            config,
            daily_cost: 0.0,
            request_count: 0,
        }
    }

    /// Select optimal model tier for request
    pub fn select_model(&mut self, context: &RequestContext) -> Result<ModelTier> {
        self.request_count += 1;
        
        debug!("Selecting model for request {} (age: {}s, quality: {}, batch: {})", 
               self.request_count, context.data_age_seconds, context.quality_requirement, context.batch_size);

        // Check budget constraints
        if self.daily_cost >= self.config.daily_budget_usd {
            warn!("Daily budget exceeded (${:.2}), forcing Cold model", self.daily_cost);
            return Ok(ModelTier::Cold);
        }

        // Priority-based selection
        match context.priority {
            RequestPriority::Critical => {
                // Always use best available model
                if context.data_age_seconds < self.config.hot_threshold_minutes * 60 {
                    self.update_cost(&ModelTier::Hot, context.batch_size);
                    Ok(ModelTier::Hot)
                } else {
                    self.update_cost(&ModelTier::Warm, context.batch_size);
                    Ok(ModelTier::Warm)
                }
            },
            RequestPriority::High => {
                // Prefer quality, but consider cost
                if context.quality_requirement > self.config.quality_threshold {
                    if context.data_age_seconds < self.config.hot_threshold_minutes * 60 {
                        self.update_cost(&ModelTier::Hot, context.batch_size);
                        Ok(ModelTier::Hot)
                    } else {
                        self.update_cost(&ModelTier::Warm, context.batch_size);
                        Ok(ModelTier::Warm)
                    }
                } else {
                    self.select_cost_optimized(context)
                }
            },
            RequestPriority::Normal => {
                // Balance cost and quality
                self.select_cost_optimized(context)
            },
            RequestPriority::Low => {
                // Minimize cost
                Ok(ModelTier::Cold)
            }
        }
    }

    /// Select cost-optimized model
    fn select_cost_optimized(&mut self, context: &RequestContext) -> Result<ModelTier> {
        let remaining_budget = self.config.daily_budget_usd - self.daily_cost;
        
        // Data age-based selection
        if context.data_age_seconds < self.config.hot_threshold_minutes * 60 {
            // Recent data - consider Hot model if budget allows
            let hot_cost = self.estimate_cost(&ModelTier::Hot, context.batch_size);
            if hot_cost <= remaining_budget * 0.5 { // Reserve 50% budget
                self.update_cost(&ModelTier::Hot, context.batch_size);
                Ok(ModelTier::Hot)
            } else {
                Ok(ModelTier::Cold)
            }
        } else if context.data_age_seconds < self.config.warm_threshold_hours * 3600 {
            // Medium-age data - consider Warm model
            let warm_cost = self.estimate_cost(&ModelTier::Warm, context.batch_size);
            if warm_cost <= remaining_budget * 0.3 { // Reserve 70% budget
                self.update_cost(&ModelTier::Warm, context.batch_size);
                Ok(ModelTier::Warm)
            } else {
                Ok(ModelTier::Cold)
            }
        } else {
            // Old data - use Cold model
            Ok(ModelTier::Cold)
        }
    }

    /// Estimate cost for model tier
    fn estimate_cost(&self, tier: &ModelTier, batch_size: usize) -> f64 {
        let tokens_per_request = 1000.0; // Average tokens
        let total_tokens = tokens_per_request * batch_size as f64;
        
        match tier {
            ModelTier::Hot => total_tokens * 0.15 / 1_000_000.0,  // $0.15/1M tokens
            ModelTier::Warm => total_tokens * 5.0 / 1_000_000.0,  // $5/1M tokens
            ModelTier::Cold => 0.0,                               // Free
        }
    }

    /// Update daily cost tracking
    fn update_cost(&mut self, tier: &ModelTier, batch_size: usize) {
        let cost = self.estimate_cost(tier, batch_size);
        self.daily_cost += cost;
        
        debug!("Updated cost: +${:.4} = ${:.2} total (tier: {:?})", cost, self.daily_cost, tier);
    }

    /// Get current statistics
    pub fn get_stats(&self) -> ModelRouterStats {
        ModelRouterStats {
            daily_cost: self.daily_cost,
            remaining_budget: self.config.daily_budget_usd - self.daily_cost,
            request_count: self.request_count,
            budget_utilization: (self.daily_cost / self.config.daily_budget_usd * 100.0).min(100.0),
        }
    }

    /// Reset daily counters (call at midnight)
    pub fn reset_daily_counters(&mut self) {
        info!("Resetting daily counters - Final cost: ${:.2}", self.daily_cost);
        self.daily_cost = 0.0;
        self.request_count = 0;
    }
}

/// Model router statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelRouterStats {
    pub daily_cost: f64,
    pub remaining_budget: f64,
    pub request_count: u64,
    pub budget_utilization: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_selection_by_age() {
        let config = ModelConfig::default();
        let mut router = ModelRouter::new(config);

        // Recent data should select Hot model
        let context = RequestContext {
            data_age_seconds: 60, // 1 minute
            quality_requirement: 0.9,
            batch_size: 10,
            priority: RequestPriority::Normal,
        };

        let model = router.select_model(&context).unwrap();
        assert!(matches!(model, ModelTier::Hot));
    }

    #[test]
    fn test_budget_constraints() {
        let config = ModelConfig {
            daily_budget_usd: 1.0, // Very low budget
            ..Default::default()
        };
        let mut router = ModelRouter::new(config);

        // Force budget exhaustion
        router.daily_cost = 1.0;

        let context = RequestContext {
            data_age_seconds: 60,
            quality_requirement: 0.9,
            batch_size: 100,
            priority: RequestPriority::Normal,
        };

        let model = router.select_model(&context).unwrap();
        assert!(matches!(model, ModelTier::Cold));
    }

    #[test]
    fn test_critical_priority() {
        let config = ModelConfig::default();
        let mut router = ModelRouter::new(config);

        let context = RequestContext {
            data_age_seconds: 60,
            quality_requirement: 0.5, // Low quality requirement
            batch_size: 10,
            priority: RequestPriority::Critical, // But critical priority
        };

        let model = router.select_model(&context).unwrap();
        assert!(matches!(model, ModelTier::Hot)); // Should still use Hot
    }
}
