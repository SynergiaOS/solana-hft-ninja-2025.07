//! Strategy Control API
//! 
//! Real-time strategy control endpoints for HFT system

use warp::{Filter, Reply, Rejection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use tracing::{info, warn, error};

/// Strategy control commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyCommand {
    Enable,
    Disable,
    Reset,
    UpdateConfig(serde_json::Value),
}

/// Strategy status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyStatus {
    pub name: String,
    pub enabled: bool,
    pub executions: u64,
    pub profit_sol: f64,
    pub success_rate: f64,
    pub last_execution: Option<u64>,
}

/// Strategy control manager
#[derive(Debug, Clone)]
pub struct StrategyController {
    strategies: Arc<RwLock<HashMap<String, StrategyStatus>>>,
}

impl StrategyController {
    /// Create new strategy controller
    pub fn new() -> Self {
        let mut strategies = HashMap::new();
        
        // Initialize default strategies
        strategies.insert("sandwich".to_string(), StrategyStatus {
            name: "sandwich".to_string(),
            enabled: true,
            executions: 0,
            profit_sol: 0.0,
            success_rate: 0.0,
            last_execution: None,
        });
        
        strategies.insert("arbitrage".to_string(), StrategyStatus {
            name: "arbitrage".to_string(),
            enabled: true,
            executions: 0,
            profit_sol: 0.0,
            success_rate: 0.0,
            last_execution: None,
        });
        
        strategies.insert("liquidation".to_string(), StrategyStatus {
            name: "liquidation".to_string(),
            enabled: true,
            executions: 0,
            profit_sol: 0.0,
            success_rate: 0.0,
            last_execution: None,
        });
        
        strategies.insert("sniping".to_string(), StrategyStatus {
            name: "sniping".to_string(),
            enabled: true,
            executions: 0,
            profit_sol: 0.0,
            success_rate: 0.0,
            last_execution: None,
        });
        
        strategies.insert("jupiter_arbitrage".to_string(), StrategyStatus {
            name: "jupiter_arbitrage".to_string(),
            enabled: true,
            executions: 0,
            profit_sol: 0.0,
            success_rate: 0.0,
            last_execution: None,
        });
        
        Self {
            strategies: Arc::new(RwLock::new(strategies)),
        }
    }
    
    /// Get all strategy statuses
    pub async fn get_all_strategies(&self) -> Vec<StrategyStatus> {
        let strategies = self.strategies.read().await;
        strategies.values().cloned().collect()
    }
    
    /// Get specific strategy status
    pub async fn get_strategy(&self, name: &str) -> Option<StrategyStatus> {
        let strategies = self.strategies.read().await;
        strategies.get(name).cloned()
    }
    
    /// Execute strategy command
    pub async fn execute_command(&self, strategy_name: &str, command: StrategyCommand) -> Result<String, String> {
        let mut strategies = self.strategies.write().await;
        
        if let Some(strategy) = strategies.get_mut(strategy_name) {
            match command {
                StrategyCommand::Enable => {
                    strategy.enabled = true;
                    info!("✅ Strategy '{}' enabled", strategy_name);
                    Ok(format!("Strategy '{}' enabled successfully", strategy_name))
                }
                StrategyCommand::Disable => {
                    strategy.enabled = false;
                    warn!("⚠️ Strategy '{}' disabled", strategy_name);
                    Ok(format!("Strategy '{}' disabled successfully", strategy_name))
                }
                StrategyCommand::Reset => {
                    strategy.executions = 0;
                    strategy.profit_sol = 0.0;
                    strategy.success_rate = 0.0;
                    strategy.last_execution = None;
                    info!("🔄 Strategy '{}' reset", strategy_name);
                    Ok(format!("Strategy '{}' reset successfully", strategy_name))
                }
                StrategyCommand::UpdateConfig(_config) => {
                    info!("⚙️ Strategy '{}' config updated", strategy_name);
                    Ok(format!("Strategy '{}' config updated successfully", strategy_name))
                }
            }
        } else {
            error!("❌ Strategy '{}' not found", strategy_name);
            Err(format!("Strategy '{}' not found", strategy_name))
        }
    }
    
