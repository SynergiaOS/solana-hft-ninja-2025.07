//! Mempool Router - Bridge between mempool listener and trading engine
//!
//! This module provides real-time communication channel for MEV opportunities
//! detected in the mempool to be processed by the trading engine.

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::mempool::{MempoolError, ParsedTransaction};

/// Global mempool event channel - initialized once at startup
static MEMPOOL_CHANNEL: OnceCell<broadcast::Sender<Arc<MempoolEvent>>> = OnceCell::new();

/// Maximum events in channel before dropping old ones
const CHANNEL_CAPACITY: usize = 4096;

/// Mempool event containing trading opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolEvent {
    pub parsed_tx: ParsedTransaction,
    pub opportunity_type: OpportunityType,
    pub priority: EventPriority,
    pub timestamp_ns: u64,
    pub slot: u64,
}

/// Types of MEV opportunities detected in mempool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpportunityType {
    /// Sandwich attack opportunity
    Sandwich {
        victim_tx_hash: String,
        swap_amount_in: u64,
        swap_amount_out: u64,
        slippage_bps: u64,
        token_in: String,
        token_out: String,
        dex_program: String,
    },

    /// Cross-DEX arbitrage opportunity
    Arbitrage {
        token_pair: TokenPair,
        buy_dex: String,
        sell_dex: String,
        profit_bps: u64,
        optimal_amount: u64,
    },

    /// New token launch detected
    NewToken {
        token_mint: String,
        pool_address: String,
        initial_liquidity_sol: u64,
        dex_program: String,
    },

    /// Large liquidation opportunity
    Liquidation {
        protocol: String,
        collateral_token: String,
        debt_token: String,
        liquidation_amount: u64,
        bonus_bps: u64,
    },

    /// Unknown or low-priority opportunity
    Unknown,
}

/// Event priority for processing order
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventPriority {
    /// Critical - process immediately (sandwich, liquidation)
    Critical = 0,
    /// High - process within 10ms (arbitrage)
    High = 1,
    /// Medium - process within 100ms (new tokens)
    Medium = 2,
    /// Low - process when idle (unknown opportunities)
    Low = 3,
}

/// Token pair for arbitrage opportunities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub base: String,
    pub quote: String,
    pub symbol: String,
}

/// Router statistics for monitoring
#[derive(Debug, Default)]
pub struct RouterStats {
    pub events_sent: u64,
    pub events_dropped: u64,
    pub opportunities_detected: u64,
    pub critical_events: u64,
    pub high_priority_events: u64,
}

/// Initialize the global mempool channel
/// Should be called once at application startup
pub fn init_mempool_channel() -> broadcast::Receiver<Arc<MempoolEvent>> {
    // Check if already initialized
    if let Some(sender) = MEMPOOL_CHANNEL.get() {
        return sender.subscribe();
    }

    let (tx, rx) = broadcast::channel(CHANNEL_CAPACITY);

    if MEMPOOL_CHANNEL.set(tx).is_err() {
        // Race condition - another thread initialized it
        return MEMPOOL_CHANNEL.get().unwrap().subscribe();
    }

    info!(
        "Mempool router initialized with capacity {}",
        CHANNEL_CAPACITY
    );
    rx
}

/// Send mempool event to all subscribers (trading engines)
pub fn send_mempool_event(event: MempoolEvent) -> Result<usize, MempoolError> {
    match MEMPOOL_CHANNEL.get() {
        Some(sender) => {
            let priority = event.priority;
            let opportunity_type = format!("{:?}", event.opportunity_type);

            match sender.send(Arc::new(event)) {
                Ok(receiver_count) => {
                    debug!(
                        "Sent {} priority {} event to {} receivers",
                        opportunity_type, priority as u8, receiver_count
                    );
                    Ok(receiver_count)
                }
                Err(_) => {
                    warn!("No active receivers for mempool events");
                    Err(MempoolError::Config("No active receivers".to_string()))
                }
            }
        }
        None => {
            error!("Mempool channel not initialized");
            Err(MempoolError::Config("Channel not initialized".to_string()))
        }
    }
}

/// Get a new receiver for mempool events
/// Each trading engine should call this to get its own receiver
pub fn subscribe_to_mempool() -> Result<broadcast::Receiver<Arc<MempoolEvent>>, MempoolError> {
    match MEMPOOL_CHANNEL.get() {
        Some(sender) => Ok(sender.subscribe()),
        None => Err(MempoolError::Config("Channel not initialized".to_string())),
    }
}

/// Opportunity detector - analyzes parsed transactions for MEV opportunities
pub struct OpportunityDetector {
    min_sandwich_amount: u64,
    min_arbitrage_profit_bps: u64,
    min_new_token_liquidity: u64,
}

impl OpportunityDetector {
    pub fn new() -> Self {
        Self {
            min_sandwich_amount: 1_000_000_000,     // 1 SOL minimum
            min_arbitrage_profit_bps: 50,           // 0.5% minimum profit
            min_new_token_liquidity: 5_000_000_000, // 5 SOL minimum liquidity
        }
    }

