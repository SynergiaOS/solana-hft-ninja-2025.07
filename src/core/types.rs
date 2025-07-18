// 🥷 Core Types - Zero-Copy, High-Performance Data Structures
// Optimized for sub-millisecond trading operations

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use bytemuck::{Pod, Zeroable};

/// High-precision price representation (fixed-point arithmetic)
/// Uses 64-bit integer for sub-penny precision without floating-point errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Pod, Zeroable, Serialize, Deserialize)]
#[repr(C)]
pub struct Price {
    /// Price in micro-units (1 SOL = 1,000,000 micro-SOL)
    pub micro_units: u64,
}

impl Price {
    pub const PRECISION: u64 = 1_000_000;
    
    pub fn from_sol(sol: f64) -> Self {
        Self {
            micro_units: (sol * Self::PRECISION as f64) as u64,
        }
    }
    
    pub fn to_sol(&self) -> f64 {
        self.micro_units as f64 / Self::PRECISION as f64
    }
    
    pub fn zero() -> Self {
        Self { micro_units: 0 }
    }
}

/// Compact order book entry for cache efficiency
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct OrderBookEntry {
    pub price: Price,
    pub quantity: u64,
    pub timestamp: u64,
}

/// High-performance order book with SIMD-friendly layout
#[derive(Debug, Clone)]
pub struct OrderBook {
    pub symbol_id: u32,
    pub bids: Vec<OrderBookEntry>,
    pub asks: Vec<OrderBookEntry>,
    pub last_update: u64,
    pub sequence: u64,
}

impl OrderBook {
    pub fn new(symbol_id: u32) -> Self {
        Self {
            symbol_id,
            bids: Vec::with_capacity(crate::core::MAX_ORDER_BOOK_DEPTH),
            asks: Vec::with_capacity(crate::core::MAX_ORDER_BOOK_DEPTH),
            last_update: current_timestamp(),
            sequence: 0,
        }
    }
    
    pub fn best_bid(&self) -> Option<Price> {
        self.bids.first().map(|entry| entry.price)
    }
    
    pub fn best_ask(&self) -> Option<Price> {
        self.asks.first().map(|entry| entry.price)
    }
    
    pub fn spread(&self) -> Option<Price> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(Price {
                micro_units: ask.micro_units.saturating_sub(bid.micro_units),
            }),
            _ => None,
        }
    }
}

/// Trade execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: u64,
    pub symbol_id: u32,
    pub side: TradeSide,
    pub price: Price,
    pub quantity: u64,
    pub timestamp: u64,
    pub strategy: String,
    pub execution_time_ns: u64,
    pub fees: Price,
    pub slippage: Price,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TradeSide {
    Buy,
    Sell,
}

/// Position tracking with P&L calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol_id: u32,
    pub quantity: i64, // Positive = long, negative = short
    pub average_price: Price,
    pub unrealized_pnl: Price,
    pub realized_pnl: Price,
    pub last_update: u64,
}

impl Position {
    pub fn new(symbol_id: u32) -> Self {
        Self {
            symbol_id,
            quantity: 0,
            average_price: Price::zero(),
            unrealized_pnl: Price::zero(),
            realized_pnl: Price::zero(),
            last_update: current_timestamp(),
        }
    }
    
    pub fn update_unrealized_pnl(&mut self, current_price: Price) {
        if self.quantity != 0 {
            let price_diff = if self.quantity > 0 {
                current_price.micro_units.saturating_sub(self.average_price.micro_units)
            } else {
                self.average_price.micro_units.saturating_sub(current_price.micro_units)
            };
            
            self.unrealized_pnl = Price {
                micro_units: (price_diff * self.quantity.abs() as u64),
            };
        }
        self.last_update = current_timestamp();
    }
}

/// Account balance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub symbol_id: u32,
    pub available: Price,
    pub locked: Price,
    pub total: Price,
    pub last_update: u64,
}

impl Balance {
    pub fn new(symbol_id: u32, amount: Price) -> Self {
        Self {
            symbol_id,
            available: amount,
            locked: Price::zero(),
            total: amount,
            last_update: current_timestamp(),
        }
    }
}

/// Market data snapshot
#[derive(Debug, Clone)]
pub struct MarketData {
    pub symbol_id: u32,
    pub price: Price,
    pub volume_24h: u64,
    pub price_change_24h: Price,
    pub high_24h: Price,
    pub low_24h: Price,
    pub timestamp: u64,
}

/// Trading signal from strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSignal {
    pub symbol_id: u32,
    pub action: SignalAction,
    pub confidence: f32, // 0.0 to 1.0
    pub target_price: Option<Price>,
    pub quantity: Option<u64>,
    pub strategy: String,
    pub timestamp: u64,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SignalAction {
    Buy,
    Sell,
    Hold,
    StopLoss,
    TakeProfit,
}

/// Risk metrics for position management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub var_95: Price,        // Value at Risk (95% confidence)
    pub var_99: Price,        // Value at Risk (99% confidence)
    pub max_drawdown: Price,  // Maximum drawdown
    pub sharpe_ratio: f64,    // Risk-adjusted return
    pub volatility: f64,      // Price volatility
    pub beta: f64,            // Market correlation
    pub last_update: u64,
}

/// Order execution parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderParams {
    pub symbol_id: u32,
    pub side: TradeSide,
    pub order_type: OrderType,
    pub quantity: u64,
    pub price: Option<Price>,
    pub time_in_force: TimeInForce,
    pub max_slippage: Option<Price>,
    pub strategy: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TimeInForce {
    GTC, // Good Till Cancelled
    IOC, // Immediate Or Cancel
    FOK, // Fill Or Kill
}

/// Performance metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub latency_ns: u64,
    pub throughput_ops_sec: f64,
    pub memory_usage_bytes: u64,
    pub cpu_usage_percent: f64,
    pub error_rate: f64,
    pub timestamp: u64,
}

/// Utility function to get current timestamp in nanoseconds
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

/// Utility function to get current timestamp in microseconds
pub fn current_timestamp_micros() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_precision() {
        let price = Price::from_sol(23.456789);
        assert_eq!(price.to_sol(), 23.456789);
        
        let zero = Price::zero();
        assert_eq!(zero.to_sol(), 0.0);
    }

    #[test]
    fn test_order_book() {
        let mut book = OrderBook::new(1);
        
        book.bids.push(OrderBookEntry {
            price: Price::from_sol(23.45),
            quantity: 100,
            timestamp: current_timestamp(),
        });
        
        book.asks.push(OrderBookEntry {
            price: Price::from_sol(23.50),
            quantity: 150,
            timestamp: current_timestamp(),
        });
        
        assert_eq!(book.best_bid().unwrap().to_sol(), 23.45);
        assert_eq!(book.best_ask().unwrap().to_sol(), 23.50);
        assert_eq!(book.spread().unwrap().to_sol(), 0.05);
    }

    #[test]
    fn test_position_pnl() {
        let mut position = Position::new(1);
        position.quantity = 100;
        position.average_price = Price::from_sol(23.00);
        
        position.update_unrealized_pnl(Price::from_sol(23.50));
        assert_eq!(position.unrealized_pnl.to_sol(), 50.0);
    }
}