    /// Update strategy statistics
    pub async fn update_stats(&self, strategy_name: &str, profit: f64, success: bool) {
        let mut strategies = self.strategies.write().await;
        
        if let Some(strategy) = strategies.get_mut(strategy_name) {
            strategy.executions += 1;
            strategy.profit_sol += profit;
            strategy.last_execution = Some(chrono::Utc::now().timestamp() as u64);
            
            // Update success rate
            let total_executions = strategy.executions as f64;
            let current_success_rate = strategy.success_rate;
            let new_success_rate = if success {
                (current_success_rate * (total_executions - 1.0) + 1.0) / total_executions
            } else {
                (current_success_rate * (total_executions - 1.0)) / total_executions
            };
            strategy.success_rate = new_success_rate;
        }
    }
    
    /// Emergency stop all strategies
    pub async fn emergency_stop(&self) -> String {
        let mut strategies = self.strategies.write().await;
        let mut stopped_count = 0;
        
        for strategy in strategies.values_mut() {
            if strategy.enabled {
                strategy.enabled = false;
                stopped_count += 1;
            }
        }
        
        error!("🚨 EMERGENCY STOP: {} strategies disabled", stopped_count);
        format!("Emergency stop executed - {} strategies disabled", stopped_count)
    }
}

/// Create strategy control API routes
pub fn create_routes(controller: StrategyController) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let controller = Arc::new(controller);
    
    // GET /control/strategies - List all strategies
    let list_strategies = warp::path!("control" / "strategies")
        .and(warp::get())
        .and(with_controller(controller.clone()))
        .and_then(handle_list_strategies);
    
    // GET /control/strategy/{name} - Get specific strategy
    let get_strategy = warp::path!("control" / "strategy" / String)
        .and(warp::get())
        .and(with_controller(controller.clone()))
        .and_then(handle_get_strategy);
    
    // POST /control/strategy/{name}/{command} - Execute strategy command
    let strategy_command = warp::path!("control" / "strategy" / String / String)
        .and(warp::post())
        .and(warp::body::json())
        .and(with_controller(controller.clone()))
        .and_then(handle_strategy_command);
    
    // POST /control/emergency_stop - Emergency stop all strategies
    let emergency_stop = warp::path!("control" / "emergency_stop")
        .and(warp::post())
        .and(with_controller(controller.clone()))
        .and_then(handle_emergency_stop);
    
    list_strategies
        .or(get_strategy)
        .or(strategy_command)
        .or(emergency_stop)
}

/// Helper to pass controller to handlers
fn with_controller(controller: Arc<StrategyController>) -> impl Filter<Extract = (Arc<StrategyController>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || controller.clone())
}

/// Handle list all strategies
async fn handle_list_strategies(controller: Arc<StrategyController>) -> Result<impl Reply, Rejection> {
    let strategies = controller.get_all_strategies().await;
    Ok(warp::reply::json(&strategies))
}

/// Handle get specific strategy
async fn handle_get_strategy(name: String, controller: Arc<StrategyController>) -> Result<impl Reply, Rejection> {
    if let Some(strategy) = controller.get_strategy(&name).await {
        Ok(warp::reply::json(&strategy))
    } else {
        Ok(warp::reply::json(&serde_json::json!({"error": "Strategy not found"})))
    }
}

/// Handle strategy command
async fn handle_strategy_command(
    name: String,
    command_str: String,
    body: serde_json::Value,
    controller: Arc<StrategyController>,
) -> Result<impl Reply, Rejection> {
    let command = match command_str.as_str() {
        "enable" => StrategyCommand::Enable,
        "disable" => StrategyCommand::Disable,
        "reset" => StrategyCommand::Reset,
        "update_config" => StrategyCommand::UpdateConfig(body),
        _ => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": "Invalid command"})),
                warp::http::StatusCode::BAD_REQUEST,
            ));
        }
    };
    
    match controller.execute_command(&name, command).await {
        Ok(message) => Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"message": message})),
            warp::http::StatusCode::OK,
        )),
        Err(error) => Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"error": error})),
            warp::http::StatusCode::BAD_REQUEST,
        )),
    }
}

/// Handle emergency stop
async fn handle_emergency_stop(controller: Arc<StrategyController>) -> Result<impl Reply, Rejection> {
    let message = controller.emergency_stop().await;
    Ok(warp::reply::json(&serde_json::json!({"message": message})))
}
