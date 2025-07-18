// 🥷 Event System - Zero-Copy, Lock-Free Communication
// High-performance event bus for sub-microsecond message passing

use crate::core::types::{Price, OrderBook, Trade, Position, TradingSignal, MarketData};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use anyhow::Result;
use tracing::{debug, warn};

/// Core event types for the trading system
#[derive(Debug, Clone)]
pub enum Event {
    // Market data events
    PriceUpdate {
        symbol_id: u32,
        price: Price,
        volume: u64,
        timestamp: u64,
    },
    OrderBookUpdate {
        order_book: Arc<OrderBook>,
    },
    MarketDataSnapshot {
        data: Arc<MarketData>,
    },
    
    // Trading events
    TradeExecuted {
        trade: Arc<Trade>,
    },
    PositionUpdate {
        position: Arc<Position>,
    },
    TradingSignal {
        signal: Arc<TradingSignal>,
    },
    
    // System events
    StrategyStarted {
        strategy_name: String,
        timestamp: u64,
    },
    StrategyError {
        strategy_name: String,
        error: String,
        timestamp: u64,
    },
    SystemShutdown {
        timestamp: u64,
    },
    
    // Risk management events
    RiskLimitExceeded {
        symbol_id: u32,
        limit_type: String,
        current_value: f64,
        limit_value: f64,
        timestamp: u64,
    },
    
    // Performance events
    LatencyAlert {
        component: String,
        latency_ns: u64,
        threshold_ns: u64,
        timestamp: u64,
    },
}

/// Event handler trait for processing events
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle_event(&self, event: &Event) -> Result<()>;
    fn name(&self) -> &str;
    fn interested_events(&self) -> Vec<EventType>;
}

/// Event type enumeration for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    PriceUpdate,
    OrderBookUpdate,
    MarketDataSnapshot,
    TradeExecuted,
    PositionUpdate,
    TradingSignal,
    StrategyStarted,
    StrategyError,
    SystemShutdown,
    RiskLimitExceeded,
    LatencyAlert,
}

impl Event {
    pub fn event_type(&self) -> EventType {
        match self {
            Event::PriceUpdate { .. } => EventType::PriceUpdate,
            Event::OrderBookUpdate { .. } => EventType::OrderBookUpdate,
            Event::MarketDataSnapshot { .. } => EventType::MarketDataSnapshot,
            Event::TradeExecuted { .. } => EventType::TradeExecuted,
            Event::PositionUpdate { .. } => EventType::PositionUpdate,
            Event::TradingSignal { .. } => EventType::TradingSignal,
            Event::StrategyStarted { .. } => EventType::StrategyStarted,
            Event::StrategyError { .. } => EventType::StrategyError,
            Event::SystemShutdown { .. } => EventType::SystemShutdown,
            Event::RiskLimitExceeded { .. } => EventType::RiskLimitExceeded,
            Event::LatencyAlert { .. } => EventType::LatencyAlert,
        }
    }
    
    pub fn timestamp(&self) -> u64 {
        match self {
            Event::PriceUpdate { timestamp, .. } => *timestamp,
            Event::OrderBookUpdate { order_book } => order_book.last_update,
            Event::MarketDataSnapshot { data } => data.timestamp,
            Event::TradeExecuted { trade } => trade.timestamp,
            Event::PositionUpdate { position } => position.last_update,
            Event::TradingSignal { signal } => signal.timestamp,
            Event::StrategyStarted { timestamp, .. } => *timestamp,
            Event::StrategyError { timestamp, .. } => *timestamp,
            Event::SystemShutdown { timestamp } => *timestamp,
            Event::RiskLimitExceeded { timestamp, .. } => *timestamp,
            Event::LatencyAlert { timestamp, .. } => *timestamp,
        }
    }
}

/// High-performance event bus with multiple delivery modes
pub struct EventBus {
    // Broadcast channel for real-time events (low latency)
    broadcast_tx: broadcast::Sender<Event>,
    
    // MPSC channels for specific handlers (high throughput)
    handler_channels: Vec<mpsc::UnboundedSender<Event>>,
    
    // Event statistics
    events_sent: std::sync::atomic::AtomicU64,
    events_dropped: std::sync::atomic::AtomicU64,
}

impl EventBus {
    pub fn new(buffer_size: usize) -> Self {
        let (broadcast_tx, _) = broadcast::channel(buffer_size);
        
        Self {
            broadcast_tx,
            handler_channels: Vec::new(),
            events_sent: std::sync::atomic::AtomicU64::new(0),
            events_dropped: std::sync::atomic::AtomicU64::new(0),
        }
    }
    
