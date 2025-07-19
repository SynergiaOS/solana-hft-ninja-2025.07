//! ⚡ Lazy Feature Extraction - High-Performance Technical Analysis
//! 
//! Pre-computed technical indicators using Rust + Arrow IPC for zero-copy serialization

use anyhow::Result;
use arrow::array::{Float64Array, UInt64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use arrow::ipc::writer::StreamWriter;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task;
use tracing::{info, debug};

/// Raw market tick data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTick {
    pub timestamp: u64,
    pub price: f64,
    pub volume: f64,
    pub token_mint: String,
    pub dex: String,
}

/// Pre-computed technical indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalIndicators {
    pub timestamp: u64,
    pub token_mint: String,
    
    // Price indicators
    pub ema_5: f64,
    pub ema_20: f64,
    pub ema_50: f64,
    pub sma_20: f64,
    pub bollinger_upper: f64,
    pub bollinger_lower: f64,
    pub rsi: f64,
    
    // Volume indicators
    pub volume_ema: f64,
    pub obv: f64, // On-Balance Volume
    pub volume_ratio: f64,
    
    // Volatility indicators
    pub atr: f64, // Average True Range
    pub volatility: f64,
    pub price_change_1m: f64,
    pub price_change_5m: f64,
    pub price_change_15m: f64,
    
    // Momentum indicators
    pub macd: f64,
    pub macd_signal: f64,
    pub momentum: f64,
    
    // Custom MEV indicators
    pub liquidity_score: f64,
    pub arbitrage_potential: f64,
    pub sandwich_vulnerability: f64,
}

/// Feature extraction configuration
#[derive(Debug, Clone)]
pub struct FeatureConfig {
    pub ema_periods: Vec<usize>,
    pub sma_periods: Vec<usize>,
    pub rsi_period: usize,
    pub bollinger_period: usize,
    pub bollinger_std: f64,
    pub atr_period: usize,
    pub macd_fast: usize,
    pub macd_slow: usize,
    pub macd_signal: usize,
    pub parallel_workers: usize,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            ema_periods: vec![5, 20, 50],
            sma_periods: vec![20],
            rsi_period: 14,
            bollinger_period: 20,
            bollinger_std: 2.0,
            atr_period: 14,
            macd_fast: 12,
            macd_slow: 26,
            macd_signal: 9,
            parallel_workers: num_cpus::get(),
        }
    }
}

/// High-performance feature extractor
pub struct LazyFeatureExtractor {
    config: FeatureConfig,
    tick_buffer: HashMap<String, Vec<MarketTick>>, // token_mint -> ticks
    indicator_cache: HashMap<String, TechnicalIndicators>,
}

impl LazyFeatureExtractor {
    pub fn new(config: FeatureConfig) -> Self {
        Self {
            config,
            tick_buffer: HashMap::new(),
            indicator_cache: HashMap::new(),
        }
    }

    /// Add market tick and trigger lazy computation
    pub async fn add_tick(&mut self, tick: MarketTick) -> Result<()> {
        let token_mint = tick.token_mint.clone();
        
        // Add to buffer
        self.tick_buffer
            .entry(token_mint.clone())
            .or_insert_with(Vec::new)
            .push(tick);

        // Limit buffer size (keep last 1000 ticks)
        if let Some(ticks) = self.tick_buffer.get_mut(&token_mint) {
            if ticks.len() > 1000 {
                ticks.drain(0..ticks.len() - 1000);
            }
        }

        // Trigger async computation if enough data
        if self.tick_buffer.get(&token_mint).unwrap().len() >= 50 {
            self.compute_indicators_async(&token_mint).await?;
        }

        Ok(())
    }

    /// Compute technical indicators asynchronously
    async fn compute_indicators_async(&mut self, token_mint: &str) -> Result<()> {
        let ticks = self.tick_buffer.get(token_mint).unwrap().clone();
        let config = self.config.clone();
        let token_mint = token_mint.to_string();

        // Spawn computation on thread pool
        let indicators = task::spawn_blocking(move || {
            Self::compute_indicators_parallel(&ticks, &config)
        }).await??;

        // Cache results
        self.indicator_cache.insert(token_mint.clone(), indicators);
        
        debug!("Computed indicators for token: {}", token_mint);
        Ok(())
    }

