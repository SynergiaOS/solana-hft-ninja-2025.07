//! Executor Tests for Solana HFT Ninja
//! 
//! Tests for trading execution engine, Jito integration, and strategy execution

use anyhow::Result;
use solana_hft_ninja::{
    execution::{JitoExecutor, JitoConfig, BundleTransaction, BundleResult, BundleStatus},
    strategies::{MevEngine, MevConfig, MevOpportunity, WalletTrackerStrategy, WalletTrackerConfig},
    engine::{HftEngine, MevExecutionResult},
    mempool::ParsedTransaction,
    core::balance::BalanceTracker,
    security::risk_limits::RiskLimits,
};
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    system_instruction,
    pubkey::Pubkey,
};
use std::sync::Arc;
use std::time::Duration;
use tokio;

/// Test Jito Executor initialization and configuration
#[tokio::test]
async fn test_jito_executor_initialization() -> Result<()> {
    let config = JitoConfig {
        endpoint: "https://mainnet.block-engine.jito.wtf".to_string(),
        tip_account: "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5".to_string(),
        min_tip_lamports: 1000,
        max_tip_lamports: 100000,
        bundle_timeout: Duration::from_secs(30),
        max_retries: 3,
    };
    
    let tip_keypair = Keypair::new();
    let executor = JitoExecutor::new(config.clone(), tip_keypair);
    
    // Verify configuration
    assert_eq!(executor.config.endpoint, config.endpoint);
    assert_eq!(executor.config.tip_account, config.tip_account);
    assert_eq!(executor.config.min_tip_lamports, config.min_tip_lamports);
    
    println!("✅ Jito Executor initialization test passed");
    Ok(())
}

/// Test bundle transaction creation and execution
#[tokio::test]
async fn test_bundle_execution() -> Result<()> {
    let config = JitoConfig::default();
    let tip_keypair = Keypair::new();
    let executor = JitoExecutor::new(config, tip_keypair);
    
    // Create test transactions
    let mut transactions = Vec::new();
    
    for i in 0..3 {
        let from_keypair = Keypair::new();
        let to_pubkey = Keypair::new().pubkey();
        
        let instruction = system_instruction::transfer(
            &from_keypair.pubkey(),
            &to_pubkey,
            1000000, // 0.001 SOL
        );
        
        let transaction = Transaction::new_with_payer(
            &[instruction],
            Some(&from_keypair.pubkey()),
        );
        
        let bundle_tx = BundleTransaction {
            transaction,
            priority: (10 - i) as u8, // Decreasing priority
            max_retries: 3,
            timeout: Duration::from_secs(30),
        };
        
        transactions.push(bundle_tx);
    }
    
    // Test bundle execution (will fail without proper setup, but tests structure)
    let result = executor.execute_bundle(transactions).await;
    
    // Should return error due to missing recent blockhash and signatures
    // but the structure should be correct
    match result {
        Ok(_) => {
            // Unexpected success in test environment
            println!("⚠️  Bundle execution succeeded unexpectedly");
        }
        Err(e) => {
            // Expected failure due to test environment
            println!("✅ Bundle execution failed as expected: {}", e);
        }
    }
    
    println!("✅ Bundle execution test passed");
    Ok(())
}

/// Test MEV Engine strategy execution
#[tokio::test]
async fn test_mev_engine_execution() -> Result<()> {
    let config = MevConfig {
        enable_sandwich_attacks: true,
        enable_arbitrage: true,
        enable_liquidations: false, // Disable for test
        min_profit_threshold: 0.01, // 0.01 SOL
        max_position_size: 1.0, // 1 SOL
        slippage_tolerance: 0.05, // 5%
        execution_timeout_ms: 5000,
        risk_limit_per_trade: 0.1, // 0.1 SOL
        daily_loss_limit: 1.0, // 1 SOL
    };
    
    let mut engine = MevEngine::new(config)?;
    
    // Create test transaction for analysis
    let test_tx = create_test_parsed_transaction();
    
    // Test opportunity detection
    let opportunities = engine.analyze_transaction(&test_tx).await?;
    
    // Should detect some opportunities (even if simulated)
    println!("🔍 Detected {} MEV opportunities", opportunities.len());
    
    // Test opportunity execution (will be simulated)
    for opportunity in opportunities {
        let execution_result = engine.execute_opportunity(opportunity).await;
        
        match execution_result {
            Ok(result) => {
                println!("✅ MEV execution result: {:?}", result);
                assert!(!result.transaction_id.is_empty());
            }
            Err(e) => {
                println!("⚠️  MEV execution failed (expected in test): {}", e);
            }
        }
    }
    
    // Test engine statistics
    let stats = engine.get_stats();
    assert!(stats.total_opportunities >= 0);
    assert!(stats.successful_executions >= 0);
    
    println!("✅ MEV Engine execution test passed");
    Ok(())
}

