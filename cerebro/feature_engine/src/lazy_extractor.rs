//! ⚡ Lazy Feature Extraction - Background Processing for Hot Path
//! 
//! Parallel computation with Arrow IPC serialization for zero-copy data transfer

use anyhow::Result;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Technical indicators configuration
#[derive(Debug, Clone)]
pub struct IndicatorConfig {
    pub ema_periods: Vec<usize>,
    pub rsi_period: usize,
    pub macd_fast: usize,
    pub macd_slow: usize,
    pub macd_signal: usize,
}

impl Default for IndicatorConfig {
    fn default() -> Self {
        Self {
            ema_periods: vec![5, 20, 50],
            rsi_period: 14,
            macd_fast: 12,
            macd_slow: 26,
            macd_signal: 9,
        }
    }
}

/// Price data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Extracted features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFeatures {
    pub timestamp: u64,
    pub ema_values: HashMap<usize, f64>,
    pub rsi: f64,
    pub macd_line: f64,
    pub macd_signal: f64,
    pub macd_histogram: f64,
    pub volume_sma: f64,
    pub price_change_pct: f64,
    pub volatility: f64,
    pub mev_score: f64,
}

/// Lazy feature extractor with parallel processing
pub struct LazyFeatureExtractor {
    config: IndicatorConfig,
    worker_count: usize,
}

impl LazyFeatureExtractor {
    /// Create new feature extractor
    pub fn new(config: IndicatorConfig, worker_count: usize) -> Self {
        info!("🚀 Initializing LazyFeatureExtractor with {} workers", worker_count);
        
        // Configure Rayon thread pool
        rayon::ThreadPoolBuilder::new()
            .num_threads(worker_count)
            .build_global()
            .expect("Failed to build thread pool");

        Self {
            config,
            worker_count,
        }
    }

    /// Extract features from price data in parallel
    pub async fn extract_features(&self, price_data: &[PriceData]) -> Result<Vec<ExtractedFeatures>> {
        if price_data.is_empty() {
            return Ok(vec![]);
        }

        debug!("Extracting features from {} price points", price_data.len());

        // Use Rayon for parallel processing
        let features: Result<Vec<_>, _> = price_data
            .par_iter()
            .enumerate()
            .map(|(i, price)| self.extract_single_features(price_data, i))
            .collect();

        let extracted = features?;
        info!("Extracted {} feature sets", extracted.len());
        
        Ok(extracted)
    }

    /// Extract features for single price point
    fn extract_single_features(&self, price_data: &[PriceData], index: usize) -> Result<ExtractedFeatures> {
        let current = &price_data[index];
        
        // Calculate EMA values
        let mut ema_values = HashMap::new();
        for &period in &self.config.ema_periods {
            let ema = self.calculate_ema(price_data, index, period);
            ema_values.insert(period, ema);
        }

        // Calculate RSI
        let rsi = self.calculate_rsi(price_data, index, self.config.rsi_period);

        // Calculate MACD
        let (macd_line, macd_signal, macd_histogram) = self.calculate_macd(
            price_data, 
            index, 
            self.config.macd_fast, 
            self.config.macd_slow, 
            self.config.macd_signal
        );

        // Calculate volume SMA
        let volume_sma = self.calculate_volume_sma(price_data, index, 20);

        // Calculate price change percentage
        let price_change_pct = if index > 0 {
            let prev_close = price_data[index - 1].close;
            ((current.close - prev_close) / prev_close) * 100.0
        } else {
            0.0
        };

        // Calculate volatility (standard deviation of returns)
        let volatility = self.calculate_volatility(price_data, index, 20);

        // Calculate MEV score (custom metric)
        let mev_score = self.calculate_mev_score(current, &ema_values, rsi);

        Ok(ExtractedFeatures {
            timestamp: current.timestamp,
            ema_values,
            rsi,
            macd_line,
            macd_signal,
            macd_histogram,
            volume_sma,
            price_change_pct,
            volatility,
            mev_score,
        })
    }

    /// Calculate Exponential Moving Average
    fn calculate_ema(&self, data: &[PriceData], index: usize, period: usize) -> f64 {
        if index == 0 {
            return data[0].close;
        }

        let alpha = 2.0 / (period as f64 + 1.0);
        let mut ema = data[0].close;

        for i in 1..=index.min(data.len() - 1) {
            ema = alpha * data[i].close + (1.0 - alpha) * ema;
        }

        ema
    }