    /// Publish event to all subscribers
    pub fn publish(&self, event: Event) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        // Send to broadcast channel (for real-time subscribers)
        match self.broadcast_tx.send(event.clone()) {
            Ok(_) => {
                self.events_sent.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            Err(_) => {
                self.events_dropped.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                warn!("Event dropped: no broadcast subscribers");
            }
        }
        
        // Send to dedicated handler channels
        for channel in &self.handler_channels {
            if let Err(_) = channel.send(event.clone()) {
                self.events_dropped.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                warn!("Event dropped: handler channel full");
            }
        }
        
        // Log latency if it exceeds threshold
        let latency = start_time.elapsed();
        if latency.as_nanos() > 1000 { // 1 microsecond threshold
            debug!("Event publish latency: {}ns", latency.as_nanos());
        }
        
        Ok(())
    }
    
    /// Subscribe to events with broadcast receiver
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.broadcast_tx.subscribe()
    }
    
    /// Register a dedicated event handler
    pub fn register_handler(&mut self, handler: Arc<dyn EventHandler>) {
        let (tx, rx) = mpsc::unbounded_channel();
        self.handler_channels.push(tx);

        // Spawn handler task
        let handler_name = handler.name().to_string();
        let interested_events = handler.interested_events();

        tokio::spawn(async move {
            let mut receiver = rx;
            while let Some(event) = receiver.recv().await {
                // Filter events based on handler interest
                if interested_events.contains(&event.event_type()) {
                    if let Err(e) = handler.handle_event(&event).await {
                        warn!("Handler '{}' error: {}", handler_name, e);
                    }
                }
            }
        });
    }
    
    /// Get event statistics
    pub fn stats(&self) -> EventBusStats {
        EventBusStats {
            events_sent: self.events_sent.load(std::sync::atomic::Ordering::Relaxed),
            events_dropped: self.events_dropped.load(std::sync::atomic::Ordering::Relaxed),
            active_subscribers: self.broadcast_tx.receiver_count(),
            handler_count: self.handler_channels.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventBusStats {
    pub events_sent: u64,
    pub events_dropped: u64,
    pub active_subscribers: usize,
    pub handler_count: usize,
}

/// Event filter for selective subscription
pub struct EventFilter {
    event_types: Vec<EventType>,
    symbol_ids: Option<Vec<u32>>,
}

impl EventFilter {
    pub fn new() -> Self {
        Self {
            event_types: Vec::new(),
            symbol_ids: None,
        }
    }
    
    pub fn with_event_types(mut self, types: Vec<EventType>) -> Self {
        self.event_types = types;
        self
    }
    
    pub fn with_symbols(mut self, symbol_ids: Vec<u32>) -> Self {
        self.symbol_ids = Some(symbol_ids);
        self
    }
    
    pub fn matches(&self, event: &Event) -> bool {
        // Check event type
        if !self.event_types.is_empty() && !self.event_types.contains(&event.event_type()) {
            return false;
        }
        
        // Check symbol filter
        if let Some(ref symbol_ids) = self.symbol_ids {
            let event_symbol = match event {
                Event::PriceUpdate { symbol_id, .. } => Some(*symbol_id),
                Event::OrderBookUpdate { order_book } => Some(order_book.symbol_id),
                Event::MarketDataSnapshot { data } => Some(data.symbol_id),
                Event::TradeExecuted { trade } => Some(trade.symbol_id),
                Event::PositionUpdate { position } => Some(position.symbol_id),
                Event::TradingSignal { signal } => Some(signal.symbol_id),
                Event::RiskLimitExceeded { symbol_id, .. } => Some(*symbol_id),
                _ => None,
            };
            
            if let Some(symbol_id) = event_symbol {
                if !symbol_ids.contains(&symbol_id) {
                    return false;
                }
            }
        }
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::current_timestamp;

    #[tokio::test]
    async fn test_event_bus() {
        let mut bus = EventBus::new(1000);
        let mut receiver = bus.subscribe();
        
        let event = Event::PriceUpdate {
            symbol_id: 1,
            price: Price::from_sol(23.45),
            volume: 1000,
            timestamp: current_timestamp(),
        };
        
        bus.publish(event.clone()).unwrap();
        
        let received = receiver.recv().await.unwrap();
        assert_eq!(received.event_type(), EventType::PriceUpdate);
    }

    #[test]
    fn test_event_filter() {
        let filter = EventFilter::new()
            .with_event_types(vec![EventType::PriceUpdate])
            .with_symbols(vec![1, 2]);
        
        let event1 = Event::PriceUpdate {
            symbol_id: 1,
            price: Price::from_sol(23.45),
            volume: 1000,
            timestamp: current_timestamp(),
        };
        
        let event2 = Event::PriceUpdate {
            symbol_id: 3,
            price: Price::from_sol(24.00),
            volume: 1000,
            timestamp: current_timestamp(),
        };
        
        assert!(filter.matches(&event1));
        assert!(!filter.matches(&event2));
    }
}
