//! Integration tests for mempool listener module

use solana_hft_ninja::mempool::listener::HeliusConfig;
use solana_hft_ninja::mempool::*;
use solana_sdk::{
    message::{Message, VersionedMessage},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::{Transaction, VersionedTransaction},
};
use std::str::FromStr;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

/// Integration test for full mempool listener flow
#[tokio::test]
async fn test_full_mempool_listener_flow() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let metrics = MempoolMetrics::new();
    let parser = ZeroCopyParser::new(metrics.clone(), 16 * 1024 * 1024);

    let config = HeliusConfig {
        api_key: "test-key".to_string(),
        endpoint: "https://test.helius.xyz".to_string(),
        commitment: CommitmentLevel::Processed,
        max_reconnect_attempts: 3,
        reconnect_delay_ms: 100,
    };

    let listener = MempoolListener::new(config, parser, metrics.clone(), tx);

    // Test that listener can be created
    assert!(!listener.is_running().await);

    // Test metrics are properly initialized
    let stats = metrics.get_stats();
    assert_eq!(stats.transactions_processed, 0);
    assert_eq!(stats.dex_detections, 0);
}

/// Test end-to-end transaction processing
#[tokio::test]
async fn test_end_to_end_transaction_processing() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let metrics = MempoolMetrics::new();
    let parser = ZeroCopyParser::new(metrics.clone(), 16 * 1024 * 1024);

    // Create test transaction with DEX interaction
    let keypair = Keypair::new();
    let raydium_program = Pubkey::from_str(dex::program_ids::RAYDIUM_AMM_V4).unwrap();

    let instruction = solana_sdk::instruction::Instruction {
        program_id: raydium_program,
        accounts: vec![
            solana_sdk::instruction::AccountMeta::new(Pubkey::new_unique(), false),
            solana_sdk::instruction::AccountMeta::new(Pubkey::new_unique(), false),
        ],
        data: vec![9, 0, 0, 0], // Raydium swap instruction
    };

    let message = Message::new(&[instruction], Some(&keypair.pubkey()));
    let transaction = Transaction::new(&[&keypair], message, Default::default());
    let versioned = VersionedTransaction::from(transaction);

    let serialized = bincode::serialize(&versioned).unwrap();

    // Parse transaction
    let parsed = parser
        .parse_transaction(&serialized, 123456789, 1000)
        .unwrap();

    // Send through channel
    tx.send(parsed.clone()).unwrap();

    // Receive from channel
    let received = timeout(Duration::from_millis(100), rx.recv())
        .await
        .unwrap()
        .unwrap();

    // Verify data integrity
    assert_eq!(received.signature, parsed.signature);
    assert_eq!(received.slot, 1000);
    assert_eq!(received.timestamp, 123456789);
    assert!(!received.dex_interactions.is_empty());

    // Verify DEX interaction
    let dex_interaction = &received.dex_interactions[0];
    assert_eq!(dex_interaction.program, dex::DexProgram::RaydiumAmm);
    assert_eq!(dex_interaction.instruction_type, dex::InstructionType::Swap);
}

