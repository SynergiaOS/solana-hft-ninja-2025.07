use prometheus::{Counter, Gauge, Histogram, Registry};
use std::sync::Arc;

pub struct MetricsCollector {
    registry: Arc<Registry>,
    trades_total: Counter,
    pnl_total: Gauge,
    latency_histogram: Histogram,
}

impl MetricsCollector {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let registry = Arc::new(Registry::new());
        
        let trades_total = Counter::new("trades_total", "Total number of trades")?;
        let pnl_total = Gauge::new("pnl_total", "Total P&L in USD")?;
        let latency_histogram = Histogram::with_opts(
            prometheus::HistogramOpts::new("trade_latency", "Trade execution latency in ms")
        )?;
        
        registry.register(Box::new(trades_total.clone()))?;
        registry.register(Box::new(pnl_total.clone()))?;
        registry.register(Box::new(latency_histogram.clone()))?;
        
        Ok(Self {
            registry,
            trades_total,
            pnl_total,
            latency_histogram,
        })
    }

    pub fn increment_trades(&self) {
        self.trades_total.inc();
    }

    pub fn set_pnl(&self, pnl: f64) {
        self.pnl_total.set(pnl);
    }

    pub fn record_latency(&self, latency_ms: f64) {
        self.latency_histogram.observe(latency_ms);
    }
}