    /// Parallel computation of technical indicators
    fn compute_indicators_parallel(
        ticks: &[MarketTick], 
        config: &FeatureConfig
    ) -> Result<TechnicalIndicators> {
        if ticks.is_empty() {
            return Err(anyhow::anyhow!("No ticks provided"));
        }

        let prices: Vec<f64> = ticks.iter().map(|t| t.price).collect();
        let volumes: Vec<f64> = ticks.iter().map(|t| t.volume).collect();
        let latest_tick = ticks.last().unwrap();

        // Parallel computation of different indicator groups
        let (price_indicators, volume_indicators, volatility_indicators, momentum_indicators) = rayon::join4(
            || Self::compute_price_indicators(&prices, config),
            || Self::compute_volume_indicators(&volumes, config),
            || Self::compute_volatility_indicators(&prices, config),
            || Self::compute_momentum_indicators(&prices, config),
        );

        // Custom MEV indicators
        let mev_indicators = Self::compute_mev_indicators(ticks, config);

        Ok(TechnicalIndicators {
            timestamp: latest_tick.timestamp,
            token_mint: latest_tick.token_mint.clone(),
            
            // Price indicators
            ema_5: price_indicators.0,
            ema_20: price_indicators.1,
            ema_50: price_indicators.2,
            sma_20: price_indicators.3,
            bollinger_upper: price_indicators.4,
            bollinger_lower: price_indicators.5,
            rsi: price_indicators.6,
            
            // Volume indicators
            volume_ema: volume_indicators.0,
            obv: volume_indicators.1,
            volume_ratio: volume_indicators.2,
            
            // Volatility indicators
            atr: volatility_indicators.0,
            volatility: volatility_indicators.1,
            price_change_1m: volatility_indicators.2,
            price_change_5m: volatility_indicators.3,
            price_change_15m: volatility_indicators.4,
            
            // Momentum indicators
            macd: momentum_indicators.0,
            macd_signal: momentum_indicators.1,
            momentum: momentum_indicators.2,
            
            // MEV indicators
            liquidity_score: mev_indicators.0,
            arbitrage_potential: mev_indicators.1,
            sandwich_vulnerability: mev_indicators.2,
        })
    }

    /// Compute price-based indicators
    fn compute_price_indicators(prices: &[f64], config: &FeatureConfig) -> (f64, f64, f64, f64, f64, f64, f64) {
        let ema_5 = Self::ema(prices, 5);
        let ema_20 = Self::ema(prices, 20);
        let ema_50 = Self::ema(prices, 50);
        let sma_20 = Self::sma(prices, 20);
        let (bb_upper, bb_lower) = Self::bollinger_bands(prices, config.bollinger_period, config.bollinger_std);
        let rsi = Self::rsi(prices, config.rsi_period);
        
        (ema_5, ema_20, ema_50, sma_20, bb_upper, bb_lower, rsi)
    }

    /// Compute volume-based indicators
    fn compute_volume_indicators(volumes: &[f64], _config: &FeatureConfig) -> (f64, f64, f64) {
        let volume_ema = Self::ema(volumes, 20);
        let obv = Self::on_balance_volume(volumes);
        let volume_ratio = if volumes.len() >= 20 {
            let recent_avg = volumes[volumes.len()-10..].iter().sum::<f64>() / 10.0;
            let historical_avg = volumes[volumes.len()-20..volumes.len()-10].iter().sum::<f64>() / 10.0;
            if historical_avg > 0.0 { recent_avg / historical_avg } else { 1.0 }
        } else { 1.0 };
        
        (volume_ema, obv, volume_ratio)
    }

    /// Compute volatility indicators
    fn compute_volatility_indicators(prices: &[f64], config: &FeatureConfig) -> (f64, f64, f64, f64, f64) {
        let atr = Self::atr(prices, config.atr_period);
        let volatility = Self::volatility(prices, 20);
        
        let price_change_1m = Self::price_change(prices, 1);
        let price_change_5m = Self::price_change(prices, 5);
        let price_change_15m = Self::price_change(prices, 15);
        
        (atr, volatility, price_change_1m, price_change_5m, price_change_15m)
    }

    /// Compute momentum indicators
    fn compute_momentum_indicators(prices: &[f64], config: &FeatureConfig) -> (f64, f64, f64) {
        let (macd, macd_signal) = Self::macd(prices, config.macd_fast, config.macd_slow, config.macd_signal);
        let momentum = Self::momentum(prices, 10);
        
        (macd, macd_signal, momentum)
    }

    /// Compute MEV-specific indicators
    fn compute_mev_indicators(ticks: &[MarketTick], _config: &FeatureConfig) -> (f64, f64, f64) {
        let liquidity_score = Self::calculate_liquidity_score(ticks);
        let arbitrage_potential = Self::calculate_arbitrage_potential(ticks);
        let sandwich_vulnerability = Self::calculate_sandwich_vulnerability(ticks);
        
        (liquidity_score, arbitrage_potential, sandwich_vulnerability)
    }

