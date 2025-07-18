//! AI Module for Solana HFT Ninja
//! 
//! Advanced AI capabilities for trading intelligence

pub mod oumi_integration;
pub mod opensearch_integration;
pub mod lmcache_integration;
pub mod deepseek_client;

pub use oumi_integration::{
    OumiEngine, OumiConfig, TradingPrediction, MarketAnalysis, 
    PredictionType, MarketTrend, RiskLevel, RecommendedAction,
    ActionType, Urgency, MarketData
};

pub use opensearch_integration::{
    OpenSearchEngine, OpenSearchConfig, SearchResult, PatternAnalysis,
    PatternType, RiskIndicator, RiskSeverity, AnomalyResult, AnomalyType,
    IndexConfig, VectorSearchConfig, AnalyticsConfig, PerformanceConfig
};

pub use lmcache_integration::{
    LMCacheEngine, LMCacheConfig, CachedInference, InferenceResult,
    EvictionPolicy, CacheStats
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug, error};

/// Combined AI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub oumi: OumiConfig,
    pub opensearch: OpenSearchConfig,
    pub integration: IntegrationConfig,
}

/// Integration configuration between AI systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub enabled: bool,
    pub cross_validation: bool,
    pub ensemble_predictions: bool,
    pub confidence_threshold: f64,
    pub update_interval_seconds: u64,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cross_validation: true,
            ensemble_predictions: true,
            confidence_threshold: 0.7,
            update_interval_seconds: 30,
        }
    }
}

/// Combined AI prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedPrediction {
    pub oumi_prediction: Option<TradingPrediction>,
    pub opensearch_patterns: Vec<SearchResult>,
    pub anomalies: Vec<AnomalyResult>,
    pub final_confidence: f64,
    pub recommended_action: RecommendedAction,
    pub risk_assessment: RiskAssessment,
}

/// Risk assessment from multiple AI sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk: RiskLevel,
    pub risk_factors: Vec<String>,
    pub confidence: f64,
    pub mitigation_strategies: Vec<String>,
}

/// Main AI coordinator
pub struct AICoordinator {
    config: AIConfig,
    oumi_engine: Arc<OumiEngine>,
    opensearch_engine: Arc<OpenSearchEngine>,
    prediction_cache: RwLock<std::collections::HashMap<String, CombinedPrediction>>,
}

impl AICoordinator {
    /// Create new AI coordinator
    pub fn new(config: AIConfig) -> Result<Self> {
        info!("🤖 Initializing AI Coordinator...");
        
        let oumi_engine = Arc::new(OumiEngine::new(config.oumi.clone())?);
        let opensearch_engine = Arc::new(OpenSearchEngine::new(config.opensearch.clone())?);
        
        Ok(Self {
            config,
            oumi_engine,
            opensearch_engine,
            prediction_cache: RwLock::new(std::collections::HashMap::new()),
        })
    }
    
    /// Initialize all AI systems
    pub async fn initialize(&self) -> Result<()> {
        info!("🤖 Initializing AI systems...");
        
        // Initialize OUMI AI
        if self.config.oumi.enabled {
            self.oumi_engine.initialize().await?;
        }
        
        // Initialize OpenSearch AI
        if self.config.opensearch.enabled {
            self.opensearch_engine.initialize().await?;
        }
        
        info!("🤖 AI Coordinator initialized successfully");
        Ok(())
    }
    
