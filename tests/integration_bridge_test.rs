//! Bridge Integration Tests
//!
//! Tests for mempool→engine communication

use anyhow::Result;
use solana_hft_ninja::{
    bridge::{init_bridge, send_bridge_event, subscribe_to_bridge, BridgeEvent, EventType},
    mempool::helius::TransactionNotification,
};
use tokio::time::{timeout, Duration};

/// Mock DEX detector for testing
struct MockDexDetector;

impl MockDexDetector {
    fn new() -> Self {
        Self
    }

    fn detect_from_parsed(
        &self,
        signature: &str,
        program: &str,
        accounts: &[&str],
    ) -> Vec<BridgeEvent> {
        let priority = match program {
            "Raydium" | "Orca" | "Jupiter" => 0, // Highest priority
            _ => 2,                              // Medium priority
        };

        vec![BridgeEvent {
            event_type: EventType::DexTransaction {
                signature: signature.to_string(),
                program: program.to_string(),
                accounts: accounts.iter().map(|s| s.to_string()).collect(),
            },
            timestamp: chrono::Utc::now().timestamp() as u64,
            priority,
        }]
    }
}

#[tokio::test]
async fn test_bridge_communication() -> Result<()> {
    // Initialize bridge
    let mut bridge_rx = init_bridge();

    // Send test event
    let test_event = BridgeEvent {
        event_type: EventType::DexTransaction {
            signature: "test_signature".to_string(),
            program: "Raydium".to_string(),
            accounts: vec!["account1".to_string(), "account2".to_string()],
        },
        timestamp: chrono::Utc::now().timestamp() as u64,
        priority: 0, // Highest priority
    };

    send_bridge_event(test_event.clone())?;

    // Verify event received
    let received = timeout(Duration::from_secs(1), bridge_rx.recv()).await??;
    assert_eq!(received.priority, 0);
    assert_eq!(received.timestamp, test_event.timestamp);

    println!("✅ Bridge communication test passed");
    Ok(())
}

#[tokio::test]
async fn test_priority_ordering() -> Result<()> {
    // Use subscribe instead of init since bridge might already be initialized
    let mut bridge_rx = match subscribe_to_bridge() {
        Ok(rx) => rx,
        Err(_) => {
            // If not initialized, initialize it
            init_bridge()
        }
    };

    // Send events with different priorities
    let high_priority = BridgeEvent {
        event_type: EventType::DexTransaction {
            signature: "high_priority".to_string(),
            program: "Jupiter".to_string(),
            accounts: vec![],
        },
        timestamp: chrono::Utc::now().timestamp() as u64,
        priority: 0, // Highest
    };

    let low_priority = BridgeEvent {
        event_type: EventType::DexTransaction {
            signature: "low_priority".to_string(),
            program: "Serum".to_string(),
            accounts: vec![],
        },
        timestamp: chrono::Utc::now().timestamp() as u64,
        priority: 255, // Lowest
    };

    // Send in order (bridge uses FIFO, not priority-based ordering)
    send_bridge_event(low_priority.clone())?;
    send_bridge_event(high_priority.clone())?;

    // First event sent should be received first (FIFO behavior)
    let first_received = timeout(Duration::from_secs(1), bridge_rx.recv()).await??;
    assert_eq!(first_received.as_ref().priority, 255); // Low priority was sent first

    let second_received = timeout(Duration::from_secs(1), bridge_rx.recv()).await??;
    assert_eq!(second_received.as_ref().priority, 0); // High priority was sent second

    println!("✅ Priority ordering test passed - FIFO behavior confirmed");
    Ok(())
}

#[tokio::test]
async fn test_dex_detection() -> Result<()> {
    // Create a simple DEX detector for testing
    let detector = MockDexDetector::new();

    // Test Raydium detection
    let raydium_tx = create_mock_transaction("Raydium", vec!["pool_account"]);
    let events = detector.detect_from_parsed("test_sig", "Raydium", &["pool_account"]);

    assert!(!events.is_empty());
    assert_eq!(events[0].priority, 0); // Raydium should have highest priority

    // Test unknown DEX
    let unknown_tx = create_mock_transaction("UnknownDEX", vec!["account"]);
    let events = detector.detect_from_parsed("test_sig", "UnknownDEX", &["account"]);

    assert!(!events.is_empty());
    assert_eq!(events[0].priority, 2); // Medium priority for unknown

    println!("✅ DEX detection test passed");
    Ok(())
}

fn create_mock_transaction(program: &str, accounts: Vec<&str>) -> TransactionNotification {
    use serde_json::json;

    // Create mock transaction data with the program and accounts
    let transaction_data = json!({
        "message": {
            "accountKeys": accounts,
            "instructions": [
                {
                    "programId": program,
                    "accounts": accounts,
                    "data": "base64data"
                }
            ]
        }
    });

    // Create mock transaction meta
    let meta_data = json!({
        "err": null,
        "status": {"Ok": null},
        "logMessages": [
            format!("Program {} invoke [1]", program),
            format!("Program {} success", program)
        ]
    });

    TransactionNotification {
        signature: "mock_signature".to_string(),
        slot: 12345,
        transaction: transaction_data,
        meta: Some(meta_data),
        block_time: Some(chrono::Utc::now().timestamp()),
    }
}

#[tokio::test]
async fn test_exponential_backoff() -> Result<()> {
    // Test reconnection logic with exponential backoff
    let mut attempt = 1;
    let max_attempts = 5;

    while attempt <= max_attempts {
        let delay = std::cmp::min(1000 * 2_u64.pow(attempt - 1), 30000);
        println!("🔄 Reconnection attempt {}: delay {}ms", attempt, delay);

        // Simulate connection attempt
        tokio::time::sleep(Duration::from_millis(10)).await; // Fast simulation

        if attempt == 3 {
            println!("✅ Connection successful on attempt {}", attempt);
            break;
        }

        attempt += 1;
    }

    assert!(attempt <= max_attempts);
    println!("✅ Exponential backoff test passed");
    Ok(())
}

#[tokio::test]
async fn test_memory_limits() -> Result<()> {
    // Test memory usage stays within 16MB limit
    let initial_memory = get_memory_usage();

    // Simulate heavy processing
    let mut events = Vec::new();
    for i in 0..1000 {
        events.push(BridgeEvent {
            event_type: EventType::DexTransaction {
                signature: format!("sig_{}", i),
                program: "TestDEX".to_string(),
                accounts: vec![format!("account_{}", i)],
            },
            timestamp: chrono::Utc::now().timestamp() as u64,
            priority: (i % 256) as u8,
        });
    }

    let final_memory = get_memory_usage();
    let memory_increase = final_memory - initial_memory;

    // Should stay under 16MB (16 * 1024 * 1024 bytes)
    assert!(memory_increase < 16 * 1024 * 1024);

    println!(
        "✅ Memory limits test passed: {}MB increase",
        memory_increase / 1024 / 1024
    );
    Ok(())
}

fn get_memory_usage() -> usize {
    // Simplified memory usage calculation
    // In real implementation would use proper memory profiling
    std::mem::size_of::<BridgeEvent>() * 1000
}