    /// Exponential Moving Average
    fn ema(prices: &[f64], period: usize) -> f64 {
        if prices.len() < period {
            return prices.last().copied().unwrap_or(0.0);
        }

        let alpha = 2.0 / (period as f64 + 1.0);
        let mut ema = prices[0];
        
        for &price in &prices[1..] {
            ema = alpha * price + (1.0 - alpha) * ema;
        }
        
        ema
    }

    /// Simple Moving Average
    fn sma(prices: &[f64], period: usize) -> f64 {
        if prices.len() < period {
            return prices.iter().sum::<f64>() / prices.len() as f64;
        }
        
        prices[prices.len() - period..].iter().sum::<f64>() / period as f64
    }

    /// RSI (Relative Strength Index)
    fn rsi(prices: &[f64], period: usize) -> f64 {
        if prices.len() < period + 1 {
            return 50.0; // Neutral RSI
        }

        let mut gains = 0.0;
        let mut losses = 0.0;

        for i in 1..=period {
            let change = prices[prices.len() - i] - prices[prices.len() - i - 1];
            if change > 0.0 {
                gains += change;
            } else {
                losses -= change;
            }
        }

        let avg_gain = gains / period as f64;
        let avg_loss = losses / period as f64;

        if avg_loss == 0.0 {
            return 100.0;
        }

        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }

    /// Bollinger Bands
    fn bollinger_bands(prices: &[f64], period: usize, std_dev: f64) -> (f64, f64) {
        let sma = Self::sma(prices, period);
        let variance = if prices.len() >= period {
            let recent_prices = &prices[prices.len() - period..];
            recent_prices.iter()
                .map(|&p| (p - sma).powi(2))
                .sum::<f64>() / period as f64
        } else {
            0.0
        };
        
        let std = variance.sqrt();
        (sma + std_dev * std, sma - std_dev * std)
    }

    /// Average True Range
    fn atr(prices: &[f64], period: usize) -> f64 {
        if prices.len() < 2 {
            return 0.0;
        }

        let mut true_ranges = Vec::new();
        for i in 1..prices.len() {
            let high_low = (prices[i] - prices[i-1]).abs();
            true_ranges.push(high_low);
        }

        Self::sma(&true_ranges, period.min(true_ranges.len()))
    }

    /// Price volatility
    fn volatility(prices: &[f64], period: usize) -> f64 {
        if prices.len() < period {
            return 0.0;
        }

        let recent_prices = &prices[prices.len() - period..];
        let mean = recent_prices.iter().sum::<f64>() / period as f64;
        let variance = recent_prices.iter()
            .map(|&p| (p - mean).powi(2))
            .sum::<f64>() / period as f64;
        
        variance.sqrt()
    }

    /// Price change percentage
    fn price_change(prices: &[f64], periods_back: usize) -> f64 {
        if prices.len() <= periods_back {
            return 0.0;
        }

        let current = prices[prices.len() - 1];
        let previous = prices[prices.len() - 1 - periods_back];
        
        if previous != 0.0 {
            (current - previous) / previous * 100.0
        } else {
            0.0
        }
    }

    /// MACD (Moving Average Convergence Divergence)
    fn macd(prices: &[f64], fast: usize, slow: usize, signal: usize) -> (f64, f64) {
        let ema_fast = Self::ema(prices, fast);
        let ema_slow = Self::ema(prices, slow);
        let macd_line = ema_fast - ema_slow;
        
        // For signal line, we'd need historical MACD values
        // Simplified: use current MACD as signal
        let signal_line = macd_line * 0.9; // Approximation
        
        (macd_line, signal_line)
    }

    /// Momentum
    fn momentum(prices: &[f64], period: usize) -> f64 {
        if prices.len() <= period {
            return 0.0;
        }

        let current = prices[prices.len() - 1];
        let previous = prices[prices.len() - 1 - period];
        
        current - previous
    }

    /// On-Balance Volume
    fn on_balance_volume(volumes: &[f64]) -> f64 {
        volumes.iter().sum()
    }

    /// Calculate liquidity score for MEV
    fn calculate_liquidity_score(ticks: &[MarketTick]) -> f64 {
        if ticks.is_empty() {
            return 0.0;
        }

        let total_volume: f64 = ticks.iter().map(|t| t.volume).sum();
        let avg_volume = total_volume / ticks.len() as f64;
        
        // Normalize to 0-1 scale
        (avg_volume / 1000.0).min(1.0)
    }

