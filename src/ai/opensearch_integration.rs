//! OpenSearch AI Integration for Solana HFT Ninja
//!
//! Intelligent search and analytics for trading patterns

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// OpenSearch AI Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenSearchConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub index_prefix: String,
    pub vector_dimensions: u32,
    pub similarity_algorithm: String,
    pub refresh_interval: String,
    pub indices: IndexConfig,
    pub vector_search: VectorSearchConfig,
    pub analytics: AnalyticsConfig,
    pub performance: PerformanceConfig,
}

/// Index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    pub market_patterns: String,
    pub wallet_behaviors: String,
    pub price_movements: String,
    pub transaction_flows: String,
    pub mev_opportunities: String,
}

/// Vector search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchConfig {
    pub enabled: bool,
    pub embedding_model: String,
    pub search_timeout_ms: u64,
    pub max_results: u32,
    pub min_score: f64,
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub anomaly_detection: bool,
    pub trend_analysis: bool,
    pub correlation_analysis: bool,
    pub predictive_modeling: bool,
    pub real_time_alerts: bool,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub bulk_size: u32,
    pub flush_interval_ms: u64,
    pub refresh_policy: String,
    pub replica_count: u32,
    pub shard_count: u32,
}

impl Default for OpenSearchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "http://localhost:9200".to_string(),
            index_prefix: "hft_ninja".to_string(),
            vector_dimensions: 768,
            similarity_algorithm: "cosine".to_string(),
            refresh_interval: "1s".to_string(),
            indices: IndexConfig {
                market_patterns: "hft_ninja_patterns".to_string(),
                wallet_behaviors: "hft_ninja_wallets".to_string(),
                price_movements: "hft_ninja_prices".to_string(),
                transaction_flows: "hft_ninja_transactions".to_string(),
                mev_opportunities: "hft_ninja_mev".to_string(),
            },
            vector_search: VectorSearchConfig {
                enabled: true,
                embedding_model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
                search_timeout_ms: 100,
                max_results: 50,
                min_score: 0.7,
            },
            analytics: AnalyticsConfig {
                anomaly_detection: true,
                trend_analysis: true,
                correlation_analysis: true,
                predictive_modeling: true,
                real_time_alerts: true,
            },
            performance: PerformanceConfig {
                bulk_size: 1000,
                flush_interval_ms: 5000,
                refresh_policy: "wait_for".to_string(),
                replica_count: 0,
                shard_count: 1,
            },
        }
    }
}

/// Search result from OpenSearch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f64,
    pub source: Value,
    pub highlights: Option<HashMap<String, Vec<String>>>,
}

/// Pattern analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAnalysis {
    pub pattern_type: PatternType,
    pub confidence: f64,
    pub similar_patterns: Vec<SearchResult>,
    pub risk_indicators: Vec<RiskIndicator>,
    pub recommendations: Vec<String>,
}

/// Types of detected patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    BullishBreakout,
    BearishBreakdown,
    RugPullPattern,
    WhaleAccumulation,
    BotTrading,
    LiquidityDrain,
    PumpAndDump,
    OrganicGrowth,
}

/// Risk indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskIndicator {
    pub indicator_type: String,
    pub severity: RiskSeverity,
    pub description: String,
    pub confidence: f64,
}

/// Risk severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyResult {
    pub anomaly_type: AnomalyType,
    pub severity: f64,
    pub description: String,
    pub affected_tokens: Vec<String>,
    pub timestamp: u64,
}

/// Types of anomalies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    PriceSpike,
    VolumeSpike,
    LiquidityDrop,
    UnusualTradingPattern,
    SuspiciousWalletActivity,
    NetworkCongestion,
}

/// OpenSearch AI Engine
pub struct OpenSearchEngine {
    config: OpenSearchConfig,
    client: reqwest::Client,
    embedding_cache: RwLock<HashMap<String, Vec<f32>>>,
    pattern_cache: RwLock<HashMap<String, PatternAnalysis>>,
}

