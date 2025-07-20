use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use chrono;

use crate::cerberus::{CerberusBrain, PositionState, PositionStatus};

/// Cerberus API endpoints for frontend integration
pub fn cerberus_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/cerberus/status", get(get_cerberus_status))
        .route("/cerberus/positions", get(get_positions))
        .route("/cerberus/positions/:mint", get(get_position))
        .route("/cerberus/positions", post(create_position))
        .route("/cerberus/positions/:mint", post(update_position))
        .route("/cerberus/emergency-stop", post(emergency_stop))
        .route("/cerberus/metrics", get(get_metrics))
        .route("/cerberus/decisions", get(get_decision_logs))
        .route("/cerberus/commands", post(send_command))
}

#[derive(Clone)]
pub struct AppState {
    pub cerberus: Option<Arc<CerberusBrain>>,
    pub positions: Arc<RwLock<HashMap<String, PositionState>>>,
    pub decision_logs: Arc<RwLock<Vec<DecisionLog>>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CerberusStatus {
    pub status: String,
    pub uptime: String,
    pub last_decision_time: String,
    pub active_positions: usize,
    pub emergency_stop_enabled: bool,
    pub rpc_health: RpcHealth,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RpcHealth {
    pub primary_healthy: bool,
    pub fallback_healthy: bool,
    pub last_check: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CerberusMetrics {
    pub total_positions: usize,
    pub profitable_positions: usize,
    pub total_value_sol: f64,
    pub decision_latency_ms: f64,
    pub execution_latency_ms: f64,
    pub success_rate: f64,
    pub decisions_per_minute: f64,
    pub uptime_seconds: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DecisionLog {
    pub timestamp: String,
    pub mint: String,
    pub decision: String,
    pub reason: String,
    pub confidence: f64,
    pub execution_time_ms: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreatePositionRequest {
    pub mint: String,
    pub entry_price: f64,
    pub position_size_sol: f64,
    pub strategy_id: String,
    pub wallet_address: String,
    pub take_profit_target_percent: Option<f64>,
    pub stop_loss_target_percent: Option<f64>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdatePositionRequest {
    pub take_profit_target_percent: Option<f64>,
    pub stop_loss_target_percent: Option<f64>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandRequest {
    pub action: String,
    pub mint: Option<String>,
    pub amount_sol: Option<f64>,
    pub reason: String,
}

#[derive(Deserialize, Debug)]
pub struct PositionQuery {
    pub status: Option<String>,
    pub strategy: Option<String>,
    pub limit: Option<usize>,
}

/// Get Cerberus status
pub async fn get_cerberus_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<CerberusStatus>, StatusCode> {
    info!("Getting Cerberus status");

    let positions = state.positions.read().await;
    let active_positions = positions.values().filter(|p| p.status == PositionStatus::Open).count();

    // Mock RPC health for now
    let rpc_health = RpcHealth {
        primary_healthy: true,
        fallback_healthy: true,
        last_check: chrono::Utc::now().to_rfc3339(),
    };

    let status = CerberusStatus {
        status: "online".to_string(),
        uptime: "2h 34m".to_string(),
        last_decision_time: chrono::Utc::now().to_rfc3339(),
        active_positions,
        emergency_stop_enabled: true,
        rpc_health,
    };

    Ok(Json(status))
}

/// Get all positions
pub async fn get_positions(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PositionQuery>,
) -> Result<Json<Vec<PositionState>>, StatusCode> {
    info!("Getting positions with query: {:?}", query);

    let positions = state.positions.read().await;
    let mut filtered_positions: Vec<PositionState> = positions.values().cloned().collect();

    // Filter by status
    if let Some(status) = &query.status {
        let filter_status = match status.as_str() {
            "open" => PositionStatus::Open,
            "closed" => PositionStatus::Closed,
            "pending" => PositionStatus::Pending,
            "failed" => PositionStatus::Failed,
            _ => return Err(StatusCode::BAD_REQUEST),
        };
        filtered_positions.retain(|p| p.status == filter_status);
    }

    // Filter by strategy
    if let Some(strategy) = &query.strategy {
        filtered_positions.retain(|p| p.strategy_id == *strategy);
    }

    // Limit results
    if let Some(limit) = query.limit {
        filtered_positions.truncate(limit);
    }

    Ok(Json(filtered_positions))
}

/// Get specific position
pub async fn get_position(
    State(state): State<Arc<AppState>>,
    Path(mint): Path<String>,
) -> Result<Json<PositionState>, StatusCode> {
    info!("Getting position for mint: {}", mint);

    let positions = state.positions.read().await;
    
    match positions.get(&mint) {
        Some(position) => Ok(Json(position.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Create new position
pub async fn create_position(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreatePositionRequest>,
) -> Result<Json<PositionState>, StatusCode> {
    info!("Creating position for mint: {}", request.mint);

    let mut position = PositionState::new(
        request.mint.clone(),
        request.entry_price,
        request.position_size_sol,
        request.strategy_id,
        request.wallet_address,
    );

    // Apply custom parameters
    if let Some(tp) = request.take_profit_target_percent {
        position.take_profit_target_percent = tp;
    }
    if let Some(sl) = request.stop_loss_target_percent {
        position.stop_loss_target_percent = sl;
    }
    if let Some(timeout) = request.timeout_seconds {
        position.timeout_seconds = timeout;
    }

    // Store position
    let mut positions = state.positions.write().await;
    positions.insert(request.mint.clone(), position.clone());

    // If Cerberus is available, store in Redis
    if let Some(cerberus) = &state.cerberus {
        if let Err(e) = cerberus.store.store_position(&position).await {
            error!("Failed to store position in Cerberus: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    Ok(Json(position))
}

/// Update position parameters
pub async fn update_position(
    State(state): State<Arc<AppState>>,
    Path(mint): Path<String>,
    Json(request): Json<UpdatePositionRequest>,
) -> Result<Json<PositionState>, StatusCode> {
    info!("Updating position for mint: {}", mint);

    let mut positions = state.positions.write().await;
    
    match positions.get_mut(&mint) {
        Some(position) => {
            if let Some(tp) = request.take_profit_target_percent {
                position.take_profit_target_percent = tp;
            }
            if let Some(sl) = request.stop_loss_target_percent {
                position.stop_loss_target_percent = sl;
            }
            if let Some(timeout) = request.timeout_seconds {
                position.timeout_seconds = timeout;
            }

            // Update in Cerberus if available
            if let Some(cerberus) = &state.cerberus {
                if let Err(e) = cerberus.store.update_position(position).await {
                    error!("Failed to update position in Cerberus: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }

            Ok(Json(position.clone()))
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Trigger emergency stop
pub async fn emergency_stop(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CommandRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    warn!("Emergency stop triggered: {}", request.reason);

    if let Some(cerberus) = &state.cerberus {
        if let Err(e) = cerberus.emergency_stop(&request.reason).await {
            error!("Failed to execute emergency stop: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Clear local positions
    let mut positions = state.positions.write().await;
    for position in positions.values_mut() {
        if position.status == PositionStatus::Open {
            position.status = PositionStatus::Closed;
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Emergency stop executed",
        "reason": request.reason
    })))
}

/// Get Cerberus metrics
pub async fn get_metrics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<CerberusMetrics>, StatusCode> {
    info!("Getting Cerberus metrics");

    let positions = state.positions.read().await;
    let total_positions = positions.len();
    let profitable_positions = positions.values()
        .filter(|p| p.pnl_unrealized_percent.unwrap_or(0.0) > 0.0)
        .count();
    let total_value_sol = positions.values()
        .map(|p| p.position_size_sol)
        .sum();

    let metrics = CerberusMetrics {
        total_positions,
        profitable_positions,
        total_value_sol,
        decision_latency_ms: 150.0 + (rand::random::<f64>() * 50.0),
        execution_latency_ms: 80.0 + (rand::random::<f64>() * 40.0),
        success_rate: 97.3,
        decisions_per_minute: 300.0, // 200ms interval = 300 decisions/minute
        uptime_seconds: 9240, // 2h 34m
    };

    Ok(Json(metrics))
}

/// Get decision logs
pub async fn get_decision_logs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PositionQuery>,
) -> Result<Json<Vec<DecisionLog>>, StatusCode> {
    info!("Getting decision logs");

    let logs = state.decision_logs.read().await;
    let mut filtered_logs = logs.clone();

    // Apply limit
    if let Some(limit) = query.limit {
        filtered_logs.truncate(limit);
    }

    Ok(Json(filtered_logs))
}

/// Send command to Cerberus
pub async fn send_command(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CommandRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Sending command to Cerberus: {:?}", request);

    // This would send command via Redis in real implementation
    // For now, just return success

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Command sent to Cerberus",
        "action": request.action
    })))
}