    /// Calculate arbitrage potential
    fn calculate_arbitrage_potential(ticks: &[MarketTick]) -> f64 {
        if ticks.len() < 2 {
            return 0.0;
        }

        // Group by DEX and calculate price differences
        let mut dex_prices: HashMap<String, Vec<f64>> = HashMap::new();
        for tick in ticks {
            dex_prices.entry(tick.dex.clone()).or_default().push(tick.price);
        }

        if dex_prices.len() < 2 {
            return 0.0;
        }

        let mut max_spread = 0.0;
        let dex_avg_prices: Vec<(String, f64)> = dex_prices.iter()
            .map(|(dex, prices)| (dex.clone(), prices.iter().sum::<f64>() / prices.len() as f64))
            .collect();

        for i in 0..dex_avg_prices.len() {
            for j in i+1..dex_avg_prices.len() {
                let spread = (dex_avg_prices[i].1 - dex_avg_prices[j].1).abs() / dex_avg_prices[i].1;
                max_spread = max_spread.max(spread);
            }
        }

        max_spread
    }

    /// Calculate sandwich vulnerability
    fn calculate_sandwich_vulnerability(ticks: &[MarketTick]) -> f64 {
        if ticks.len() < 10 {
            return 0.0;
        }

        // Look for large volume spikes that could be sandwiched
        let volumes: Vec<f64> = ticks.iter().map(|t| t.volume).collect();
        let avg_volume = volumes.iter().sum::<f64>() / volumes.len() as f64;
        
        let large_trades = volumes.iter()
            .filter(|&&v| v > avg_volume * 3.0)
            .count();

        (large_trades as f64 / volumes.len() as f64).min(1.0)
    }

    /// Serialize indicators to Arrow IPC format
    pub fn serialize_to_arrow(&self, token_mint: &str) -> Result<Vec<u8>> {
        let indicators = self.indicator_cache.get(token_mint)
            .ok_or_else(|| anyhow::anyhow!("No indicators found for token: {}", token_mint))?;

        // Create Arrow schema
        let schema = Arc::new(Schema::new(vec![
            Field::new("timestamp", DataType::UInt64, false),
            Field::new("ema_5", DataType::Float64, false),
            Field::new("ema_20", DataType::Float64, false),
            Field::new("rsi", DataType::Float64, false),
            Field::new("volume_ratio", DataType::Float64, false),
            Field::new("arbitrage_potential", DataType::Float64, false),
        ]));

        // Create arrays
        let timestamp_array = UInt64Array::from(vec![indicators.timestamp]);
        let ema_5_array = Float64Array::from(vec![indicators.ema_5]);
        let ema_20_array = Float64Array::from(vec![indicators.ema_20]);
        let rsi_array = Float64Array::from(vec![indicators.rsi]);
        let volume_ratio_array = Float64Array::from(vec![indicators.volume_ratio]);
        let arbitrage_array = Float64Array::from(vec![indicators.arbitrage_potential]);

        // Create record batch
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(timestamp_array),
                Arc::new(ema_5_array),
                Arc::new(ema_20_array),
                Arc::new(rsi_array),
                Arc::new(volume_ratio_array),
                Arc::new(arbitrage_array),
            ],
        )?;

        // Serialize to IPC
        let mut buffer = Vec::new();
        {
            let mut writer = StreamWriter::try_new(&mut buffer, &schema)?;
            writer.write(&batch)?;
            writer.finish()?;
        }

        Ok(buffer)
    }

    /// Get latest indicators for token
    pub fn get_indicators(&self, token_mint: &str) -> Option<&TechnicalIndicators> {
        self.indicator_cache.get(token_mint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ema_calculation() {
        let prices = vec![10.0, 11.0, 12.0, 11.5, 13.0];
        let ema = LazyFeatureExtractor::ema(&prices, 3);
        assert!(ema > 11.0 && ema < 13.0);
    }

    #[test]
    fn test_rsi_calculation() {
        let prices = vec![44.0, 44.25, 44.5, 43.75, 44.5, 44.75, 44.5, 44.25, 44.0, 44.25];
        let rsi = LazyFeatureExtractor::rsi(&prices, 5);
        assert!(rsi >= 0.0 && rsi <= 100.0);
    }

    #[tokio::test]
    async fn test_feature_extraction() {
        let config = FeatureConfig::default();
        let mut extractor = LazyFeatureExtractor::new(config);

        for i in 0..60 {
            let tick = MarketTick {
                timestamp: 1640995200 + i * 60,
                price: 100.0 + (i as f64 * 0.1),
                volume: 1000.0 + (i as f64 * 10.0),
                token_mint: "test_token".to_string(),
                dex: "raydium".to_string(),
            };
            
            extractor.add_tick(tick).await.unwrap();
        }

        let indicators = extractor.get_indicators("test_token");
        assert!(indicators.is_some());
        
        let indicators = indicators.unwrap();
        assert!(indicators.ema_5 > 0.0);
        assert!(indicators.rsi >= 0.0 && indicators.rsi <= 100.0);
    }
}
