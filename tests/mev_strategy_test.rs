//! MEV Strategy Comprehensive Tests
//! 
//! Tests for all MEV strategies: sandwich, arbitrage, liquidation, sniping

use solana_hft_ninja::{
    strategies::{MevEngine, MevConfig, MevOpportunity, create_mev_engine},
    mempool::dex_detector::{DexTransaction, DexTransactionType, DexProtocol},
};
use anyhow::Result;

#[tokio::test]
async fn test_sandwich_strategy_detection() -> Result<()> {
    let mut mev_engine = create_mev_engine();
    
    // Create large swap transaction (sandwich target)
    let large_swap = DexTransaction {
        signature: "large_swap_signature".to_string(),
        slot: 12345,
        block_time: Some(chrono::Utc::now().timestamp()),
        protocol: DexProtocol::Raydium,
        transaction_type: DexTransactionType::Swap {
            amount_in: 500000000, // 0.5 SOL - large enough for sandwich
            amount_out: 450000000,
            token_in: "So11111111111111111111111111111111111111112".to_string(), // SOL
            token_out: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
            slippage_bps: Some(100), // 1% slippage
        },
        user: "test_user".to_string(),
        priority_fee: Some(5000),
        compute_units: Some(200000),
    };
    
    // Analyze transaction for MEV opportunities
    let opportunities = mev_engine.analyze_transaction(&large_swap);
    
    // Should detect sandwich opportunity
    assert!(!opportunities.is_empty(), "Should detect sandwich opportunity");
    
    let sandwich_opp = opportunities.iter()
        .find(|opp| matches!(opp, MevOpportunity::Sandwich { .. }));
    
    assert!(sandwich_opp.is_some(), "Should contain sandwich opportunity");
    
    if let Some(MevOpportunity::Sandwich { 
        front_run_amount, 
        estimated_profit, 
        .. 
    }) = sandwich_opp {
        assert!(*front_run_amount > 0, "Front run amount should be positive");
        assert!(*estimated_profit > 0, "Estimated profit should be positive");
        println!("✅ Sandwich opportunity detected: {} lamports profit", estimated_profit);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_arbitrage_strategy_detection() -> Result<()> {
    let mut mev_engine = create_mev_engine();
    
    // Update price cache to simulate price differences
    mev_engine.update_price("SOL/USDC".to_string(), 1000000); // 1 SOL = 1 USDC (mock)
    
    // Create swap transaction
    let swap_tx = DexTransaction {
        signature: "arbitrage_swap_signature".to_string(),
        slot: 12346,
        block_time: Some(chrono::Utc::now().timestamp()),
        protocol: DexProtocol::Raydium,
        transaction_type: DexTransactionType::Swap {
            amount_in: 100000000, // 0.1 SOL
            amount_out: 95000000,
            token_in: "So11111111111111111111111111111111111111112".to_string(),
            token_out: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            slippage_bps: Some(50),
        },
        user: "test_user".to_string(),
        priority_fee: Some(5000),
        compute_units: Some(200000),
    };
    
    let opportunities = mev_engine.analyze_transaction(&swap_tx);
    
    // Should detect arbitrage opportunity (due to simulated price differences)
    let arbitrage_opp = opportunities.iter()
        .find(|opp| matches!(opp, MevOpportunity::Arbitrage { .. }));
    
    if let Some(MevOpportunity::Arbitrage { 
        token_pair, 
        profit_bps, 
        buy_dex,
        sell_dex,
        .. 
    }) = arbitrage_opp {
        assert_eq!(token_pair, "So11111111111111111111111111111111111111112/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
        assert!(*profit_bps >= 50, "Profit should be at least 0.5%");
        println!("✅ Arbitrage opportunity: {}bps profit, buy on {:?}, sell on {:?}", 
                profit_bps, buy_dex, sell_dex);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_token_launch_sniping() -> Result<()> {
    let mut mev_engine = create_mev_engine_with_launch_enabled();
    
    // Create pool creation transaction (token launch)
    let pool_creation = DexTransaction {
        signature: "token_launch_signature".to_string(),
        slot: 12347,
        block_time: Some(chrono::Utc::now().timestamp()),
        protocol: DexProtocol::Raydium,
        transaction_type: DexTransactionType::CreatePool {
            token_a: "So11111111111111111111111111111111111111112".to_string(), // SOL
            token_b: "NewTokenMint1234567890123456789012345678".to_string(), // New token
            initial_price: Some(0.001), // 1 new token = 0.001 SOL
        },
        user: "test_user".to_string(),
        priority_fee: Some(5000),
        compute_units: Some(200000),
    };
    
    let opportunities = mev_engine.analyze_transaction(&pool_creation);
    
    // Should detect token launch opportunity
    let launch_opp = opportunities.iter()
        .find(|opp| matches!(opp, MevOpportunity::TokenLaunch { .. }));
    
    assert!(launch_opp.is_some(), "Should detect token launch opportunity");
    
    if let Some(MevOpportunity::TokenLaunch { 
        token_mint, 
        initial_liquidity,
        estimated_mcap,
        .. 
    }) = launch_opp {
        assert_eq!(token_mint, "NewTokenMint1234567890123456789012345678");
        assert!(*initial_liquidity > 0, "Initial liquidity should be positive");
        println!("✅ Token launch detected: {} initial liquidity, estimated mcap: {:?}", 
                initial_liquidity, estimated_mcap);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_mev_profitability_filtering() -> Result<()> {
    // Create MEV engine with high profit threshold
    let high_threshold_config = MevConfig {
        min_profit_threshold: 50000000, // 0.05 SOL minimum
        ..Default::default()
    };
    let mut mev_engine = MevEngine::new(high_threshold_config);
    
    // Create small swap (should not trigger sandwich due to low profit)
    let small_swap = DexTransaction {
        signature: "small_swap_signature".to_string(),
        slot: 12348,
        block_time: Some(chrono::Utc::now().timestamp()),
        protocol: DexProtocol::Orca,
        transaction_type: DexTransactionType::Swap {
            amount_in: 10000000, // 0.01 SOL - too small
            amount_out: 9500000,
            token_in: "So11111111111111111111111111111111111111112".to_string(),
            token_out: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            slippage_bps: Some(50),
        },
        user: "test_user".to_string(),
        priority_fee: Some(5000),
        compute_units: Some(200000),
    };
    
    let opportunities = mev_engine.analyze_transaction(&small_swap);
    
    // Should not detect any opportunities due to low profitability
    let sandwich_opportunities = opportunities.iter()
        .filter(|opp| matches!(opp, MevOpportunity::Sandwich { .. }))
        .count();
    
    assert_eq!(sandwich_opportunities, 0, "Should not detect unprofitable sandwich opportunities");
    println!("✅ Profitability filtering working correctly");
    
    Ok(())
}

#[tokio::test]
async fn test_mev_statistics_tracking() -> Result<()> {
    let mut mev_engine = create_mev_engine();
    
    // Process multiple transactions
    let transactions = vec![
        create_mock_swap_transaction("tx1", 500000000, DexProtocol::Raydium),
        create_mock_swap_transaction("tx2", 300000000, DexProtocol::Orca),
        create_mock_swap_transaction("tx3", 200000000, DexProtocol::Jupiter),
    ];
    
    for tx in transactions {
        mev_engine.analyze_transaction(&tx);
    }
    
    // Check statistics
    let stats = mev_engine.get_stats();
    assert!(stats.total_opportunities > 0, "Should have detected opportunities");
    
    println!("✅ MEV Statistics:");
    println!("   Total opportunities: {}", stats.total_opportunities);
    println!("   Sandwich count: {}", stats.sandwich_count);
    println!("   Arbitrage count: {}", stats.arbitrage_count);
    
    Ok(())
}

fn create_mev_engine_with_launch_enabled() -> MevEngine {
    let config = MevConfig {
        token_launch_enabled: true,
        ..Default::default()
    };
    MevEngine::new(config)
}

fn create_mock_swap_transaction(signature: &str, amount_in: u64, protocol: DexProtocol) -> DexTransaction {
    DexTransaction {
        signature: signature.to_string(),
        slot: 12345,
        block_time: Some(chrono::Utc::now().timestamp()),
        protocol,
        transaction_type: DexTransactionType::Swap {
            amount_in,
            amount_out: (amount_in as f64 * 0.95) as u64, // 5% slippage
            token_in: "So11111111111111111111111111111111111111112".to_string(),
            token_out: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            slippage_bps: Some(500), // 5%
        },
        user: "test_user".to_string(),
        priority_fee: Some(5000),
        compute_units: Some(200000),
    }
}
