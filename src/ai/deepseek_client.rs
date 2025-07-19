/*!
🧮 DeepSeek-Math Client for Rust HFT Ninja
Cost-effective AI calculations for small portfolio trading.
Optimized for <$1 daily operational cost with smart caching.
*/

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Position sizing request for Kelly Criterion calculation
#[derive(Debug, Clone, Serialize)]
pub struct PositionSizeRequest {
    pub capital: f64,
    pub risk_tolerance: f64,
    pub expected_return: f64,
    pub volatility: f64,
    pub strategy: String,
}

/// Arbitrage profit calculation request
#[derive(Debug, Clone, Serialize)]
pub struct ArbitrageProfitRequest {
    pub token: String,
    pub price_a: f64,
    pub price_b: f64,
    pub liquidity_a: f64,
    pub liquidity_b: f64,
    pub gas_cost: f64,
}

/// Sandwich attack calculation request
#[derive(Debug, Clone, Serialize)]
pub struct SandwichCalculationRequest {
    pub target_tx_size: f64,
    pub pool_liquidity: f64,
    pub current_price: f64,
    pub slippage: f64,
}

/// Risk assessment request
#[derive(Debug, Clone, Serialize)]
pub struct RiskAssessmentRequest {
    pub strategy: String,
    pub token: String,
    pub position_size: f64,
    pub market_conditions: HashMap<String, serde_json::Value>,
    pub volatility: f64,
    pub liquidity: f64,
}

/// Trading calculation response
#[derive(Debug, Clone, Deserialize)]
pub struct CalculationResponse {
    pub calculation_type: String,
    pub result: serde_json::Value,
    pub confidence: f64,
    pub reasoning: String,
    pub execution_time_ms: u64,
    pub model_used: String,
    pub timestamp: f64,
}

/// Risk assessment response
#[derive(Debug, Clone, Deserialize)]
pub struct RiskAssessmentResponse {
    pub risk_score: f64,
    pub risk_factors: Vec<String>,
    pub recommended_position_size: f64,
    pub max_loss_estimate: f64,
    pub confidence: f64,
    pub reasoning: String,
    pub timestamp: f64,
}

/// DeepSeek-Math API health status
#[derive(Debug, Clone, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub model_loaded: bool,
    pub memory_usage_mb: f64,
    pub cache_hit_ratio: f64,
    pub uptime_seconds: f64,
}

/// DeepSeek-Math performance metrics
#[derive(Debug, Clone, Deserialize)]
pub struct DeepSeekMetrics {
    pub model_name: String,
    pub calculations_performed: u64,
    pub average_latency: f64,
    pub cache_hit_ratio: f64,
    pub memory_usage_mb: f64,
    pub uptime_seconds: f64,
}

/// DeepSeek-Math client configuration
#[derive(Debug, Clone)]
pub struct DeepSeekConfig {
    pub api_url: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub enable_caching: bool,
    pub cost_limit_usd: f64,
}

impl Default for DeepSeekConfig {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:8003".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            enable_caching: true,
            cost_limit_usd: 1.0, // $1 daily limit
        }
    }
}

/// DeepSeek-Math client for cost-effective AI calculations
pub struct DeepSeekClient {
    client: Client,
    config: DeepSeekConfig,
    daily_cost_usd: f64,
    request_count: u64,
    cache_hits: u64,
}