/// Test memory usage under load
#[tokio::test]
async fn test_memory_usage_under_load() {
    let metrics = MempoolMetrics::new();
    let parser = ZeroCopyParser::new(metrics.clone(), 1024 * 1024); // 1MB limit

    let mut handles = vec![];

    for i in 0..100 {
        let parser = parser.clone();
        let metrics = metrics.clone();

        handles.push(tokio::spawn(async move {
            // Fix KeypairPubkeyMismatch: use Transaction::new_signed_with_payer
            let keypair = Keypair::new();
            // Create a transfer instruction (0 lamports to self) - minimal valid transaction
            let instruction = solana_sdk::system_instruction::transfer(
                &keypair.pubkey(),
                &keypair.pubkey(),
                0, // 0 lamports transfer to self
            );
            let recent_blockhash = solana_sdk::hash::Hash::default();
            let transaction = Transaction::new_signed_with_payer(
                &[instruction],
                Some(&keypair.pubkey()),
                &[&keypair],
                recent_blockhash,
            );
            let versioned = VersionedTransaction::from(transaction);

            let serialized = bincode::serialize(&versioned).unwrap();

            // Process with proper error handling
            for j in 0..10 {
                match parser.parse_transaction(&serialized, (i * 10 + j) as u64, i as u64) {
                    Ok(_) => {} // Success - metrics will be updated
                    Err(e) => eprintln!("Parse error in task {}, iteration {}: {:?}", i, j, e),
                }
            }
        }));
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let stats = metrics.get_stats();
    assert_eq!(stats.transactions_processed, 1000);

    // Memory usage should be within limits
    assert!(stats.memory_usage_bytes <= 1024 * 1024);
}

/// Test concurrent transaction processing
#[tokio::test]
async fn test_concurrent_transaction_processing() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let metrics = MempoolMetrics::new();
    let parser = ZeroCopyParser::new(metrics.clone(), 16 * 1024 * 1024);

    let mut handles = vec![];

    // Spawn multiple producers
    for i in 0..5 {
        let tx = tx.clone();
        let parser = parser.clone();

        handles.push(tokio::spawn(async move {
            for j in 0..20 {
                let keypair = Keypair::new();
                let raydium_program = Pubkey::from_str(dex::program_ids::RAYDIUM_AMM_V4).unwrap();

                let instruction = solana_sdk::instruction::Instruction {
                    program_id: raydium_program,
                    accounts: vec![
                        solana_sdk::instruction::AccountMeta::new(Pubkey::new_unique(), false),
                        solana_sdk::instruction::AccountMeta::new(Pubkey::new_unique(), false),
                    ],
                    data: vec![9, 0, 0, 0], // Raydium swap
                };

                let message = Message::new(&[instruction], Some(&keypair.pubkey()));
                let recent_blockhash = solana_sdk::hash::Hash::new_unique();
                let transaction = Transaction::new(&[&keypair], message, recent_blockhash);
                let versioned = VersionedTransaction::from(transaction);

                let serialized = bincode::serialize(&versioned).unwrap();

                if let Ok(parsed) = parser.parse_transaction(&serialized, i * 100 + j, i * 100 + j)
                {
                    tx.send(parsed).unwrap();
                }
            }
        }));
    }

    // Collect results
    let mut received_count = 0;
    let timeout_duration = Duration::from_millis(500);

    while let Ok(Some(_)) = timeout(timeout_duration, rx.recv()).await {
        received_count += 1;
        if received_count >= 100 {
            break;
        }
    }

    assert_eq!(received_count, 100);

    let stats = metrics.get_stats();
    assert_eq!(stats.transactions_processed, 100);
    assert_eq!(stats.dex_detections, 100);
}

/// Test error handling and recovery
#[tokio::test]
async fn test_error_handling_and_recovery() {
    let metrics = MempoolMetrics::new();
    let parser = ZeroCopyParser::new(metrics.clone(), 1024); // 1KB limit

    // Test oversized transaction
    let large_data = vec![0u8; 2048];
    let result = parser.parse_transaction(&large_data, 0, 0);

    assert!(matches!(result, Err(MempoolError::MemoryLimitExceeded(0))));

    // Test invalid transaction data
    let invalid_data = vec![1, 2, 3, 4, 5];
    let result = parser.parse_transaction(&invalid_data, 0, 0);

    assert!(matches!(result, Err(MempoolError::Deserialization(_))));

    // Verify error metrics
    let stats = metrics.get_stats();
    assert!(stats.deserialization_errors > 0);
}

