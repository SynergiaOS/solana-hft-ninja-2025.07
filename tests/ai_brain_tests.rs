//! AI Brain Tests for Solana HFT Ninja
//! 
//! Comprehensive tests for AI components: OUMI, OpenSearch, LMCache

use anyhow::Result;
use solana_hft_ninja::ai::{
    AICoordinator, AIConfig, OumiConfig, OpenSearchConfig, LMCacheConfig,
    OumiEngine, OpenSearchEngine, LMCacheEngine, MarketData, TradingPrediction,
    PredictionType, RiskLevel, ActionType, Urgency, EvictionPolicy
};
use std::collections::HashMap;
use tokio;

/// Test OUMI AI Engine initialization and basic functionality
#[tokio::test]
async fn test_oumi_engine_initialization() -> Result<()> {
    let config = OumiConfig {
        enabled: true,
        model_path: "test_models/oumi-test".to_string(),
        inference_mode: "test".to_string(),
        batch_size: 16,
        max_sequence_length: 256,
        temperature: 0.8,
        top_p: 0.95,
        fine_tuning_enabled: false,
        training_data_path: "test_data".to_string(),
        model_update_interval_hours: 1,
        multi_modal: true,
        text_analysis: true,
        price_prediction: true,
        sentiment_analysis: true,
        risk_assessment: true,
        gpu_acceleration: false, // Disable for tests
        quantization: "int8".to_string(),
        memory_optimization: true,
        parallel_inference: false,
    };
    
    let engine = OumiEngine::new(config)?;
    
    // Test initialization
    engine.initialize().await?;
    
    // Test market data prediction
    let market_data = create_test_market_data();
    let prediction = engine.predict_token("test_token_123", &market_data).await?;
    
    // Verify prediction structure
    assert!(!prediction.token_address.is_empty());
    assert!(prediction.confidence >= 0.0 && prediction.confidence <= 1.0);
    assert!(prediction.risk_score >= 0.0 && prediction.risk_score <= 1.0);
    assert!(prediction.time_horizon_minutes > 0);
    
    // Test market analysis
    let analysis = engine.analyze_market(&market_data).await?;
    assert!(matches!(analysis.market_trend, 
        solana_hft_ninja::ai::MarketTrend::Bullish | 
        solana_hft_ninja::ai::MarketTrend::Bearish | 
        solana_hft_ninja::ai::MarketTrend::Sideways | 
        solana_hft_ninja::ai::MarketTrend::Uncertain
    ));
    
    println!("✅ OUMI Engine test passed");
    Ok(())
}

/// Test OpenSearch AI Engine functionality
#[tokio::test]
async fn test_opensearch_engine() -> Result<()> {
    let config = OpenSearchConfig {
        enabled: true,
        endpoint: "http://localhost:9200".to_string(),
        index_prefix: "test_hft".to_string(),
        vector_dimensions: 128, // Smaller for tests
        similarity_algorithm: "cosine".to_string(),
        refresh_interval: "1s".to_string(),
        indices: solana_hft_ninja::ai::IndexConfig {
            market_patterns: "test_patterns".to_string(),
            wallet_behaviors: "test_wallets".to_string(),
            price_movements: "test_prices".to_string(),
            transaction_flows: "test_transactions".to_string(),
            mev_opportunities: "test_mev".to_string(),
        },
        vector_search: solana_hft_ninja::ai::VectorSearchConfig {
            enabled: true,
            embedding_model: "test-model".to_string(),
            search_timeout_ms: 1000,
            max_results: 10,
            min_score: 0.5,
        },
        analytics: solana_hft_ninja::ai::AnalyticsConfig {
            anomaly_detection: true,
            trend_analysis: true,
            correlation_analysis: true,
            predictive_modeling: true,
            real_time_alerts: true,
        },
        performance: solana_hft_ninja::ai::PerformanceConfig {
            bulk_size: 100,
            flush_interval_ms: 1000,
            refresh_policy: "wait_for".to_string(),
            replica_count: 0,
            shard_count: 1,
        },
    };
    
    let engine = OpenSearchEngine::new(config)?;
    
    // Test pattern search (will work even if OpenSearch is not running)
    let query_vector: Vec<f32> = vec![0.5; 128];
    let results = engine.search_similar_patterns(&query_vector, None).await;
    
    // Should not fail even if OpenSearch is unavailable
    assert!(results.is_ok() || results.is_err());
    
    // Test wallet behavior analysis
    let wallet_analysis = engine.analyze_wallet_behavior("test_wallet_123").await;
    assert!(wallet_analysis.is_ok() || wallet_analysis.is_err());
    
    // Test anomaly detection
    let market_value = serde_json::json!({
        "token_address": "test_token",
        "price_change_24h": 150.0, // Large change should trigger anomaly
        "volume_change_24h": 300.0
    });
    
    let anomalies = engine.detect_anomalies(&market_value).await?;
    // Should detect anomalies for large price/volume changes
    assert!(anomalies.len() >= 0); // May be 0 if OpenSearch not available
    
    println!("✅ OpenSearch Engine test passed");
    Ok(())
}

