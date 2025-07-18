//! API Module
//! 
//! REST API endpoints for HFT system control and monitoring

// 🥷 API Layer - High-Performance REST & WebSocket API
// Unified API for trading engine control and monitoring

pub mod strategy_control;
pub mod rest;
pub mod websocket;
pub mod types;

use crate::core::Engine;
use anyhow::Result;
use std::sync::Arc;

/// Start the API server
pub async fn start_server(engine: Arc<Engine>, port: u16) -> Result<()> {
    tracing::info!("🌐 Starting API server on port {}", port);

    // Placeholder implementation - start simple HTTP server
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("API server would start on {}", addr);

    // For now, just wait indefinitely
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

pub use strategy_control::*;
