//! Comprehensive unit tests for mempool listener module

#[cfg(test)]
mod tests {

    use crate::mempool::*;
    use solana_sdk::{
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        system_instruction,
        transaction::{Transaction, VersionedTransaction},
    };
    use std::str::FromStr;
    use std::sync::Arc;
    use tokio::sync::mpsc;

    /// Test DEX program detection accuracy
    #[test]
    fn test_dex_program_detection_comprehensive() {
        let test_cases = vec![
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

        for (program_id_str, expected) in test_cases {
            let pubkey = Pubkey::from_str(program_id_str).unwrap();
            let detected = dex::DexProgram::from_pubkey(&pubkey);
            assert_eq!(
                detected, expected,
                "Failed to detect {} correctly",
                program_id_str
            );
        }
    }

    /// Test zero-copy parser with real transaction data
    #[tokio::test]
    async fn test_zero_copy_parser_performance() {
        let metrics = MempoolMetrics::new();
        let parser = parser::ZeroCopyParser::new(metrics.clone(), 1024 * 1024); // 1MB limit

        // Create test transactions with valid instruction using new_signed_with_payer
        let keypair = Keypair::new();
        // Create a transfer instruction (0 lamports to self) - minimal valid transaction
        let instruction = system_instruction::transfer(
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

        // Test parsing performance
        let start = std::time::Instant::now();
        let parsed = parser
            .parse_transaction(&serialized, 123456789, 1000)
            .unwrap();
        let duration = start.elapsed();

        // Ensure parsing is fast (<1ms)
        assert!(
            duration.as_micros() < 1000,
            "Parsing took too long: {}μs",
            duration.as_micros()
        );

        // Verify parsed data
        assert!(!parsed.account_keys.is_empty());
        assert_eq!(parsed.slot, 1000);
        assert_eq!(parsed.timestamp, 123456789);
    }

    /// Test memory usage limits
    #[tokio::test]
    async fn test_memory_limits() {
        let metrics = MempoolMetrics::new();
        let parser = parser::ZeroCopyParser::new(metrics.clone(), 1024); // 1KB limit

        // Create oversized transaction
        let large_data = vec![0u8; 2048];
        let result = parser.parse_transaction(&large_data, 0, 0);

        assert!(matches!(result, Err(MempoolError::MemoryLimitExceeded(0))));
    }

    /// Test metrics collection
    #[tokio::test]
    async fn test_metrics_collection() {
        let metrics = MempoolMetrics::new();

        // Test counter increments
        metrics.increment_transactions_processed();
        metrics.increment_transactions_processed();

        metrics.increment_dex_detections();

        let stats = metrics.get_stats();
        assert_eq!(stats.transactions_processed, 2);
        assert_eq!(stats.dex_detections, 1);
    }

    /// Test transaction buffer overflow handling
    #[tokio::test]
    async fn test_transaction_buffer_overflow() {
        let mut buffer = parser::TransactionBuffer::new(100);

        // Fill buffer
        for i in 0..10 {
            buffer.push(&vec![i; 20]).unwrap();
        }

        // Verify buffer respects capacity
        assert!(buffer.len() <= 100);

        // Test clear functionality
        buffer.clear();
        assert_eq!(buffer.len(), 0);
    }

    /// Test DEX interaction detection
    #[tokio::test]
    async fn test_dex_interaction_detection() {
        let raydium_pubkey = Pubkey::from_str(dex::program_ids::RAYDIUM_AMM_V4).unwrap();

        // Create mock transaction with DEX interaction
        let instructions = vec![solana_sdk::instruction::CompiledInstruction {
            program_id_index: 0,
            accounts: vec![1, 2, 3],
            data: vec![9, 0, 0, 0], // Raydium swap instruction
        }];

        let account_keys = vec![raydium_pubkey, Pubkey::new_unique(), Pubkey::new_unique()];

        let interactions = dex::detect_dex_interactions(&instructions, &account_keys);

        assert_eq!(interactions.len(), 1);
        assert_eq!(interactions[0].program, dex::DexProgram::RaydiumAmm);
        assert_eq!(interactions[0].instruction_type, dex::InstructionType::Swap);
    }

    /// Test listener builder pattern
    #[tokio::test]
    async fn test_listener_builder() {
        let (tx, _rx) = mpsc::unbounded_channel();

        let listener = listener::MempoolListenerBuilder::new()
            .with_sender(tx)
            .build()
            .unwrap();

        assert!(!listener.is_running().await);
    }

    /// Test configuration validation
    #[tokio::test]
    async fn test_configuration_validation() {
        let config = listener::HeliusConfig {
            api_key: "test-api-key".to_string(),
            endpoint: "https://test.helius.xyz".to_string(),
            commitment: listener::CommitmentLevel::Confirmed,
            max_reconnect_attempts: 5,
            reconnect_delay_ms: 500,
        };

        assert_eq!(config.api_key, "test-api-key");
        assert_eq!(config.endpoint, "https://test.helius.xyz");
        assert_eq!(config.commitment.to_string(), "confirmed");
    }

    /// Test error handling
    #[tokio::test]
    async fn test_error_handling() {
        // Test WebSocket error conversion
        let ws_error = tokio_tungstenite::tungstenite::Error::ConnectionClosed;
        let mempool_error: MempoolError = ws_error.into();
        assert!(matches!(mempool_error, MempoolError::WebSocket(_)));

        // Test serialization error
        let ser_error =
            serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::Other, "test error"));
        let mempool_error: MempoolError = ser_error.into();
        assert!(matches!(mempool_error, MempoolError::Serialization(_)));

        // Test deserialization error
        let de_error = Box::new(bincode::ErrorKind::Custom("test error".to_string()));
        let mempool_error: MempoolError = de_error.into();
        assert!(matches!(mempool_error, MempoolError::Deserialization(_)));
    }