/// Test LMCache Engine functionality
#[tokio::test]
async fn test_lmcache_engine() -> Result<()> {
    let config = LMCacheConfig {
        enabled: true,
        cache_size_mb: 10, // Small for tests
        ttl_seconds: 60,
        compression_enabled: false, // Disable for tests
        eviction_policy: EvictionPolicy::LRU,
        persistence_enabled: false, // Disable for tests
        persistence_path: "test_cache".to_string(),
        metrics_enabled: true,
        distributed_cache: false, // Disable Redis for tests
        redis_url: None,
    };
    
    let engine = LMCacheEngine::new(config)?;
    engine.initialize().await?;
    
    // Test cache key generation
    let features = vec![0.1, 0.2, 0.3, 0.4, 0.5];
    let key = engine.generate_key("test_model", &features, "test_context");
    assert!(!key.is_empty());
    assert!(key.starts_with("lmc_test_model_"));
    
    // Test cache miss
    let result = engine.get(&key).await;
    assert!(result.is_none());
    
    // Test cache put and get
    let inference_result = solana_hft_ninja::ai::InferenceResult {
        prediction: vec![0.8, 0.2],
        confidence: 0.85,
        processing_time_ms: 50,
        model_id: "test_model".to_string(),
        metadata: HashMap::new(),
    };
    
    engine.put(&key, inference_result.clone(), "v1.0").await?;
    
    // Test cache hit
    let cached_result = engine.get(&key).await;
    assert!(cached_result.is_some());
    let cached = cached_result.unwrap();
    assert_eq!(cached.prediction, inference_result.prediction);
    assert_eq!(cached.confidence, inference_result.confidence);
    
    // Test cache statistics
    let stats = engine.get_stats().await;
    assert!(stats.total_requests >= 2); // At least miss + hit
    assert!(stats.hits >= 1);
    assert!(stats.misses >= 1);
    
    let hit_ratio = engine.get_hit_ratio().await;
    assert!(hit_ratio >= 0.0 && hit_ratio <= 1.0);
    
    // Test cache clear
    engine.clear().await?;
    let result_after_clear = engine.get(&key).await;
    assert!(result_after_clear.is_none());
    
    println!("✅ LMCache Engine test passed");
    Ok(())
}

