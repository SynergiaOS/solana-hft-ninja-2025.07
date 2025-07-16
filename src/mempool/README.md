# Mempool Listener Module

A high-performance mempool listener for Solana HFT systems using Helius WebSocket API with zero-copy deserialization.

## Overview

This module provides real-time transaction monitoring from Solana's mempool with the following features:

- **Zero-copy deserialization** using `bytemuck` for maximum performance
- **Sub-millisecond latency** (<1ms processing time)
- **DEX interaction detection** for Raydium, Orca, and Jupiter
- **Memory-efficient** (16MB per thread limit)
- **WebSocket reconnection** with exponential backoff
- **Comprehensive metrics** collection
- **Error handling** with graceful recovery

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Helius WS     │───▶│  MempoolListener │───▶│  Processing     │
│   API           │    │  (WebSocket)     │    │  Channel        │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌──────────────────┐
                       │  ZeroCopyParser  │
                       │  (Deserialization)│
                       └──────────────────┘
                                │
                                ▼
                       ┌──────────────────┐
                       │  DEX Detection   │
                       │  (Raydium/Orca/  │
                       │   Jupiter)       │
                       └──────────────────┘
```

## Usage

### Basic Setup

```rust
use solana_hft_ninja::mempool::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create channel for receiving parsed transactions
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    
    // Configure Helius connection
    let config = HeliusConfig {
        api_key: std::env::var("HELIUS_KEY")?,
        endpoint: "https://api.helius.xyz".to_string(),
        commitment: CommitmentLevel::Processed,
        max_reconnect_attempts: 10,
        reconnect_delay_ms: 1000,
    };
    
    // Create parser and metrics
    let metrics = MempoolMetrics::new();
    let parser = ZeroCopyParser::new(metrics.clone(), 16 * 1024 * 1024);
    
    // Create and start listener
    let listener = MempoolListener::new(config, parser, metrics, tx);
    
    // Start listening in background
    let listener_handle = tokio::spawn(async move {
        listener.start().await
    });
    
    // Process transactions
    while let Some(parsed_tx) = rx.recv().await {
        println!("Received transaction: {:?}", parsed_tx.signature);
        
        for interaction in &parsed_tx.dex_interactions {
            println!("DEX interaction: {} - {}", 
                interaction.program.name(), 
                interaction.instruction_type
            );
        }
    }
    
    Ok(())
}
```

### Using Builder Pattern

```rust
use solana_hft_ninja::mempool::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    
    let listener = MempoolListenerBuilder::new()
        .with_config(HeliusConfig {
            api_key: std::env::var("HELIUS_KEY")?,
            ..Default::default()
        })
        .with_metrics(MempoolMetrics::new())
        .with_sender(tx)
        .build()?;
    
    // Start listener
    tokio::spawn(async move {
        listener.start().await
    });
    
    Ok(())
}
```

### Custom DEX Detection

```rust
use solana_hft_ninja::mempool::dex::*;

fn custom_dex_filter(interactions: &[DexInteraction]) -> Vec<&DexInteraction> {
    interactions
        .iter()
        .filter(|ix| {
            matches!(ix.program, DexProgram::RaydiumAmm | DexProgram::OrcaWhirlpool) &&
            matches!(ix.instruction_type, InstructionType::Swap)
        })
        .collect()
}
```

## Configuration

### Environment Variables

- `HELIUS_KEY`: Your Helius API key (required)
- `RUST_LOG`: Logging level (optional, default: info)

### HeliusConfig Options

| Option | Description | Default |
|--------|-------------|---------|
| `api_key` | Helius API key | From env |
| `endpoint` | Helius endpoint | `https://api.helius.xyz` |
| `commitment` | Commitment level | `Processed` |
| `max_reconnect_attempts` | Max reconnect tries | `10` |
| `reconnect_delay_ms` | Reconnect delay | `1000` |

## Performance Characteristics

### Benchmarks

- **Processing Latency**: <1ms per transaction
- **Memory Usage**: <16MB per thread
- **CPU Usage**: <5% per core
- **Throughput**: >10,000 TPS

### Memory Management

- Zero-copy deserialization using `bytemuck`
- Fixed-size transaction buffer
- Automatic memory cleanup on disconnect
- Configurable memory limits

## Error Handling

The module implements comprehensive error handling:

- **WebSocket errors**: Automatic reconnection with exponential backoff
- **Deserialization errors**: Graceful handling with metrics tracking
- **Memory limits**: Configurable limits with overflow protection
- **Rate limiting**: Backpressure handling

## Metrics

Available metrics via Prometheus:

- `mempool_transactions_total`: Total transactions processed
- `mempool_bytes_received_total`: Total bytes received
- `mempool_connection_attempts_total`: Connection attempts
- `mempool_connection_failures_total`: Connection failures
- `mempool_deserialization_errors_total`: Deserialization errors
- `mempool_dex_detections_total`: DEX interactions detected
- `mempool_memory_usage_bytes`: Current memory usage
- `mempool_latency_ms`: Average processing latency
- `mempool_processing_duration`: Transaction processing duration histogram

## Testing

### Unit Tests

```bash
cargo test --package solana-hft-ninja --lib mempool::tests
```

### Integration Tests

```bash
cargo test --test mempool_integration
```

### Performance Tests

```bash
cargo test --release --test mempool_integration -- --nocapture
```

## Supported DEX Programs

| DEX | Program ID | Features |
|-----|------------|----------|
| **Raydium AMM V4** | `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8` | Swaps, Liquidity |
| **Raydium CLMM** | `CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK` | Concentrated Liquidity |
| **Orca Whirlpool** | `whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc` | AMM Swaps |
| **Orca Aquafarm** | `82yxjeMsvaURa4MbZZ7WZZHfobirZYkH1zF8fmeGtyaQ` | Yield Farming |
| **Jupiter V6** | `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4` | Aggregator |
| **Jupiter Limit Order** | `j1o2qRpjcyUwEvwtcfhEQefh773ZgjxcVRry7LDqg5X` | Limit Orders |
| **Jupiter DCA** | `DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M` | Dollar Cost Averaging |

## Troubleshooting

### Common Issues

1. **WebSocket Connection Failed**
   - Check API key validity
   - Verify network connectivity
   - Check Helius service status

2. **High Memory Usage**
   - Reduce transaction buffer size
   - Check for memory leaks in processing
   - Monitor metrics for patterns

3. **Missing DEX Interactions**
   - Verify program IDs are up-to-date
   - Check instruction parsing logic
   - Review transaction data format

### Debug Logging

Enable debug logging:
```bash
RUST_LOG=debug cargo run
```

### Metrics Endpoint

Access Prometheus metrics:
```bash
curl http://localhost:9090/metrics
```

## API Reference

### MempoolListener

Main listener struct for WebSocket connections.

#### Methods

- `start()`: Start listening for transactions
- `stop()`: Stop the listener
- `is_running()`: Check if listener is active
- `metrics()`: Get current metrics

### ZeroCopyParser

High-performance transaction parser.

#### Methods

- `parse_transaction()`: Parse transaction data
- `has_dex_interactions()`: Check for DEX interactions
- `get_transaction_size()`: Get transaction size

### MempoolMetrics

Metrics collection and monitoring.

#### Methods

- `get_stats()`: Get current statistics
- `processing_timer()`: Create processing timer
- Various increment methods for counters

## License

MIT License - See LICENSE file for details.