impl DeepSeekClient {
    /// Create new DeepSeek-Math client
    pub fn new(config: DeepSeekConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config,
            daily_cost_usd: 0.0,
            request_count: 0,
            cache_hits: 0,
        }
    }

    /// Check if DeepSeek-Math API is healthy
    pub async fn health_check(&self) -> Result<HealthResponse> {
        let url = format!("{}/health", self.config.api_url);

        let response = timeout(Duration::from_secs(10), self.client.get(&url).send())
            .await
            .context("Health check timeout")?
            .context("Health check request failed")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Health check failed: {}",
                response.status()
            ));
        }

        let health: HealthResponse = response
            .json()
            .await
            .context("Failed to parse health response")?;

        debug!("🧮 DeepSeek-Math health: {:?}", health);
        Ok(health)
    }

    /// Calculate optimal position size using Kelly Criterion
    pub async fn calculate_position_size(
        &mut self,
        request: PositionSizeRequest,
    ) -> Result<CalculationResponse> {
        if !self.check_cost_limit().await? {
            return Err(anyhow::anyhow!("Daily cost limit exceeded"));
        }

        let url = format!("{}/calculate/position-size", self.config.api_url);
        let start_time = Instant::now();

        debug!("🧮 Calculating position size: {:?}", request);

        let response = self.make_request(&url, &request).await?;
        let calculation: CalculationResponse = response
            .json()
            .await
            .context("Failed to parse position size response")?;

        let latency = start_time.elapsed().as_millis();
        self.update_metrics(latency as f64, &calculation).await;

        info!(
            "✅ Position size calculated: {:.4} SOL (confidence: {:.2}%, latency: {}ms)",
            calculation
                .result
                .get("position_size")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            calculation.confidence * 100.0,
            latency
        );

        Ok(calculation)
    }

    /// Calculate arbitrage profit potential
    pub async fn calculate_arbitrage_profit(
        &mut self,
        request: ArbitrageProfitRequest,
    ) -> Result<CalculationResponse> {
        if !self.check_cost_limit().await? {
            return Err(anyhow::anyhow!("Daily cost limit exceeded"));
        }

        let url = format!("{}/calculate/arbitrage-profit", self.config.api_url);
        let start_time = Instant::now();

        debug!("🧮 Calculating arbitrage profit: {:?}", request);

        let response = self.make_request(&url, &request).await?;
        let calculation: CalculationResponse = response
            .json()
            .await
            .context("Failed to parse arbitrage response")?;

        let latency = start_time.elapsed().as_millis();
        self.update_metrics(latency as f64, &calculation).await;

        let profit_sol = calculation
            .result
            .get("profit_sol")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let feasible = calculation
            .result
            .get("feasible")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        info!(
            "✅ Arbitrage profit calculated: {:.4} SOL (feasible: {}, confidence: {:.2}%)",
            profit_sol,
            feasible,
            calculation.confidence * 100.0
        );

        Ok(calculation)
    }

    /// Calculate sandwich attack parameters
    pub async fn calculate_sandwich_parameters(
        &mut self,
        request: SandwichCalculationRequest,
    ) -> Result<CalculationResponse> {
        if !self.check_cost_limit().await? {
            return Err(anyhow::anyhow!("Daily cost limit exceeded"));
        }

        let url = format!("{}/calculate/sandwich", self.config.api_url);
        let start_time = Instant::now();

        debug!("🧮 Calculating sandwich parameters: {:?}", request);

        let response = self.make_request(&url, &request).await?;
        let calculation: CalculationResponse = response
            .json()
            .await
            .context("Failed to parse sandwich response")?;

        let latency = start_time.elapsed().as_millis();
        self.update_metrics(latency as f64, &calculation).await;

        let expected_profit = calculation
            .result
            .get("expected_profit")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let risk_score = calculation
            .result
            .get("risk_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5);

        info!(
            "✅ Sandwich parameters calculated: {:.4} SOL profit (risk: {:.2}, confidence: {:.2}%)",
            expected_profit,
            risk_score,
            calculation.confidence * 100.0
        );

        Ok(calculation)
    }

    /// Assess trading risk
    pub async fn assess_trading_risk(
        &mut self,
        request: RiskAssessmentRequest,
    ) -> Result<RiskAssessmentResponse> {
        if !self.check_cost_limit().await? {
            return Err(anyhow::anyhow!("Daily cost limit exceeded"));
        }

        let url = format!("{}/assess/risk", self.config.api_url);
        let start_time = Instant::now();

        debug!("🧮 Assessing trading risk: {:?}", request);

        let response = self.make_request(&url, &request).await?;
        let assessment: RiskAssessmentResponse = response
            .json()
            .await
            .context("Failed to parse risk assessment response")?;

        let latency = start_time.elapsed().as_millis();
        self.update_risk_metrics(latency as f64, &assessment).await;

        info!(
            "✅ Risk assessed: {:.2} score, recommended size: {:.4} SOL (confidence: {:.2}%)",
            assessment.risk_score,
            assessment.recommended_position_size,
            assessment.confidence * 100.0
        );

        Ok(assessment)
    }

    /// Get performance metrics
    pub async fn get_metrics(&self) -> Result<DeepSeekMetrics> {
        let url = format!("{}/metrics", self.config.api_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get metrics")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Metrics request failed: {}",
                response.status()
            ));
        }

        let metrics: DeepSeekMetrics = response
            .json()
            .await
            .context("Failed to parse metrics response")?;

        Ok(metrics)
    }

    /// Clear model cache to reduce memory usage
    pub async fn clear_cache(&self) -> Result<()> {
        let url = format!("{}/cache/clear", self.config.api_url);

        let response = self
            .client
            .post(&url)
            .send()
            .await
            .context("Failed to clear cache")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Cache clear failed: {}", response.status()));
        }

        info!("🧹 DeepSeek-Math cache cleared");
        Ok(())
    }

    /// Check if within daily cost limit
    async fn check_cost_limit(&self) -> Result<bool> {
        if self.daily_cost_usd >= self.config.cost_limit_usd {
            warn!(
                "💰 Daily cost limit reached: ${:.6} >= ${:.6}",
                self.daily_cost_usd, self.config.cost_limit_usd
            );
            return Ok(false);
        }
        Ok(true)
    }

    /// Make HTTP request with retries
    async fn make_request<T: Serialize>(
        &self,
        url: &str,
        payload: &T,
    ) -> Result<reqwest::Response> {
        let mut last_error = None;

        for attempt in 1..=self.config.max_retries {
            match timeout(
                Duration::from_secs(self.config.timeout_seconds),
                self.client.post(url).json(payload).send(),
            )
            .await
            {
                Ok(Ok(response)) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else {
                        last_error = Some(anyhow::anyhow!("HTTP {}", response.status()));
                    }
                }
                Ok(Err(e)) => {
                    last_error = Some(anyhow::anyhow!("Request error: {}", e));
                }
                Err(_) => {
                    last_error = Some(anyhow::anyhow!("Request timeout"));
                }
            }

            if attempt < self.config.max_retries {
                let delay = Duration::from_millis(100 * attempt as u64);
                tokio::time::sleep(delay).await;
                debug!(
                    "🔄 Retrying request (attempt {}/{})",
                    attempt + 1,
                    self.config.max_retries
                );
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
    }

    /// Update performance metrics
    async fn update_metrics(&mut self, latency_ms: f64, calculation: &CalculationResponse) {
        self.request_count += 1;

        // Estimate cost (very rough approximation)
        let estimated_cost = (calculation.execution_time_ms as f64 / 1000.0) * 0.000001;
        self.daily_cost_usd += estimated_cost;

        debug!(
            "📊 DeepSeek metrics: requests={}, cost=${:.6}, latency={:.1}ms",
            self.request_count, self.daily_cost_usd, latency_ms
        );
    }

    /// Update risk assessment metrics
    async fn update_risk_metrics(&mut self, latency_ms: f64, assessment: &RiskAssessmentResponse) {
        self.request_count += 1;

        // Estimate cost for risk assessment
        let estimated_cost = 0.000002; // Slightly higher for risk assessment
        self.daily_cost_usd += estimated_cost;

        debug!(
            "📊 Risk assessment metrics: requests={}, cost=${:.6}, latency={:.1}ms, risk={:.2}",
            self.request_count, self.daily_cost_usd, latency_ms, assessment.risk_score
        );
    }

    /// Get current cost efficiency
    pub fn get_cost_efficiency(&self) -> f64 {
        if self.request_count == 0 {
            return 0.0;
        }
        self.daily_cost_usd / self.request_count as f64
    }

    /// Get cache hit ratio
    pub fn get_cache_hit_ratio(&self) -> f64 {
        if self.request_count == 0 {
            return 0.0;
        }
        self.cache_hits as f64 / self.request_count as f64
    }
}