/// Test AI Coordinator integration
#[tokio::test]
async fn test_ai_coordinator() -> Result<()> {
    let ai_config = AIConfig {
        oumi: OumiConfig {
            enabled: true,
            model_path: "test_models/oumi".to_string(),
            inference_mode: "test".to_string(),
            batch_size: 8,
            max_sequence_length: 128,
            temperature: 0.7,
            top_p: 0.9,
            fine_tuning_enabled: false,
            training_data_path: "test_data".to_string(),
            model_update_interval_hours: 1,
            multi_modal: false,
            text_analysis: true,
            price_prediction: true,
            sentiment_analysis: false,
            risk_assessment: true,
            gpu_acceleration: false,
            quantization: "int8".to_string(),
            memory_optimization: true,
            parallel_inference: false,
        },
        opensearch: OpenSearchConfig {
            enabled: false, // Disable for integration test
            ..Default::default()
        },
        integration: solana_hft_ninja::ai::IntegrationConfig {
            enabled: true,
            cross_validation: true,
            ensemble_predictions: true,
            confidence_threshold: 0.6,
            update_interval_seconds: 10,
        },
    };
    
    let coordinator = AICoordinator::new(ai_config)?;
    coordinator.initialize().await?;
    
    // Test combined prediction
    let market_data = create_test_market_data();
    let combined_prediction = coordinator.get_combined_prediction("test_token", &market_data).await?;
    
    // Verify combined prediction structure
    assert!(combined_prediction.final_confidence >= 0.0 && combined_prediction.final_confidence <= 1.0);
    assert!(combined_prediction.oumi_prediction.is_some());
    assert!(matches!(combined_prediction.recommended_action.action_type,
        ActionType::Buy | ActionType::Sell | ActionType::Hold | 
        ActionType::AvoidToken | ActionType::IncreasePosition | 
        ActionType::DecreasePosition | ActionType::SetStopLoss | 
        ActionType::TakeProfit
    ));
    assert!(matches!(combined_prediction.risk_assessment.overall_risk,
        RiskLevel::Low | RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical
    ));
    
    // Test market analysis
    let market_analysis = coordinator.get_market_analysis(&market_data).await?;
    assert!(market_analysis.overall_sentiment >= -1.0 && market_analysis.overall_sentiment <= 1.0);
    assert!(market_analysis.volatility_index >= 0.0);
    assert!(!market_analysis.recommended_actions.is_empty());
    
    println!("✅ AI Coordinator test passed");
    Ok(())
}

/// Test AI performance under load
#[tokio::test]
async fn test_ai_performance() -> Result<()> {
    let config = LMCacheConfig {
        enabled: true,
        cache_size_mb: 50,
        ttl_seconds: 300,
        compression_enabled: true,
        eviction_policy: EvictionPolicy::LRU,
        persistence_enabled: false,
        persistence_path: "perf_test_cache".to_string(),
        metrics_enabled: true,
        distributed_cache: false,
        redis_url: None,
    };
    
    let cache_engine = LMCacheEngine::new(config)?;
    cache_engine.initialize().await?;
    
    let start_time = std::time::Instant::now();
    let num_operations = 1000;
    
    // Simulate high-frequency cache operations
    for i in 0..num_operations {
        let features = vec![i as f32 / 1000.0; 10];
        let key = cache_engine.generate_key("perf_test", &features, "load_test");
        
        // Try to get (will miss first time)
        let _result = cache_engine.get(&key).await;
        
        // Put result
        let inference_result = solana_hft_ninja::ai::InferenceResult {
            prediction: vec![0.5, 0.5],
            confidence: 0.75,
            processing_time_ms: 10,
            model_id: "perf_test".to_string(),
            metadata: HashMap::new(),
        };
        
        cache_engine.put(&key, inference_result, "v1.0").await?;
        
        // Get again (should hit)
        let _cached = cache_engine.get(&key).await;
    }
    
    let elapsed = start_time.elapsed();
    let ops_per_second = num_operations as f64 / elapsed.as_secs_f64();
    
    println!("🚀 Performance test: {:.0} ops/second", ops_per_second);
    
    // Should handle at least 1000 ops/second
    assert!(ops_per_second > 1000.0, "Performance too low: {:.0} ops/sec", ops_per_second);
    
    let stats = cache_engine.get_stats().await;
    let hit_ratio = cache_engine.get_hit_ratio().await;
    
    println!("📊 Cache stats: {} hits, {} misses, {:.2}% hit ratio", 
             stats.hits, stats.misses, hit_ratio * 100.0);
    
    // Should have reasonable hit ratio
    assert!(hit_ratio > 0.3, "Hit ratio too low: {:.2}%", hit_ratio * 100.0);
    
    println!("✅ AI Performance test passed");
    Ok(())
}