    /// Get combined prediction for a token
    pub async fn get_combined_prediction(&self, token_address: &str, market_data: &MarketData) -> Result<CombinedPrediction> {
        debug!("🤖 Generating combined prediction for token: {}", token_address);
        
        // Check cache first
        let cache_key = format!("{}_{}", token_address, chrono::Utc::now().timestamp() / 60); // 1-minute cache
        {
            let cache = self.prediction_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }
        
        // Get OUMI prediction
        let oumi_prediction = if self.config.oumi.enabled {
            match self.oumi_engine.predict_token(token_address, market_data).await {
                Ok(pred) => Some(pred),
                Err(e) => {
                    warn!("🤖 OUMI prediction failed: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        // Get OpenSearch patterns
        let opensearch_patterns = if self.config.opensearch.enabled {
            // Create a simple query vector (in real implementation, this would be from embeddings)
            let query_vector: Vec<f32> = vec![0.5; self.config.opensearch.vector_dimensions as usize];
            
            match self.opensearch_engine.search_similar_patterns(&query_vector, None).await {
                Ok(patterns) => patterns,
                Err(e) => {
                    warn!("🤖 OpenSearch pattern search failed: {}", e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };
        
        // Detect anomalies
        let anomalies = if self.config.opensearch.enabled {
            let market_value = serde_json::to_value(market_data)?;
            match self.opensearch_engine.detect_anomalies(&market_value).await {
                Ok(anomalies) => anomalies,
                Err(e) => {
                    warn!("🤖 Anomaly detection failed: {}", e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };
        
        // Combine predictions
        let combined = self.combine_predictions(oumi_prediction, &opensearch_patterns, &anomalies).await?;
        
        // Cache result
        {
            let mut cache = self.prediction_cache.write().await;
            cache.insert(cache_key, combined.clone());
        }
        
        Ok(combined)
    }
    
    /// Combine predictions from multiple AI sources
    async fn combine_predictions(
        &self,
        oumi_prediction: Option<TradingPrediction>,
        opensearch_patterns: &[SearchResult],
        anomalies: &[AnomalyResult],
    ) -> Result<CombinedPrediction> {
        
        // Calculate final confidence based on multiple sources
        let mut confidence_scores = Vec::new();
        
        // OUMI confidence
        if let Some(ref pred) = oumi_prediction {
            confidence_scores.push(pred.confidence);
        }
        
        // OpenSearch pattern confidence
        if !opensearch_patterns.is_empty() {
            let avg_pattern_score = opensearch_patterns.iter()
                .map(|p| p.score)
                .sum::<f64>() / opensearch_patterns.len() as f64;
            confidence_scores.push(avg_pattern_score / 10.0); // Normalize to 0-1
        }
        
        // Anomaly impact on confidence
        let anomaly_impact = anomalies.iter()
            .map(|a| a.severity)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);
        
        // Final confidence calculation
        let final_confidence = if confidence_scores.is_empty() {
            0.5 // Default neutral confidence
        } else {
            let base_confidence = confidence_scores.iter().sum::<f64>() / confidence_scores.len() as f64;
            // Reduce confidence if anomalies detected
            base_confidence * (1.0 - anomaly_impact * 0.3)
        };
        
        // Determine recommended action
        let recommended_action = self.determine_action(&oumi_prediction, opensearch_patterns, anomalies, final_confidence).await;
        
        // Risk assessment
        let risk_assessment = self.assess_risk(&oumi_prediction, anomalies, final_confidence).await;
        
        Ok(CombinedPrediction {
            oumi_prediction,
            opensearch_patterns: opensearch_patterns.to_vec(),
            anomalies: anomalies.to_vec(),
            final_confidence,
            recommended_action,
            risk_assessment,
        })
    }
    
    /// Determine recommended action based on all AI inputs
    async fn determine_action(
        &self,
        oumi_prediction: &Option<TradingPrediction>,
        _opensearch_patterns: &[SearchResult],
        anomalies: &[AnomalyResult],
        confidence: f64,
    ) -> RecommendedAction {
        
        // High anomaly severity = avoid
        if anomalies.iter().any(|a| a.severity > 0.8) {
            return RecommendedAction {
                action_type: ActionType::AvoidToken,
                token_address: None,
                confidence: 0.9,
                urgency: Urgency::High,
                reasoning: "High-severity anomalies detected".to_string(),
            };
        }
        
        // Use OUMI prediction if available and confident
        if let Some(pred) = oumi_prediction {
            if pred.confidence > self.config.integration.confidence_threshold {
                let action_type = match pred.prediction_type {
                    PredictionType::PriceIncrease => ActionType::Buy,
                    PredictionType::PriceDecrease => ActionType::Sell,
                    PredictionType::RugPull => ActionType::AvoidToken,
                    PredictionType::WhaleActivity => ActionType::Hold,
                    _ => ActionType::Hold,
                };
                
                return RecommendedAction {
                    action_type,
                    token_address: Some(pred.token_address.clone()),
                    confidence: pred.confidence,
                    urgency: if pred.confidence > 0.9 { Urgency::High } else { Urgency::Medium },
                    reasoning: pred.reasoning.clone(),
                };
            }
        }
        
        // Default conservative action
        RecommendedAction {
            action_type: ActionType::Hold,
            token_address: None,
            confidence,
            urgency: Urgency::Low,
            reasoning: "Insufficient confidence for active trading".to_string(),
        }
    }
    
    /// Assess overall risk from all AI sources
    async fn assess_risk(
        &self,
        oumi_prediction: &Option<TradingPrediction>,
        anomalies: &[AnomalyResult],
        confidence: f64,
    ) -> RiskAssessment {
        
        let mut risk_factors = Vec::new();
        let mut risk_score = 0.0;
        
        // OUMI risk factors
        if let Some(pred) = oumi_prediction {
            risk_score += pred.risk_score;
            if pred.risk_score > 0.7 {
                risk_factors.push("High AI-predicted risk".to_string());
            }
        }
        
        // Anomaly risk factors
        for anomaly in anomalies {
            risk_score += anomaly.severity * 0.5; // Weight anomalies
            risk_factors.push(format!("Anomaly detected: {}", anomaly.description));
        }
        
        // Confidence impact on risk
        if confidence < 0.5 {
            risk_factors.push("Low prediction confidence".to_string());
            risk_score += 0.3;
        }
        
        // Determine overall risk level
        let overall_risk = if risk_score > 0.8 {
            RiskLevel::Critical
        } else if risk_score > 0.6 {
            RiskLevel::High
        } else if risk_score > 0.4 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };
        
        // Mitigation strategies
        let mitigation_strategies = vec![
            "Use smaller position sizes".to_string(),
            "Set tight stop-losses".to_string(),
            "Monitor for exit signals".to_string(),
            "Diversify across multiple tokens".to_string(),
        ];
        
        RiskAssessment {
            overall_risk,
            risk_factors,
            confidence,
            mitigation_strategies,
        }
    }
    
    /// Get market analysis from OUMI
    pub async fn get_market_analysis(&self, market_data: &MarketData) -> Result<MarketAnalysis> {
        if !self.config.oumi.enabled {
            return Err(anyhow::anyhow!("OUMI AI is disabled"));
        }
        
        self.oumi_engine.analyze_market(market_data).await
    }
    
    /// Analyze wallet behavior using OpenSearch
    pub async fn analyze_wallet(&self, wallet_address: &str) -> Result<PatternAnalysis> {
        if !self.config.opensearch.enabled {
            return Err(anyhow::anyhow!("OpenSearch AI is disabled"));
        }
        
        self.opensearch_engine.analyze_wallet_behavior(wallet_address).await
    }
}
