//! Example usage of the mempool listener module

use solana_hft_ninja::mempool::*;
use std::env;
use tokio::sync::mpsc;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting mempool listener example...");

    // Get Helius API key from environment
    let api_key = env::var("HELIUS_KEY")
        .expect("HELIUS_KEY environment variable must be set");

    // Create channel for receiving parsed transactions
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Configure Helius connection
    let config = HeliusConfig {
        api_key,
        endpoint: "https://api.helius.xyz".to_string(),
        commitment: CommitmentLevel::Processed,
        max_reconnect_attempts: 10,
        reconnect_delay_ms: 1000,
    };

    // Create metrics and parser
    let metrics = MempoolMetrics::new();
    let parser = ZeroCopyParser::new(metrics.clone(), 16 * 1024 * 1024);

    // Create listener
    let listener = MempoolListener::new(config, parser, metrics.clone(), tx);

    // Start listener in background
    let listener_handle = tokio::spawn(async move {
        if let Err(e) = listener.start().await {
            error!("Listener error: {}", e);
        }
    });

    // Process transactions
    let mut processed_count = 0;
    let mut dex_count = 0;

    while let Some(parsed_tx) = rx.recv().await {
        processed_count += 1;

        // Print transaction summary
        info!(
            "Transaction {}: slot={}, timestamp={}, dex_interactions={}",
            bs58::encode(&parsed_tx.signature).into_string(),
            parsed_tx.slot,
            parsed_tx.timestamp,
            parsed_tx.dex_interactions.len()
        );

        // Process DEX interactions
        for interaction in &parsed_tx.dex_interactions {
            dex_count += 1;

            info!(
                "  DEX: {} - {} ({} accounts)",
                interaction.program.name(),
                interaction.instruction_type,
                interaction.accounts.len()
            );

            // Example: Filter for specific DEX programs
            if matches!(
                interaction.program,
                DexProgram::RaydiumAmm | DexProgram::OrcaWhirlpool
            ) {
                info!("    -> High-priority DEX detected!");
            }
        }

        // Print metrics every 100 transactions
        if processed_count % 100 == 0 {
            let stats = metrics.get_stats();
            info!("=== Metrics ===");
            info!("  Transactions processed: {}", stats.transactions_processed);
            info!("  DEX interactions: {}", stats.dex_detections);
            info!("  Bytes received: {}", stats.bytes_received);
            info!("  Memory usage: {} MB", stats.memory_usage_bytes / 1024 / 1024);
            info!("  Connection attempts: {}", stats.connection_attempts);
            info!("  Connection failures: {}", stats.connection_failures);
            info!("  Deserialization errors: {}", stats.deserialization_errors);
        }
    }

    // Wait for listener to finish
    listener_handle.await?;

    Ok(())
}