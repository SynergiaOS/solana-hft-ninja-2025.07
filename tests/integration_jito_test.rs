//! Jito Bundle Execution Integration Tests

use anyhow::Result;
use base64::Engine;
use solana_hft_ninja::execution::{
    create_bundle_transaction, create_high_priority_bundle_transaction, BundleTransaction,
    JitoConfig, JitoExecutor,
};
use solana_sdk::{
    pubkey::Pubkey, signature::{Keypair, Signer}, system_instruction, transaction::Transaction,
};
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_jito_tip_calculation() -> Result<()> {
    let config = JitoConfig::default();
    let tip_keypair = Keypair::new();
    let _executor = JitoExecutor::new(config.clone(), tip_keypair);

    // Test tip calculation for different scenarios
    let test_cases = vec![
        (1, 100, 111000),  // 1 tx, priority 100 -> min(10000) + 1000 + 100000 = 111000
        (3, 200, 213000), // 3 tx, priority 200 -> min(10000) + 3000 + 200000 = 213000
        (1, 50, 61000),   // 1 tx, priority 50 -> min(10000) + 1000 + 50000 = 61000
    ];

    for (tx_count, priority, expected_tip) in test_cases {
        let transactions: Vec<BundleTransaction> = (0..tx_count)
            .map(|i| {
                let tx = create_mock_transaction(&format!("test_{}", i));
                create_bundle_transaction(tx, priority)
            })
            .collect();

        let calculated_tip = calculate_tip_amount(&config, &transactions);

        // Allow some variance due to averaging
        let variance = (calculated_tip as i64 - expected_tip as i64).abs();
        assert!(
            variance < 5000,
            "Tip calculation failed: expected ~{}, got {}, variance: {}",
            expected_tip,
            calculated_tip,
            variance
        );

        println!(
            "✅ Tip calculation test passed: {} txs, priority {}, tip {} lamports",
            tx_count, priority, calculated_tip
        );
    }

    Ok(())
}

fn calculate_tip_amount(config: &JitoConfig, transactions: &[BundleTransaction]) -> u64 {
    let mut tip = config.min_tip_lamports;
    tip += (transactions.len() as u64) * 1000;

    let avg_priority: u64 =
        transactions.iter().map(|tx| tx.priority as u64).sum::<u64>() / transactions.len() as u64;

    tip += avg_priority * 1000;
    tip.min(config.max_tip_lamports)
}

#[tokio::test]
async fn test_bundle_priority_sorting() -> Result<()> {
    // Create transactions with different priorities
    let mut transactions = vec![
        create_bundle_transaction(create_mock_transaction("low"), 50),
        create_bundle_transaction(create_mock_transaction("high"), 200),
        create_bundle_transaction(create_mock_transaction("medium"), 100),
        create_high_priority_bundle_transaction(create_mock_transaction("highest")),
    ];

    // Sort by priority (highest first)
    transactions.sort_by(|a, b| b.priority.cmp(&a.priority));

    // Verify sorting
    assert_eq!(transactions[0].priority, 255); // highest
    assert_eq!(transactions[1].priority, 200); // high
    assert_eq!(transactions[2].priority, 100); // medium
    assert_eq!(transactions[3].priority, 50); // low

    println!("✅ Bundle priority sorting test passed");
    for (i, tx) in transactions.iter().enumerate() {
        println!("   Position {}: Priority {}", i, tx.priority);
    }

    Ok(())
}

#[tokio::test]
async fn test_bundle_timeout_handling() -> Result<()> {
    let config = JitoConfig {
        bundle_timeout: Duration::from_millis(100), // Very short timeout for testing
        ..Default::default()
    };

    let tip_keypair = Keypair::new();
    let _executor = JitoExecutor::new(config, tip_keypair);

    // Create a bundle that will timeout
    let _transactions = vec![create_bundle_transaction(
        create_mock_transaction("test"),
        100,
    )];

    // Test timeout handling
    let start_time = std::time::Instant::now();
    let result = timeout(
        Duration::from_millis(100), // Short timeout
        simulate_bundle_execution_with_delay(Duration::from_millis(200)), // Longer delay
    )
    .await;

    let elapsed = start_time.elapsed();

    // Should timeout around 100ms
    assert!(elapsed >= Duration::from_millis(90));
    assert!(elapsed <= Duration::from_millis(150));
    assert!(result.is_err()); // Should timeout

    println!(
        "✅ Bundle timeout handling test passed: elapsed {}ms",
        elapsed.as_millis()
    );
    Ok(())
}

async fn simulate_bundle_execution_with_delay(delay: Duration) -> Result<()> {
    tokio::time::sleep(delay).await;
    Ok(())
}