    /// Analyze parsed transaction for MEV opportunities
    pub fn detect_opportunities(&self, parsed_tx: &ParsedTransaction) -> Vec<MempoolEvent> {
        let mut events = Vec::new();
        let timestamp_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        for interaction in &parsed_tx.dex_interactions {
            // Detect sandwich opportunities
            if let Some(sandwich_op) = self.detect_sandwich_opportunity(parsed_tx, interaction) {
                events.push(MempoolEvent {
                    parsed_tx: parsed_tx.clone(),
                    opportunity_type: sandwich_op,
                    priority: EventPriority::Critical,
                    timestamp_ns,
                    slot: parsed_tx.slot,
                });
            }

            // Detect new token launches
            if let Some(new_token_op) = self.detect_new_token_opportunity(parsed_tx, interaction) {
                events.push(MempoolEvent {
                    parsed_tx: parsed_tx.clone(),
                    opportunity_type: new_token_op,
                    priority: EventPriority::Medium,
                    timestamp_ns,
                    slot: parsed_tx.slot,
                });
            }
        }

        events
    }

    /// Detect sandwich attack opportunities
    fn detect_sandwich_opportunity(
        &self,
        parsed_tx: &ParsedTransaction,
        interaction: &crate::mempool::DexInteraction,
    ) -> Option<OpportunityType> {
        use crate::mempool::InstructionType;

        if interaction.instruction_type != InstructionType::Swap {
            return None;
        }

        // Parse swap data (simplified - would need proper instruction parsing)
        let swap_amount = self.estimate_swap_amount(&interaction.data);
        if swap_amount < self.min_sandwich_amount {
            return None;
        }

        // Estimate slippage based on swap size
        let slippage_bps = self.estimate_slippage(swap_amount);
        if slippage_bps < 100 {
            // Less than 1% slippage not worth sandwiching
            return None;
        }

        Some(OpportunityType::Sandwich {
            victim_tx_hash: format!("{:?}", parsed_tx.signature),
            swap_amount_in: swap_amount,
            swap_amount_out: swap_amount * 95 / 100, // Rough estimate
            slippage_bps,
            token_in: interaction
                .accounts
                .get(0)
                .copied()
                .unwrap_or_default()
                .to_string(),
            token_out: interaction
                .accounts
                .get(1)
                .copied()
                .unwrap_or_default()
                .to_string(),
            dex_program: interaction.program.name().to_string(),
        })
    }

    /// Detect new token launch opportunities
    fn detect_new_token_opportunity(
        &self,
        parsed_tx: &ParsedTransaction,
        interaction: &crate::mempool::DexInteraction,
    ) -> Option<OpportunityType> {
        use crate::mempool::InstructionType;

        if interaction.instruction_type != InstructionType::CreatePool {
            return None;
        }

        // Extract pool creation details (simplified)
        let initial_liquidity = self.estimate_initial_liquidity(&interaction.data);
        if initial_liquidity < self.min_new_token_liquidity {
            return None;
        }

        Some(OpportunityType::NewToken {
            token_mint: interaction
                .accounts
                .get(0)
                .copied()
                .unwrap_or_default()
                .to_string(),
            pool_address: interaction
                .accounts
                .get(1)
                .copied()
                .unwrap_or_default()
                .to_string(),
            initial_liquidity_sol: initial_liquidity,
            dex_program: interaction.program.name().to_string(),
        })
    }

    /// Estimate swap amount from instruction data (placeholder)
    fn estimate_swap_amount(&self, _data: &[u8]) -> u64 {
        // TODO: Implement proper instruction parsing
        // This would parse the actual swap instruction data
        1_000_000_000 // 1 SOL placeholder
    }

    /// Estimate slippage based on swap amount
    fn estimate_slippage(&self, swap_amount: u64) -> u64 {
        // Rough slippage estimation based on swap size
        match swap_amount {
            0..=1_000_000_000 => 50,               // 0.5% for small swaps
            1_000_000_001..=10_000_000_000 => 150, // 1.5% for medium swaps
            _ => 300,                              // 3% for large swaps
        }
    }

    /// Estimate initial liquidity from pool creation data
    fn estimate_initial_liquidity(&self, _data: &[u8]) -> u64 {
        // TODO: Parse actual pool creation instruction
        5_000_000_000 // 5 SOL placeholder
    }
}

impl Default for OpportunityDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_channel_initialization() {
        // Try to subscribe first - if channel exists, this will work
        let _rx = match subscribe_to_mempool() {
            Ok(rx) => rx,
            Err(_) => init_mempool_channel(), // Initialize if not exists
        };

        // Channel should now be initialized
        assert!(MEMPOOL_CHANNEL.get().is_some());

        // Test subscription - should work now
        let rx2 = subscribe_to_mempool().unwrap();
        assert_eq!(rx2.len(), 0);
    }

    #[tokio::test]
    async fn test_event_sending() {
        // Ensure channel is initialized - if already initialized, just subscribe
        let _rx = match subscribe_to_mempool() {
            Ok(rx) => rx,
            Err(_) => init_mempool_channel(),
        };

        let event = MempoolEvent {
            parsed_tx: ParsedTransaction {
                signature: [0u8; 64],
                account_keys: vec![],
                instructions: vec![],
                dex_interactions: vec![],
                timestamp: 0,
                slot: 1000,
            },
            opportunity_type: OpportunityType::Unknown,
            priority: EventPriority::Low,
            timestamp_ns: 0,
            slot: 1000,
        };

        let result = send_mempool_event(event);
        assert!(result.is_ok());
    }
}
