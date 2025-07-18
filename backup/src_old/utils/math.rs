pub struct MathUtils;

impl MathUtils {
    pub fn calculate_spread(bid: f64, ask: f64) -> f64 {
        (ask - bid) / bid * 100.0
    }

    pub fn calculate_mid_price(bid: f64, ask: f64) -> f64 {
        (bid + ask) / 2.0
    }

    pub fn apply_slippage(price: f64, slippage_bps: u64, is_buy: bool) -> f64 {
        let slippage = slippage_bps as f64 / 10_000.0;
        if is_buy {
            price * (1.0 + slippage)
        } else {
            price * (1.0 - slippage)
        }
    }

    pub fn calculate_pnl(entry_price: f64, exit_price: f64, size: f64, is_long: bool) -> f64 {
        if is_long {
            (exit_price - entry_price) * size
        } else {
            (entry_price - exit_price) * size
        }
    }
}