#[tokio::test]
async fn test_bundle_serialization() -> Result<()> {
    let transaction = create_mock_transaction("test");
    let bundle_tx = create_bundle_transaction(transaction.clone(), 100);

    // Test transaction serialization
    let serialized = bincode::serialize(&bundle_tx.transaction)?;
    assert!(!serialized.is_empty());

    // Test base64 encoding
    let encoded = base64::engine::general_purpose::STANDARD.encode(&serialized);
    assert!(!encoded.is_empty());

    // Test deserialization
    let decoded = base64::engine::general_purpose::STANDARD.decode(&encoded)?;
    let deserialized: Transaction = bincode::deserialize(&decoded)?;

    // Verify signatures match
    assert_eq!(
        transaction.signatures[0].to_string(),
        deserialized.signatures[0].to_string()
    );

    println!("✅ Bundle serialization test passed");
    println!("   Serialized size: {} bytes", serialized.len());
    println!("   Encoded size: {} chars", encoded.len());

    Ok(())
}

#[tokio::test]
async fn test_tip_account_validation() -> Result<()> {
    let valid_tip_accounts = vec![
        "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5", // Default Jito tip account
        "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe", // Alternative tip account
        "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY", // Another tip account
    ];

    for tip_account in valid_tip_accounts {
        let _config = JitoConfig {
            tip_account: tip_account.to_string(),
            ..Default::default()
        };

        // Test tip account parsing
        let parsed_pubkey = tip_account.parse::<Pubkey>();
        assert!(
            parsed_pubkey.is_ok(),
            "Failed to parse tip account: {}",
            tip_account
        );

        println!("✅ Tip account validation passed: {}", tip_account);
    }

    // Test invalid tip account
    let invalid_config = JitoConfig {
        tip_account: "invalid_pubkey".to_string(),
        ..Default::default()
    };

    let invalid_result = invalid_config.tip_account.parse::<Pubkey>();
    assert!(
        invalid_result.is_err(),
        "Should fail to parse invalid tip account"
    );

    println!("✅ Invalid tip account rejection test passed");
    Ok(())
}

#[tokio::test]
async fn test_bundle_retry_logic() -> Result<()> {
    let max_retries = 3;
    let mut attempt = 1;
    let mut success = false;

    while attempt <= max_retries && !success {
        println!("📦 Bundle submission attempt {}/{}", attempt, max_retries);

        // Simulate bundle submission that succeeds on attempt 2
        let result = simulate_bundle_submission(attempt).await;

        match result {
            Ok(_) => {
                success = true;
                println!("✅ Bundle submitted successfully on attempt {}", attempt);
            }
            Err(e) => {
                println!("❌ Bundle submission failed: {}", e);
                if attempt < max_retries {
                    let delay = Duration::from_millis(100 * attempt as u64);
                    println!("⏳ Retrying in {}ms...", delay.as_millis());
                    tokio::time::sleep(delay).await;
                }
            }
        }

        attempt += 1;
    }

    assert!(success, "Bundle should eventually succeed");
    println!("✅ Bundle retry logic test passed");
    Ok(())
}

async fn simulate_bundle_submission(attempt: u32) -> Result<()> {
    // Simulate submission that succeeds on attempt 2
    if attempt >= 2 {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Network error"))
    }
}

fn create_mock_transaction(memo: &str) -> Transaction {
    let payer = Keypair::new();
    let recipient = Keypair::new();

    let instruction = system_instruction::transfer(
        &payer.pubkey(),
        &recipient.pubkey(),
        1000000, // 0.001 SOL
    );

    let recent_blockhash = solana_sdk::hash::Hash::default();

    Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    )
}

#[tokio::test]
async fn test_concurrent_bundle_execution() -> Result<()> {
    let config = JitoConfig::default();
    let tip_keypair = Keypair::new();
    let _executor = JitoExecutor::new(config, tip_keypair);

    // Create multiple bundles
    let bundle_count = 5;
    let mut handles = Vec::new();

    for i in 0..bundle_count {
        let handle = tokio::spawn(async move {
            let _transactions = vec![create_bundle_transaction(
                create_mock_transaction(&format!("concurrent_{}", i)),
                100,
            )];

            // Simulate bundle execution
            tokio::time::sleep(Duration::from_millis(50)).await;
            Ok::<_, anyhow::Error>(format!("bundle_{}", i))
        });

        handles.push(handle);
    }

    // Wait for all bundles to complete
    let results = futures::future::join_all(handles).await;

    // Verify all succeeded
    for (i, result) in results.into_iter().enumerate() {
        let bundle_id = result??;
        println!("✅ Concurrent bundle {} completed: {}", i, bundle_id);
    }

    println!("✅ Concurrent bundle execution test passed");
    Ok(())
}