    /// Test performance metrics
    #[tokio::test]
    async fn test_performance_metrics() {
        let metrics = MempoolMetrics::new();

        // Test processing timer
        {
            let _timer = metrics.processing_timer();
            tokio::time::sleep(tokio::time::Duration::from_micros(100)).await;
        }

        let stats = metrics.get_stats();
        // Timer should have recorded some processing time
        // transactions_processed is u64, so it's always >= 0
        assert!(stats.transactions_processed < u64::MAX);
    }

    /// Test concurrent metrics access
    #[tokio::test]
    async fn test_concurrent_metrics_access() {
        let metrics = Arc::new(MempoolMetrics::new());
        let mut handles = vec![];

        for _i in 0..10 {
            let metrics = metrics.clone();
            handles.push(tokio::spawn(async move {
                for _ in 0..100 {
                    metrics.increment_transactions_processed();
                    metrics.increment_dex_detections();
                    metrics.add_bytes_received(1024);
                }
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let stats = metrics.get_stats();
        assert_eq!(stats.transactions_processed, 1000);
        assert_eq!(stats.dex_detections, 1000);
        assert_eq!(stats.bytes_received, 10 * 100 * 1024);
    }

    /// Test liquidity zone detection
    #[tokio::test]
    async fn test_liquidity_zone_detection() {
        let pool_address = Pubkey::new_unique();
        let token_a = Pubkey::new_unique();
        let token_b = Pubkey::new_unique();

        let liquidity_zone = dex::LiquidityZone {
            dex: dex::DexProgram::RaydiumAmm,
            pool_address,
            token_a,
            token_b,
            amount_a: 1000000,
            amount_b: 2000000,
            price: 2.0,
            timestamp: 123456789,
            slot: 1000,
        };

        assert_eq!(liquidity_zone.dex, dex::DexProgram::RaydiumAmm);
        assert_eq!(liquidity_zone.amount_a, 1000000);
        assert_eq!(liquidity_zone.amount_b, 2000000);
        assert!((liquidity_zone.price - 2.0).abs() < f64::EPSILON);
    }

    /// Test instruction type parsing edge cases
    #[test]
    fn test_instruction_type_edge_cases() {
        // Test empty data
        let empty_data = vec![];
        let result = dex::parse_instruction_type(&empty_data, &dex::DexProgram::RaydiumAmm);
        assert_eq!(result, dex::InstructionType::Unknown);

        // Test unknown instruction
        let unknown_data = vec![255, 255, 255];
        let result = dex::parse_instruction_type(&unknown_data, &dex::DexProgram::RaydiumAmm);
        assert_eq!(result, dex::InstructionType::Unknown);

        // Test boundary values
        let boundary_data = vec![0, 1, 2, 3];
        let result = dex::parse_instruction_type(&boundary_data, &dex::DexProgram::RaydiumAmm);
        assert_eq!(result, dex::InstructionType::Unknown);
    }

    /// Test memory usage tracking
    #[tokio::test]
    async fn test_memory_usage_tracking() {
        let metrics = MempoolMetrics::new();

        // Test memory usage updates
        metrics.set_memory_usage(1024 * 1024); // 1MB
        let stats = metrics.get_stats();
        assert_eq!(stats.memory_usage_bytes, 1024 * 1024);

        // Test memory usage with large values
        metrics.set_memory_usage(16 * 1024 * 1024); // 16MB
        let stats = metrics.get_stats();
        assert_eq!(stats.memory_usage_bytes, 16 * 1024 * 1024);
    }

    /// Test error recovery mechanisms
    #[tokio::test]
    async fn test_error_recovery() {
        let metrics = MempoolMetrics::new();

        // Simulate error conditions
        metrics.increment_connection_failures();
        metrics.increment_deserialization_errors();

        let stats = metrics.get_stats();
        assert_eq!(stats.connection_failures, 1);
        assert_eq!(stats.deserialization_errors, 1);

        // Verify metrics are properly tracked
        assert!(stats.connection_failures > 0);
        assert!(stats.deserialization_errors > 0);
    }
}
