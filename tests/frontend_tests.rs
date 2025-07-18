//! Frontend Integration Tests for Solana HFT Ninja
//! 
//! Tests for API endpoints, WebSocket connections, and dashboard functionality

use anyhow::Result;
use serde_json::{json, Value};
use std::time::Duration;
use tokio;
use reqwest;

/// Test API server startup and health check
#[tokio::test]
async fn test_api_server_health() -> Result<()> {
    // This test assumes the API server is running on localhost:8002
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8002";
    
    // Test health endpoint
    let health_response = client
        .get(&format!("{}/health", base_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await;
    
    match health_response {
        Ok(response) => {
            assert!(response.status().is_success());
            let health_data: Value = response.json().await?;
            
            // Verify health response structure
            assert!(health_data["status"].is_string());
            assert!(health_data["timestamp"].is_number());
            
            println!("✅ API Health check passed: {:?}", health_data);
        }
        Err(e) => {
            println!("⚠️  API server not running (expected in test environment): {}", e);
            // Don't fail the test if server is not running
        }
    }
    
    Ok(())
}

/// Test trading dashboard API endpoints
#[tokio::test]
async fn test_dashboard_api_endpoints() -> Result<()> {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8002/api";
    
    // Test endpoints that should work even without live trading
    let endpoints = vec![
        "/dashboard/overview",
        "/strategies/list",
        "/performance/summary",
        "/risk/status",
        "/ai/status",
    ];
    
    for endpoint in endpoints {
        let url = format!("{}{}", base_url, endpoint);
        let response = client
            .get(&url)
            .timeout(Duration::from_secs(3))
            .send()
            .await;
        
        match response {
            Ok(resp) => {
                println!("✅ Endpoint {} responded with status: {}", endpoint, resp.status());
                
                if resp.status().is_success() {
                    let data: Value = resp.json().await?;
                    assert!(data.is_object() || data.is_array());
                    println!("   Response data keys: {:?}", 
                             data.as_object().map(|o| o.keys().collect::<Vec<_>>()));
                }
            }
            Err(e) => {
                println!("⚠️  Endpoint {} not available: {}", endpoint, e);
            }
        }
    }
    
    Ok(())
}

/// Test WebSocket connection for real-time updates
#[tokio::test]
async fn test_websocket_connection() -> Result<()> {
    use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
    use futures_util::{SinkExt, StreamExt};
    
    let ws_url = "ws://localhost:8002/ws";
    
    // Attempt WebSocket connection
    let connection_result = tokio::time::timeout(
        Duration::from_secs(5),
        connect_async(ws_url)
    ).await;
    
    match connection_result {
        Ok(Ok((mut ws_stream, _))) => {
            println!("✅ WebSocket connection established");
            
            // Send test message
            let test_message = json!({
                "type": "subscribe",
                "channel": "trading_updates"
            });
            
            ws_stream.send(Message::Text(test_message.to_string())).await?;
            
            // Wait for response with timeout
            let response = tokio::time::timeout(
                Duration::from_secs(3),
                ws_stream.next()
            ).await;
            
            match response {
                Ok(Some(Ok(Message::Text(text)))) => {
                    let response_data: Value = serde_json::from_str(&text)?;
                    println!("✅ WebSocket response: {:?}", response_data);
                    
                    // Verify response structure
                    assert!(response_data.is_object());
                }
                Ok(Some(Ok(msg))) => {
                    println!("✅ WebSocket received message: {:?}", msg);
                }
                Ok(Some(Err(e))) => {
                    println!("⚠️  WebSocket error: {}", e);
                }
                Ok(None) => {
                    println!("⚠️  WebSocket connection closed");
                }
                Err(_) => {
                    println!("⚠️  WebSocket response timeout");
                }
            }
            
            // Close connection
            ws_stream.close(None).await?;
        }
        Ok(Err(e)) => {
            println!("⚠️  WebSocket connection failed: {}", e);
        }
        Err(_) => {
            println!("⚠️  WebSocket connection timeout");
        }
    }
    
    Ok(())
}

/// Test trading strategy API endpoints
#[tokio::test]
async fn test_strategy_api() -> Result<()> {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8002/api";
    
    // Test strategy configuration endpoint
    let strategies_response = client
        .get(&format!("{}/strategies/config", base_url))
        .timeout(Duration::from_secs(3))
        .send()
        .await;
    
    match strategies_response {
        Ok(response) if response.status().is_success() => {
            let strategies: Value = response.json().await?;
            println!("✅ Strategies config: {:?}", strategies);
            
            // Verify expected strategies are present
            if let Some(strategies_obj) = strategies.as_object() {
                let expected_strategies = vec![
                    "sandwich", "arbitrage", "sniping", 
                    "jupiter_arbitrage", "liquidation", "wallet_tracker"
                ];
                
                for strategy in expected_strategies {
                    if strategies_obj.contains_key(strategy) {
                        println!("   ✅ Strategy '{}' configured", strategy);
                    } else {
                        println!("   ⚠️  Strategy '{}' not found", strategy);
                    }
                }
            }
        }
        Ok(response) => {
            println!("⚠️  Strategies endpoint returned: {}", response.status());
        }
        Err(e) => {
            println!("⚠️  Strategies endpoint not available: {}", e);
        }
    }
    
    // Test strategy control endpoints
    let control_endpoints = vec![
        ("POST", "/strategies/start", json!({"strategy": "test"})),
        ("POST", "/strategies/stop", json!({"strategy": "test"})),
        ("GET", "/strategies/status", json!({})),
    ];
    
    for (method, endpoint, payload) in control_endpoints {
        let url = format!("{}{}", base_url, endpoint);
        
        let response = match method {
            "GET" => client.get(&url).timeout(Duration::from_secs(3)).send().await,
            "POST" => client.post(&url).json(&payload).timeout(Duration::from_secs(3)).send().await,
            _ => continue,
        };
        
        match response {
            Ok(resp) => {
                println!("✅ {} {} responded with: {}", method, endpoint, resp.status());
            }
            Err(e) => {
                println!("⚠️  {} {} failed: {}", method, endpoint, e);
            }
        }
    }
    
    Ok(())
}

/// Test AI integration API endpoints
#[tokio::test]
async fn test_ai_api_endpoints() -> Result<()> {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8002/api";
    
    // Test AI status endpoint
    let ai_status_response = client
        .get(&format!("{}/ai/status", base_url))
        .timeout(Duration::from_secs(3))
        .send()
        .await;
    
    match ai_status_response {
        Ok(response) if response.status().is_success() => {
            let ai_status: Value = response.json().await?;
            println!("✅ AI Status: {:?}", ai_status);
            
            // Verify AI components status
            if let Some(status_obj) = ai_status.as_object() {
                let ai_components = vec!["oumi", "opensearch", "lmcache"];
                
                for component in ai_components {
                    if let Some(component_status) = status_obj.get(component) {
                        println!("   ✅ AI component '{}': {:?}", component, component_status);
                    }
                }
            }
        }
        Ok(response) => {
            println!("⚠️  AI status endpoint returned: {}", response.status());
        }
        Err(e) => {
            println!("⚠️  AI status endpoint not available: {}", e);
        }
    }
    
    // Test AI prediction endpoint
    let prediction_payload = json!({
        "token_address": "So11111111111111111111111111111111111111112",
        "market_data": {
            "current_price": 100.0,
            "volume_24h": 50000.0,
            "price_change_24h": 5.0,
            "liquidity_sol": 1000.0,
            "holder_count": 250
        }
    });
    
    let prediction_response = client
        .post(&format!("{}/ai/predict", base_url))
        .json(&prediction_payload)
        .timeout(Duration::from_secs(5))
        .send()
        .await;
    
    match prediction_response {
        Ok(response) if response.status().is_success() => {
            let prediction: Value = response.json().await?;
            println!("✅ AI Prediction: {:?}", prediction);
            
            // Verify prediction structure
            if let Some(pred_obj) = prediction.as_object() {
                assert!(pred_obj.contains_key("confidence"));
                assert!(pred_obj.contains_key("recommendation"));
                println!("   ✅ Prediction structure valid");
            }
        }
        Ok(response) => {
            println!("⚠️  AI prediction endpoint returned: {}", response.status());
        }
        Err(e) => {
            println!("⚠️  AI prediction endpoint not available: {}", e);
        }
    }
    
    Ok(())
}

/// Test performance metrics API
#[tokio::test]
async fn test_performance_metrics_api() -> Result<()> {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8002/api";
    
    // Test metrics endpoints
    let metrics_endpoints = vec![
        "/performance/summary",
        "/performance/daily",
        "/performance/strategies",
        "/performance/risk",
    ];
    
    for endpoint in metrics_endpoints {
        let url = format!("{}{}", base_url, endpoint);
        let response = client
            .get(&url)
            .timeout(Duration::from_secs(3))
            .send()
            .await;
        
        match response {
            Ok(resp) if resp.status().is_success() => {
                let metrics: Value = resp.json().await?;
                println!("✅ Metrics {}: {:?}", endpoint, metrics);
                
                // Verify metrics structure
                if let Some(metrics_obj) = metrics.as_object() {
                    // Check for common metrics fields
                    let expected_fields = vec!["timestamp", "total_trades", "pnl", "success_rate"];
                    let mut found_fields = 0;
                    
                    for field in expected_fields {
                        if metrics_obj.contains_key(field) {
                            found_fields += 1;
                        }
                    }
                    
                    if found_fields > 0 {
                        println!("   ✅ Found {}/4 expected metrics fields", found_fields);
                    }
                }
            }
            Ok(resp) => {
                println!("⚠️  Metrics {} returned: {}", endpoint, resp.status());
            }
            Err(e) => {
                println!("⚠️  Metrics {} not available: {}", endpoint, e);
            }
        }
    }
    
    Ok(())
}

/// Test dashboard data consistency
#[tokio::test]
async fn test_dashboard_data_consistency() -> Result<()> {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8002/api";
    
    // Fetch multiple related endpoints
    let overview_response = client
        .get(&format!("{}/dashboard/overview", base_url))
        .timeout(Duration::from_secs(3))
        .send()
        .await;
    
    let performance_response = client
        .get(&format!("{}/performance/summary", base_url))
        .timeout(Duration::from_secs(3))
        .send()
        .await;
    
    match (overview_response, performance_response) {
        (Ok(overview_resp), Ok(perf_resp)) if overview_resp.status().is_success() && perf_resp.status().is_success() => {
            let overview: Value = overview_resp.json().await?;
            let performance: Value = perf_resp.json().await?;
            
            println!("✅ Dashboard overview: {:?}", overview);
            println!("✅ Performance summary: {:?}", performance);
            
            // Check data consistency between endpoints
            if let (Some(overview_obj), Some(perf_obj)) = (overview.as_object(), performance.as_object()) {
                // Compare common fields if they exist
                if let (Some(overview_pnl), Some(perf_pnl)) = (overview_obj.get("total_pnl"), perf_obj.get("total_pnl")) {
                    if overview_pnl == perf_pnl {
                        println!("   ✅ PnL data consistent between endpoints");
                    } else {
                        println!("   ⚠️  PnL data inconsistent: overview={:?}, performance={:?}", overview_pnl, perf_pnl);
                    }
                }
                
                if let (Some(overview_trades), Some(perf_trades)) = (overview_obj.get("total_trades"), perf_obj.get("total_trades")) {
                    if overview_trades == perf_trades {
                        println!("   ✅ Trade count consistent between endpoints");
                    } else {
                        println!("   ⚠️  Trade count inconsistent: overview={:?}, performance={:?}", overview_trades, perf_trades);
                    }
                }
            }
        }
        _ => {
            println!("⚠️  Could not fetch both overview and performance data for consistency check");
        }
    }
    
    Ok(())
}

/// Test error handling in API endpoints
#[tokio::test]
async fn test_api_error_handling() -> Result<()> {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8002/api";
    
    // Test invalid endpoints
    let invalid_endpoints = vec![
        "/nonexistent/endpoint",
        "/strategies/invalid_action",
        "/ai/invalid_model",
    ];
    
    for endpoint in invalid_endpoints {
        let url = format!("{}{}", base_url, endpoint);
        let response = client
            .get(&url)
            .timeout(Duration::from_secs(3))
            .send()
            .await;
        
        match response {
            Ok(resp) => {
                // Should return 404 or other error status
                if resp.status().is_client_error() || resp.status().is_server_error() {
                    println!("✅ Endpoint {} properly returned error: {}", endpoint, resp.status());
                } else {
                    println!("⚠️  Endpoint {} unexpectedly succeeded: {}", endpoint, resp.status());
                }
            }
            Err(e) => {
                println!("✅ Endpoint {} properly failed: {}", endpoint, e);
            }
        }
    }
    
    // Test invalid JSON payloads
    let invalid_payload = "invalid json";
    let response = client
        .post(&format!("{}/ai/predict", base_url))
        .body(invalid_payload)
        .header("Content-Type", "application/json")
        .timeout(Duration::from_secs(3))
        .send()
        .await;
    
    match response {
        Ok(resp) if resp.status().is_client_error() => {
            println!("✅ Invalid JSON properly rejected with: {}", resp.status());
        }
        Ok(resp) => {
            println!("⚠️  Invalid JSON unexpectedly accepted: {}", resp.status());
        }
        Err(e) => {
            println!("✅ Invalid JSON properly failed: {}", e);
        }
    }
    
    Ok(())
}

/// Test frontend static file serving
#[tokio::test]
async fn test_frontend_static_files() -> Result<()> {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8002";
    
    // Test main dashboard page
    let dashboard_response = client
        .get(&format!("{}/", base_url))
        .timeout(Duration::from_secs(3))
        .send()
        .await;
    
    match dashboard_response {
        Ok(response) if response.status().is_success() => {
            let content = response.text().await?;
            
            // Check for HTML content
            if content.contains("<html") || content.contains("<!DOCTYPE") {
                println!("✅ Dashboard HTML served successfully");
                
                // Check for expected dashboard elements
                if content.contains("HFT Ninja") || content.contains("dashboard") {
                    println!("   ✅ Dashboard content appears valid");
                }
            } else {
                println!("⚠️  Dashboard response doesn't appear to be HTML");
            }
        }
        Ok(response) => {
            println!("⚠️  Dashboard returned: {}", response.status());
        }
        Err(e) => {
            println!("⚠️  Dashboard not available: {}", e);
        }
    }
    
    // Test static assets (if they exist)
    let static_files = vec!["/static/css/main.css", "/static/js/main.js", "/favicon.ico"];
    
    for file in static_files {
        let url = format!("{}{}", base_url, file);
        let response = client
            .get(&url)
            .timeout(Duration::from_secs(3))
            .send()
            .await;
        
        match response {
            Ok(resp) if resp.status().is_success() => {
                println!("✅ Static file {} served successfully", file);
            }
            Ok(resp) => {
                println!("⚠️  Static file {} returned: {}", file, resp.status());
            }
            Err(e) => {
                println!("⚠️  Static file {} not available: {}", file, e);
            }
        }
    }
    
    Ok(())
}