/// Test DEX program detection accuracy
#[tokio::test]
async fn test_dex_program_detection_accuracy() {
    let test_programs = vec![
        (
            dex::program_ids::RAYDIUM_AMM_V4,
            dex::DexProgram::RaydiumAmm,
        ),
        (dex::program_ids::RAYDIUM_CLMM, dex::DexProgram::RaydiumClmm),
        (
            dex::program_ids::ORCA_WHIRLPOOL,
            dex::DexProgram::OrcaWhirlpool,
        ),
        (
            dex::program_ids::ORCA_AQUAFARM,
            dex::DexProgram::OrcaAquafarm,
        ),
        (dex::program_ids::JUPITER_V6, dex::DexProgram::JupiterV6),
        (
            dex::program_ids::JUPITER_LIMIT_ORDER,
            dex::DexProgram::JupiterLimitOrder,
        ),
        (dex::program_ids::JUPITER_DCA, dex::DexProgram::JupiterDca),
    ];

    for (program_id_str, expected_program) in test_programs {
        let program_id = Pubkey::from_str(program_id_str).unwrap();

        // Create transaction with this program
        let keypair = Keypair::new();
        let instruction = solana_sdk::instruction::Instruction {
            program_id,
            accounts: vec![
                solana_sdk::instruction::AccountMeta::new(Pubkey::new_unique(), false),
                solana_sdk::instruction::AccountMeta::new(Pubkey::new_unique(), false),
            ],
            data: vec![1, 2, 3, 4],
        };

        let message = Message::new(&[instruction], Some(&keypair.pubkey()));
        let transaction = Transaction::new(&[&keypair], message, Default::default());
        let versioned = VersionedTransaction::from(transaction);

        let serialized = bincode::serialize(&versioned).unwrap();

        // Parse and verify DEX detection
        let metrics = MempoolMetrics::new();
        let parser = ZeroCopyParser::new(metrics, 16 * 1024 * 1024);
        let parsed = parser.parse_transaction(&serialized, 0, 0).unwrap();

        assert!(!parsed.dex_interactions.is_empty());
        assert_eq!(parsed.dex_interactions[0].program, expected_program);
    }
}

/// Test listener lifecycle
#[tokio::test]
async fn test_listener_lifecycle() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let metrics = MempoolMetrics::new();
    let parser = ZeroCopyParser::new(metrics.clone(), 16 * 1024 * 1024);

    let config = HeliusConfig {
        api_key: "test-key".to_string(),
        endpoint: "https://test.helius.xyz".to_string(),
        commitment: CommitmentLevel::Processed,
        max_reconnect_attempts: 1,
        reconnect_delay_ms: 50,
    };

    let listener = MempoolListener::new(config, parser, metrics, tx);

    // Test start/stop lifecycle
    assert!(!listener.is_running().await);

    // Note: We don't actually start the listener in tests as it requires network
    // This is more of a structural test
}

/// Test performance under realistic load
#[tokio::test]
async fn test_performance_under_load() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let metrics = MempoolMetrics::new();
    let parser = ZeroCopyParser::new(metrics.clone(), 16 * 1024 * 1024);

    let start_time = std::time::Instant::now();
    let mut handles = vec![];

    // Process 1000 transactions concurrently
    for i in 0..10 {
        let tx = tx.clone();
        let parser = parser.clone();

        handles.push(tokio::spawn(async move {
            for j in 0..100 {
                let keypair = Keypair::new();
                let message = Message::new(&[], Some(&keypair.pubkey()));
                let recent_blockhash = solana_sdk::hash::Hash::new_unique();
                let transaction = Transaction::new(&[&keypair], message, recent_blockhash);
                let versioned = VersionedTransaction::from(transaction);

                let serialized = bincode::serialize(&versioned).unwrap();

                if let Ok(parsed) = parser.parse_transaction(&serialized, i * 100 + j, i * 100 + j)
                {
                    tx.send(parsed).unwrap();
                }
            }
        }));
    }

    // Collect all results
    let mut received = 0;
    while let Ok(Some(_)) = timeout(Duration::from_millis(100), rx.recv()).await {
        received += 1;
        if received >= 1000 {
            break;
        }
    }

    let elapsed = start_time.elapsed();

    // Ensure we processed all transactions
    assert_eq!(received, 1000);

    // Ensure processing was fast (<500ms for 1000 transactions)
    assert!(elapsed.as_millis() < 500);

    let stats = metrics.get_stats();
    assert_eq!(stats.transactions_processed, 1000);
}