impl OpenSearchEngine {
    /// Create new OpenSearch AI engine
    pub fn new(config: OpenSearchConfig) -> Result<Self> {
        info!("🔍 Initializing OpenSearch AI Engine...");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(
                config.vector_search.search_timeout_ms,
            ))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            config,
            client,
            embedding_cache: RwLock::new(HashMap::new()),
            pattern_cache: RwLock::new(HashMap::new()),
        })
    }

    /// Initialize indices and mappings
    pub async fn initialize(&self) -> Result<()> {
        if !self.config.enabled {
            warn!("🔍 OpenSearch AI is disabled");
            return Ok(());
        }

        info!("🔍 Setting up OpenSearch indices...");

        // Create indices with proper mappings
        self.create_index(
            &self.config.indices.market_patterns,
            &self.get_pattern_mapping(),
        )
        .await?;
        self.create_index(
            &self.config.indices.wallet_behaviors,
            &self.get_wallet_mapping(),
        )
        .await?;
        self.create_index(
            &self.config.indices.price_movements,
            &self.get_price_mapping(),
        )
        .await?;
        self.create_index(
            &self.config.indices.transaction_flows,
            &self.get_transaction_mapping(),
        )
        .await?;
        self.create_index(
            &self.config.indices.mev_opportunities,
            &self.get_mev_mapping(),
        )
        .await?;

        info!("🔍 OpenSearch AI Engine initialized successfully");
        Ok(())
    }

    /// Create index with mapping
    async fn create_index(&self, index_name: &str, mapping: &Value) -> Result<()> {
        let url = format!("{}/{}", self.config.endpoint, index_name);

        let response = self
            .client
            .put(&url)
            .json(mapping)
            .send()
            .await
            .context("Failed to create index")?;

        if response.status().is_success() {
            debug!("🔍 Created index: {}", index_name);
        } else {
            warn!(
                "🔍 Index {} may already exist or failed to create",
                index_name
            );
        }

        Ok(())
    }

    /// Index market pattern data
    pub async fn index_pattern(&self, pattern_data: &Value) -> Result<String> {
        if !self.config.enabled {
            return Ok("disabled".to_string());
        }

        let url = format!(
            "{}/{}/_doc",
            self.config.endpoint, self.config.indices.market_patterns
        );

        let response = self
            .client
            .post(&url)
            .json(pattern_data)
            .send()
            .await
            .context("Failed to index pattern")?;

        let result: Value = response.json().await?;
        let doc_id = result["_id"].as_str().unwrap_or("unknown").to_string();

        debug!("🔍 Indexed pattern with ID: {}", doc_id);
        Ok(doc_id)
    }

    /// Search for similar patterns
    pub async fn search_similar_patterns(
        &self,
        query_vector: &[f32],
        pattern_type: Option<PatternType>,
    ) -> Result<Vec<SearchResult>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        let mut query = json!({
            "size": self.config.vector_search.max_results,
            "min_score": self.config.vector_search.min_score,
            "query": {
                "script_score": {
                    "query": {"match_all": {}},
                    "script": {
                        "source": "cosineSimilarity(params.query_vector, 'pattern_vector') + 1.0",
                        "params": {
                            "query_vector": query_vector
                        }
                    }
                }
            }
        });

        // Add pattern type filter if specified
        if let Some(pt) = pattern_type {
            query["query"]["script_score"]["query"] = json!({
                "term": {
                    "pattern_type": format!("{:?}", pt)
                }
            });
        }

        let url = format!(
            "{}/{}/_search",
            self.config.endpoint, self.config.indices.market_patterns
        );

        let response = self
            .client
            .post(&url)
            .json(&query)
            .send()
            .await
            .context("Failed to search patterns")?;

        let result: Value = response.json().await?;
        let empty_vec = vec![];
        let hits = result["hits"]["hits"].as_array().unwrap_or(&empty_vec);

        let search_results: Vec<SearchResult> = hits
            .iter()
            .map(|hit| SearchResult {
                id: hit["_id"].as_str().unwrap_or("").to_string(),
                score: hit["_score"].as_f64().unwrap_or(0.0),
                source: hit["_source"].clone(),
                highlights: None,
            })
            .collect();

        debug!("🔍 Found {} similar patterns", search_results.len());
        Ok(search_results)
    }

    /// Detect anomalies in market data
    pub async fn detect_anomalies(&self, market_data: &Value) -> Result<Vec<AnomalyResult>> {
        if !self.config.enabled || !self.config.analytics.anomaly_detection {
            return Ok(Vec::new());
        }

        // Simulate anomaly detection (replace with actual OpenSearch ML)
        let mut anomalies = Vec::new();

        // Check for price spikes
        if let Some(price_change) = market_data["price_change_24h"].as_f64() {
            if price_change.abs() > 50.0 {
                anomalies.push(AnomalyResult {
                    anomaly_type: if price_change > 0.0 {
                        AnomalyType::PriceSpike
                    } else {
                        AnomalyType::PriceSpike
                    },
                    severity: (price_change.abs() / 100.0).min(1.0),
                    description: format!("Unusual price movement: {:.2}%", price_change),
                    affected_tokens: vec![market_data["token_address"]
                        .as_str()
                        .unwrap_or("unknown")
                        .to_string()],
                    timestamp: chrono::Utc::now().timestamp() as u64,
                });
            }
        }

        // Check for volume spikes
        if let Some(volume_change) = market_data["volume_change_24h"].as_f64() {
            if volume_change > 200.0 {
                anomalies.push(AnomalyResult {
                    anomaly_type: AnomalyType::VolumeSpike,
                    severity: (volume_change / 500.0).min(1.0),
                    description: format!("Unusual volume spike: {:.2}%", volume_change),
                    affected_tokens: vec![market_data["token_address"]
                        .as_str()
                        .unwrap_or("unknown")
                        .to_string()],
                    timestamp: chrono::Utc::now().timestamp() as u64,
                });
            }
        }

        debug!("🔍 Detected {} anomalies", anomalies.len());
        Ok(anomalies)
    }

    /// Analyze wallet behavior patterns
    pub async fn analyze_wallet_behavior(&self, wallet_address: &str) -> Result<PatternAnalysis> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("OpenSearch AI is disabled"));
        }

        // Check cache first
        let cache_key = format!("wallet_{}", wallet_address);
        {
            let cache = self.pattern_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        // Search for wallet patterns
        let query = json!({
            "size": 100,
            "query": {
                "term": {
                    "wallet_address": wallet_address
                }
            },
            "sort": [
                {"timestamp": {"order": "desc"}}
            ]
        });

        let url = format!(
            "{}/{}/_search",
            self.config.endpoint, self.config.indices.wallet_behaviors
        );

        let response = self
            .client
            .post(&url)
            .json(&query)
            .send()
            .await
            .context("Failed to search wallet behavior")?;

        let result: Value = response.json().await?;
        let empty_vec2 = vec![];
        let hits = result["hits"]["hits"].as_array().unwrap_or(&empty_vec2);

        // Analyze patterns (simplified)
        let pattern_analysis = PatternAnalysis {
            pattern_type: PatternType::OrganicGrowth, // Default
            confidence: 0.75,
            similar_patterns: hits
                .iter()
                .take(5)
                .map(|hit| SearchResult {
                    id: hit["_id"].as_str().unwrap_or("").to_string(),
                    score: hit["_score"].as_f64().unwrap_or(0.0),
                    source: hit["_source"].clone(),
                    highlights: None,
                })
                .collect(),
            risk_indicators: vec![RiskIndicator {
                indicator_type: "transaction_frequency".to_string(),
                severity: RiskSeverity::Low,
                description: "Normal transaction frequency".to_string(),
                confidence: 0.8,
            }],
            recommendations: vec![
                "Monitor for unusual activity patterns".to_string(),
                "Track large transactions".to_string(),
            ],
        };

        // Cache result
        {
            let mut cache = self.pattern_cache.write().await;
            cache.insert(cache_key, pattern_analysis.clone());
        }

        Ok(pattern_analysis)
    }

    /// Get index mapping for patterns
    fn get_pattern_mapping(&self) -> Value {
        json!({
            "mappings": {
                "properties": {
                    "pattern_type": {"type": "keyword"},
                    "confidence": {"type": "float"},
                    "timestamp": {"type": "date"},
                    "token_address": {"type": "keyword"},
                    "pattern_vector": {
                        "type": "dense_vector",
                        "dims": self.config.vector_dimensions
                    },
                    "price_data": {"type": "object"},
                    "volume_data": {"type": "object"},
                    "technical_indicators": {"type": "object"}
                }
            },
            "settings": {
                "number_of_shards": self.config.performance.shard_count,
                "number_of_replicas": self.config.performance.replica_count,
                "refresh_interval": self.config.refresh_interval
            }
        })
    }

    /// Get index mapping for wallets
    fn get_wallet_mapping(&self) -> Value {
        json!({
            "mappings": {
                "properties": {
                    "wallet_address": {"type": "keyword"},
                    "behavior_type": {"type": "keyword"},
                    "risk_score": {"type": "float"},
                    "timestamp": {"type": "date"},
                    "transaction_count": {"type": "integer"},
                    "total_volume": {"type": "float"},
                    "success_rate": {"type": "float"},
                    "behavior_vector": {
                        "type": "dense_vector",
                        "dims": self.config.vector_dimensions
                    }
                }
            }
        })
    }

    /// Get index mapping for prices
    fn get_price_mapping(&self) -> Value {
        json!({
            "mappings": {
                "properties": {
                    "token_address": {"type": "keyword"},
                    "price": {"type": "float"},
                    "volume": {"type": "float"},
                    "timestamp": {"type": "date"},
                    "exchange": {"type": "keyword"},
                    "price_change_1h": {"type": "float"},
                    "price_change_24h": {"type": "float"},
                    "volume_change_24h": {"type": "float"}
                }
            }
        })
    }

    /// Get index mapping for transactions
    fn get_transaction_mapping(&self) -> Value {
        json!({
            "mappings": {
                "properties": {
                    "signature": {"type": "keyword"},
                    "from_address": {"type": "keyword"},
                    "to_address": {"type": "keyword"},
                    "amount": {"type": "float"},
                    "token_address": {"type": "keyword"},
                    "timestamp": {"type": "date"},
                    "transaction_type": {"type": "keyword"},
                    "success": {"type": "boolean"},
                    "gas_used": {"type": "integer"}
                }
            }
        })
    }

    /// Get index mapping for MEV opportunities
    fn get_mev_mapping(&self) -> Value {
        json!({
            "mappings": {
                "properties": {
                    "opportunity_type": {"type": "keyword"},
                    "profit_potential": {"type": "float"},
                    "risk_score": {"type": "float"},
                    "timestamp": {"type": "date"},
                    "token_addresses": {"type": "keyword"},
                    "dex_involved": {"type": "keyword"},
                    "execution_time_ms": {"type": "integer"},
                    "success": {"type": "boolean"}
                }
            }
        })
    }
}
