// 🥷 Core Trading Engine - Unified Architecture
// High-performance, zero-copy trading engine with event-driven design

pub mod engine;
pub mod types;
pub mod events;
pub mod memory;
pub mod wallet;

// Legacy modules (to be refactored)
pub mod transaction;
pub mod balance;
pub mod solana_client;
pub mod devnet_trader;

// Re-export main types
pub use engine::{Engine, EngineConfig};
pub use types::{
    Price, OrderBook, Trade, Position, Balance as NewBalance,
    MarketData, TradingSignal, RiskMetrics
};
pub use events::{Event, EventBus, EventHandler};
pub use memory::{MemoryPool, ObjectPool};
pub use wallet::Wallet;

// Legacy exports (for compatibility)
pub use wallet::WalletManager;
pub use transaction::TransactionBuilder;
pub use balance::BalanceTracker;
pub use solana_client::{SolanaClient, TransactionResult, AccountInfo, SimulationResult};
pub use devnet_trader::{DevnetTrader, TradeOrder, TradeAction, TradeResult, TradeStatus};

// Performance-critical constants
pub const MAX_ORDER_BOOK_DEPTH: usize = 1000;
pub const MAX_CONCURRENT_TRADES: usize = 100;
pub const EVENT_BUFFER_SIZE: usize = 10000;
pub const MEMORY_POOL_SIZE: usize = 1024 * 1024; // 1MB

// Zero-copy string interning for symbols
use std::collections::HashMap;
use std::sync::RwLock;

lazy_static::lazy_static! {
    static ref SYMBOL_INTERNER: RwLock<HashMap<String, u32>> = RwLock::new(HashMap::new());
    static ref SYMBOL_LOOKUP: RwLock<Vec<String>> = RwLock::new(Vec::new());
}

/// Intern a symbol string for zero-copy operations
pub fn intern_symbol(symbol: &str) -> u32 {
    let mut interner = SYMBOL_INTERNER.write().unwrap();
    let mut lookup = SYMBOL_LOOKUP.write().unwrap();

    if let Some(&id) = interner.get(symbol) {
        return id;
    }

    let id = lookup.len() as u32;
    lookup.push(symbol.to_string());
    interner.insert(symbol.to_string(), id);
    id
}

/// Get symbol string from interned ID
pub fn get_symbol(id: u32) -> Option<String> {
    let lookup = SYMBOL_LOOKUP.read().unwrap();
    lookup.get(id as usize).cloned()
}