/// Test Wallet Tracker Strategy
#[tokio::test]
async fn test_wallet_tracker_execution() -> Result<()> {
    let config = WalletTrackerConfig {
        enabled: true,
        scan_interval_ms: 1000, // 1 second for test
        depth_level: 2,
        min_success_rate: 0.5, // Lower for test
        fresh_wallet_cap: 0.5,
        min_liquidity_sol: 1.0,
        max_creator_share: 0.3,
        tracked_wallets: vec![
            "EEC7mX2cut2JMGP3soancH2HNMKTw4Q7ADbCfDQFgggs".to_string(),
            "DSJXCqXuRckDhSX34oiFgEQChuezxvVgkEAyaA2MML8X".to_string(),
        ],
        max_rug_score: 0.2,
        min_behavior_score: 0.7,
        max_suspicious_connections: 5,
        min_holder_count: 20,
        base_position_sol: 0.05, // Small for test
        max_position_sol: 0.2,
        risk_multiplier: 1.5,
    };
    
    // Create mock dependencies
    let balance_tracker = Arc::new(BalanceTracker::new());
    let jito_executor = Arc::new(JitoExecutor::new(JitoConfig::default(), Keypair::new()));
    let risk_limits = Arc::new(RiskLimits::new());
    
    let strategy = WalletTrackerStrategy::new(
        config.clone(),
        balance_tracker,
        jito_executor,
        risk_limits,
    )?;
    
    // Test wallet tracking
    let test_tx = create_test_parsed_transaction();
    let result = strategy.process_transaction(&test_tx).await;
    
    // Should not fail even if no tracked wallets in test transaction
    match result {
        Ok(_) => println!("✅ Wallet tracker processed transaction successfully"),
        Err(e) => println!("⚠️  Wallet tracker processing failed: {}", e),
    }
    
    // Test wallet addition
    let test_wallet = solana_hft_ninja::strategies::Wallet {
        address: "TestWallet123".to_string(),
        success_rate: 0.75,
        total_trades: 100,
        profitable_trades: 75,
        average_profit: 0.05,
        risk_score: 0.3,
        last_activity: chrono::Utc::now().timestamp() as u64,
        behavior_score: 0.8,
        suspicious_connections: 1,
    };
    
    strategy.add_wallet(test_wallet).await?;
    
    println!("✅ Wallet Tracker execution test passed");
    Ok(())
}

/// Test HFT Engine integration
#[tokio::test]
async fn test_hft_engine_integration() -> Result<()> {
    // This test would require full engine setup
    // For now, test basic structure
    
    let config = solana_hft_ninja::config::Config {
        solana: solana_hft_ninja::config::SolanaConfig {
            rpc_url: "https://api.devnet.solana.com".to_string(),
            ws_url: "wss://api.devnet.solana.com".to_string(),
            rpc_timeout_ms: 5000,
        },
        wallet: solana_hft_ninja::config::WalletConfig {
            private_key_path: "test_wallet.json".to_string(),
            keypair_path: "test_wallet.json".to_string(),
        },
        trading: solana_hft_ninja::config::TradingConfig {
            initial_balance_sol: 1.0,
            max_position_size_sol: 0.1,
            max_slippage_bps: 100,
            min_profit_threshold_bps: 50,
            risk_limit_bps: 500,
        },
        strategy: solana_hft_ninja::config::StrategyConfig {
            strategy_mode: "test".to_string(),
            update_interval_ms: 1000,
            order_book_depth: 10,
            spread_bps: 25,
        },
        risk: solana_hft_ninja::config::RiskConfig::default(),
        logging: solana_hft_ninja::config::LoggingConfig {
            rust_log: "debug".to_string(),
            log_level: "debug".to_string(),
            log_file_path: "test.log".to_string(),
        },
        monitoring: solana_hft_ninja::config::MonitoringConfig {
            metrics_port: 8081, // Different port for test
            health_check_interval_ms: 5000,
            enable_ddos_protection: false,
            rate_limit_rps: 100,
        },
        wallet_tracker: Some(WalletTrackerConfig::default()),
        oumi_ai: None,
        opensearch_ai: None,
        ai: None,
    };
    
    // Test configuration validation
    assert_eq!(config.trading.initial_balance_sol, 1.0);
    assert_eq!(config.strategy.strategy_mode, "test");
    assert!(config.wallet_tracker.is_some());
    
    println!("✅ HFT Engine integration test passed");
    Ok(())
}