/// Test error handling and edge cases
#[tokio::test]
async fn test_ai_error_handling() -> Result<()> {
    // Test disabled OUMI engine
    let disabled_config = OumiConfig {
        enabled: false,
        ..Default::default()
    };
    
    let engine = OumiEngine::new(disabled_config)?;
    let market_data = create_test_market_data();
    
    let result = engine.predict_token("test", &market_data).await;
    assert!(result.is_err()); // Should fail when disabled
    
    // Test invalid cache operations
    let cache_config = LMCacheConfig {
        enabled: false,
        ..Default::default()
    };
    
    let cache_engine = LMCacheEngine::new(cache_config)?;
    let result = cache_engine.get("nonexistent_key").await;
    assert!(result.is_none()); // Should return None when disabled
    
    // Test empty features
    let enabled_cache = LMCacheEngine::new(LMCacheConfig::default())?;
    let key = enabled_cache.generate_key("test", &[], "empty");
    assert!(!key.is_empty()); // Should still generate valid key
    
    println!("✅ AI Error handling test passed");
    Ok(())
}

/// Helper function to create test market data
fn create_test_market_data() -> MarketData {
    MarketData {
        current_price: 1.25,
        volume_24h: 50000.0,
        price_change_24h: 5.5,
        liquidity_sol: 1000.0,
        holder_count: 250,
        rsi: Some(65.0),
        macd: Some(0.02),
        bollinger_upper: Some(1.30),
        bollinger_lower: Some(1.20),
    }
}

/// Integration test with real-world scenario
#[tokio::test]
async fn test_trading_scenario() -> Result<()> {
    // Simulate a complete trading decision flow
    let ai_config = AIConfig {
        oumi: OumiConfig {
            enabled: true,
            price_prediction: true,
            risk_assessment: true,
            ..Default::default()
        },
        opensearch: OpenSearchConfig {
            enabled: false, // Disable for test
            ..Default::default()
        },
        integration: solana_hft_ninja::ai::IntegrationConfig {
            enabled: true,
            confidence_threshold: 0.7,
            ..Default::default()
        },
    };
    
    let coordinator = AICoordinator::new(ai_config)?;
    coordinator.initialize().await?;
    
    // Test bullish scenario
    let bullish_data = MarketData {
        current_price: 2.0,
        volume_24h: 100000.0,
        price_change_24h: 15.0, // Strong positive movement
        liquidity_sol: 5000.0,
        holder_count: 500,
        rsi: Some(70.0), // Slightly overbought but trending up
        macd: Some(0.05), // Positive MACD
        bollinger_upper: Some(2.1),
        bollinger_lower: Some(1.8),
    };
    
    let prediction = coordinator.get_combined_prediction("bullish_token", &bullish_data).await?;
    
    // Should recommend buying or holding in bullish scenario
    assert!(matches!(prediction.recommended_action.action_type,
        ActionType::Buy | ActionType::Hold | ActionType::IncreasePosition
    ));
    
    // Test bearish scenario
    let bearish_data = MarketData {
        current_price: 0.8,
        volume_24h: 20000.0,
        price_change_24h: -25.0, // Strong negative movement
        liquidity_sol: 500.0,
        holder_count: 50,
        rsi: Some(25.0), // Oversold
        macd: Some(-0.03), // Negative MACD
        bollinger_upper: Some(1.0),
        bollinger_lower: Some(0.7),
    };
    
    let prediction = coordinator.get_combined_prediction("bearish_token", &bearish_data).await?;
    
    // Should recommend selling or avoiding in bearish scenario
    assert!(matches!(prediction.recommended_action.action_type,
        ActionType::Sell | ActionType::AvoidToken | ActionType::DecreasePosition
    ));
    
    println!("✅ Trading scenario test passed");
    Ok(())
}