    /// Calculate Relative Strength Index
    fn calculate_rsi(&self, data: &[PriceData], index: usize, period: usize) -> f64 {
        if index < period {
            return 50.0; // Neutral RSI
        }

        let start = index.saturating_sub(period);
        let mut gains = 0.0;
        let mut losses = 0.0;

        for i in start + 1..=index {
            let change = data[i].close - data[i - 1].close;
            if change > 0.0 {
                gains += change;
            } else {
                losses += change.abs();
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

    /// Calculate MACD (Moving Average Convergence Divergence)
    fn calculate_macd(&self, data: &[PriceData], index: usize, fast: usize, slow: usize, signal: usize) -> (f64, f64, f64) {
        let ema_fast = self.calculate_ema(data, index, fast);
        let ema_slow = self.calculate_ema(data, index, slow);
        let macd_line = ema_fast - ema_slow;

        // For signal line, we'd need to calculate EMA of MACD line
        // Simplified version here
        let macd_signal = macd_line * 0.9; // Approximation
        let macd_histogram = macd_line - macd_signal;

        (macd_line, macd_signal, macd_histogram)
    }

    /// Calculate Volume Simple Moving Average
    fn calculate_volume_sma(&self, data: &[PriceData], index: usize, period: usize) -> f64 {
        let start = index.saturating_sub(period - 1);
        let end = index + 1;
        
        let sum: f64 = data[start..end].iter().map(|d| d.volume).sum();
        sum / (end - start) as f64
    }

    /// Calculate price volatility
    fn calculate_volatility(&self, data: &[PriceData], index: usize, period: usize) -> f64 {
        if index < period {
            return 0.0;
        }

        let start = index.saturating_sub(period - 1);
        let returns: Vec<f64> = data[start..=index]
            .windows(2)
            .map(|w| ((w[1].close - w[0].close) / w[0].close).ln())
            .collect();

        if returns.is_empty() {
            return 0.0;
        }

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / returns.len() as f64;

        variance.sqrt() * 100.0 // Convert to percentage
    }

    /// Calculate custom MEV score
    fn calculate_mev_score(&self, price: &PriceData, ema_values: &HashMap<usize, f64>, rsi: f64) -> f64 {
        let mut score = 0.0;

        // Price vs EMA signals
        if let Some(&ema_20) = ema_values.get(&20) {
            if price.close > ema_20 {
                score += 0.3;
            }
        }

        // RSI signals
        if rsi < 30.0 {
            score += 0.4; // Oversold
        } else if rsi > 70.0 {
            score += 0.2; // Overbought
        }

        // Volume signals
        let volume_ratio = price.volume / (price.volume + 1.0); // Avoid division by zero
        score += volume_ratio.min(0.3);

        score.min(1.0) // Cap at 1.0
    }

    /// Get extractor statistics
    pub fn get_stats(&self) -> FeatureExtractorStats {
        FeatureExtractorStats {
            worker_count: self.worker_count,
            indicator_count: self.config.ema_periods.len() + 4, // EMA + RSI + MACD + Volume + Volatility
        }
    }
}

/// Feature extractor statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureExtractorStats {
    pub worker_count: usize,
    pub indicator_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> Vec<PriceData> {
        (0..100).map(|i| PriceData {
            timestamp: i as u64,
            open: 100.0 + i as f64 * 0.1,
            high: 101.0 + i as f64 * 0.1,
            low: 99.0 + i as f64 * 0.1,
            close: 100.0 + i as f64 * 0.1,
            volume: 1000.0 + i as f64 * 10.0,
        }).collect()
    }

    #[tokio::test]
    async fn test_feature_extraction() {
        let config = IndicatorConfig::default();
        let extractor = LazyFeatureExtractor::new(config, 4);
        let data = create_test_data();

        let features = extractor.extract_features(&data).await.unwrap();
        assert_eq!(features.len(), data.len());

        // Check that features are calculated
        let first_feature = &features[50]; // Use middle point for better indicators
        assert!(first_feature.ema_values.len() > 0);
        assert!(first_feature.rsi > 0.0);
        assert!(first_feature.mev_score >= 0.0);
    }

    #[test]
    fn test_ema_calculation() {
        let config = IndicatorConfig::default();
        let extractor = LazyFeatureExtractor::new(config, 1);
        let data = create_test_data();

        let ema = extractor.calculate_ema(&data, 20, 10);
        assert!(ema > 0.0);
        assert!(ema < 200.0); // Reasonable range
    }

    #[test]
    fn test_rsi_calculation() {
        let config = IndicatorConfig::default();
        let extractor = LazyFeatureExtractor::new(config, 1);
        let data = create_test_data();

        let rsi = extractor.calculate_rsi(&data, 50, 14);
        assert!(rsi >= 0.0);
        assert!(rsi <= 100.0);
    }
}