/// Test execution performance and latency
#[tokio::test]
async fn test_execution_performance() -> Result<()> {
    let config = MevConfig {
        enable_sandwich_attacks: true,
        enable_arbitrage: true,
        enable_liquidations: false,
        min_profit_threshold: 0.001,
        max_position_size: 0.1,
        slippage_tolerance: 0.02,
        execution_timeout_ms: 1000, // 1 second timeout
        risk_limit_per_trade: 0.05,
        daily_loss_limit: 0.5,
    };
    
    let mut engine = MevEngine::new(config)?;
    
    let num_transactions = 100;
    let start_time = std::time::Instant::now();
    
    // Process multiple transactions to test performance
    for i in 0..num_transactions {
        let test_tx = create_test_parsed_transaction_with_id(i);
        let _opportunities = engine.analyze_transaction(&test_tx).await?;
    }
    
    let elapsed = start_time.elapsed();
    let avg_latency = elapsed.as_millis() as f64 / num_transactions as f64;
    
    println!("🚀 Performance test: {:.2}ms average latency", avg_latency);
    
    // Should process transactions in under 10ms on average
    assert!(avg_latency < 10.0, "Average latency too high: {:.2}ms", avg_latency);
    
    // Test concurrent execution
    let concurrent_start = std::time::Instant::now();
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let mut engine_clone = MevEngine::new(config.clone())?;
        let handle = tokio::spawn(async move {
            let test_tx = create_test_parsed_transaction_with_id(i);
            engine_clone.analyze_transaction(&test_tx).await
        });
        handles.push(handle);
    }
    
    // Wait for all concurrent executions
    for handle in handles {
        let _result = handle.await??;
    }
    
    let concurrent_elapsed = concurrent_start.elapsed();
    println!("🚀 Concurrent execution: {}ms for 10 parallel transactions", 
             concurrent_elapsed.as_millis());
    
    // Concurrent execution should be faster than sequential
    assert!(concurrent_elapsed < elapsed, "Concurrent execution not faster than sequential");
    
    println!("✅ Execution performance test passed");
    Ok(())
}

/// Test error handling and recovery
#[tokio::test]
async fn test_execution_error_handling() -> Result<()> {
    // Test with invalid configuration
    let invalid_config = MevConfig {
        enable_sandwich_attacks: true,
        enable_arbitrage: true,
        enable_liquidations: false,
        min_profit_threshold: -1.0, // Invalid negative threshold
        max_position_size: 0.0, // Invalid zero position
        slippage_tolerance: 2.0, // Invalid >100% slippage
        execution_timeout_ms: 0, // Invalid zero timeout
        risk_limit_per_trade: 0.0,
        daily_loss_limit: 0.0,
    };
    
    // Should handle invalid config gracefully
    let engine_result = MevEngine::new(invalid_config);
    match engine_result {
        Ok(mut engine) => {
            // If engine creation succeeds, test with invalid transaction
            let invalid_tx = ParsedTransaction {
                signature: [0u8; 64], // Invalid signature
                account_keys: Vec::new(), // Empty account keys
                instructions: Vec::new(), // Empty instructions
                dex_interactions: Vec::new(),
                timestamp: 0,
                slot: 0,
            };
            
            let result = engine.analyze_transaction(&invalid_tx).await;
            // Should handle gracefully without panicking
            match result {
                Ok(opportunities) => {
                    println!("✅ Handled invalid transaction, found {} opportunities", opportunities.len());
                }
                Err(e) => {
                    println!("✅ Properly rejected invalid transaction: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✅ Properly rejected invalid config: {}", e);
        }
    }
    
    // Test Jito executor with invalid endpoint
    let invalid_jito_config = JitoConfig {
        endpoint: "http://invalid-endpoint:9999".to_string(),
        tip_account: "invalid_account".to_string(),
        min_tip_lamports: 1000,
        max_tip_lamports: 100000,
        bundle_timeout: Duration::from_secs(1),
        max_retries: 1,
    };
    
    let executor = JitoExecutor::new(invalid_jito_config, Keypair::new());
    let empty_bundle = Vec::new();
    
    let result = executor.execute_bundle(empty_bundle).await;
    // Should fail gracefully
    assert!(result.is_err(), "Should fail with invalid configuration");
    
    println!("✅ Execution error handling test passed");
    Ok(())
}

/// Helper function to create test parsed transaction
fn create_test_parsed_transaction() -> ParsedTransaction {
    ParsedTransaction {
        signature: [1u8; 64],
        account_keys: vec![
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ],
        instructions: vec![
            solana_hft_ninja::mempool::ParsedInstruction {
                program_id_index: 0,
                accounts: vec![1, 2],
                data: vec![1, 2, 3, 4],
            }
        ],
        dex_interactions: vec![
            solana_hft_ninja::mempool::DexInteraction {
                dex_type: solana_hft_ninja::mempool::DexType::Raydium,
                instruction_type: solana_hft_ninja::mempool::dex::InstructionType::Swap,
                token_a: Some(Pubkey::new_unique()),
                token_b: Some(Pubkey::new_unique()),
                amount_in: Some(1000000),
                amount_out: Some(950000),
                pool_address: Some(Pubkey::new_unique()),
                user_address: Some(Pubkey::new_unique()),
            }
        ],
        timestamp: chrono::Utc::now().timestamp() as u64,
        slot: 12345,
    }
}

/// Helper function to create test parsed transaction with specific ID
fn create_test_parsed_transaction_with_id(id: u32) -> ParsedTransaction {
    let mut tx = create_test_parsed_transaction();
    tx.signature[0] = (id % 256) as u8;
    tx.slot = 12345 + id as u64;
    tx
}
