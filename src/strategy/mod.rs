use crate::{config::StrategyConfig, types::MarketSnapshot};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Strategy: Send + Sync {
    async fn generate_orders(&self, market_snapshot: &MarketSnapshot) -> Result<Vec<Order>>;
    fn name(&self) -> &'static str;
}

#[derive(Debug)]
pub struct Order {
    pub side: OrderSide,
    pub size: f64,
    pub price: f64,
    pub market: String,
}

#[derive(Debug)]
pub enum OrderSide {
    Buy,
    Sell,
}

pub fn create_strategy(config: &StrategyConfig) -> Result<Box<dyn Strategy>> {
    match config.strategy_mode.as_str() {
        "market_making" => Ok(Box::new(MarketMakingStrategy::new(config))),
        "arbitrage" => Ok(Box::new(ArbitrageStrategy::new(config))),
        _ => Err(anyhow::anyhow!("Unknown strategy: {}", config.strategy_mode)),
    }
}

pub struct MarketMakingStrategy {
    config: StrategyConfig,
}

impl MarketMakingStrategy {
    pub fn new(config: &StrategyConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

#[async_trait]
impl Strategy for MarketMakingStrategy {
    async fn generate_orders(&self, _market_snapshot: &MarketSnapshot) -> Result<Vec<Order>> {
        // Placeholder for market making logic
        Ok(vec![])
    }

    fn name(&self) -> &'static str {
        "market_making"
    }
}

pub struct ArbitrageStrategy {
    config: StrategyConfig,
}

impl ArbitrageStrategy {
    pub fn new(config: &StrategyConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

#[async_trait]
impl Strategy for ArbitrageStrategy {
    async fn generate_orders(&self, _market_snapshot: &MarketSnapshot) -> Result<Vec<Order>> {
        // Placeholder for arbitrage logic
        Ok(vec![])
    }

    fn name(&self) -> &'static str {
        "arbitrage"
    }
}