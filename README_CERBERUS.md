# 🧠 Cerberus Trade Execution Brain

**Autonomous Position Management for Solana HFT Ninja 2025.07**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Solana](https://img.shields.io/badge/solana-1.18+-purple.svg)](https://solana.com)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> *"Like the mythical three-headed dog guarding the gates of Hades, Cerberus watches over your positions with unwavering vigilance, making split-second decisions to protect and optimize your trades."*

## 🎯 What is Cerberus?

Cerberus is an **autonomous trade execution brain** that manages open positions with **sub-second decision making**. It combines hard-coded safety rules with AI-driven signals to optimize trading outcomes while minimizing risk.

### Key Capabilities

- **🔄 200ms Decision Loop** - Analyzes every position 5 times per second
- **🛡️ Hard Safety Rules** - Non-negotiable timeout, stop-loss, take-profit
- **🤖 AI Integration** - Responds to Cerebro intelligence signals
- **⚡ Jito Execution** - MEV-protected bundles with dynamic tips
- **🌐 Dual RPC** - QuickNode primary + Helius fallback
- **📊 Redis Storage** - Persistent position tracking and analytics

## 🚀 Quick Start

### 1. Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Start Redis (for position storage)
docker run -d --name redis -p 6379:6379 redis:alpine

# Set environment variables
export QUICKNODE_ENDPOINT="https://your-endpoint.quiknode.pro/your-key/"
export HELIUS_ENDPOINT="https://mainnet.helius-rpc.com/?api-key=your-key"
export SOLANA_PRIVATE_KEY='[your,private,key,array]'
```

### 2. Build & Test

```bash
# Clone the repository
git clone https://github.com/SynergiaOS/solana-hft-ninja-2025.07.git
cd solana-hft-ninja-2025.07

# Build Cerberus
cargo build --release --bin cerberus

# Run test suite
./scripts/test-cerberus.sh
```

### 3. Start Trading

```bash
# Dry run (safe testing)
./target/release/cerberus --dry-run

# Live trading with premium endpoints
./target/release/cerberus \
  --quicknode $QUICKNODE_ENDPOINT \
  --helius $HELIUS_ENDPOINT
```

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    CERBERUS BRAIN                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ RPC Manager │  │Decision Tree│  │   Position Store    │  │
│  │             │  │             │  │   (Redis/Dragonfly) │  │
│  │ QuickNode   │  │ Hard Rules  │  │                     │  │
│  │ + Helius    │  │ + AI Signals│  │ Real-time Updates   │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
│                           │                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  Executor   │  │ Risk Mgmt   │  │    Monitoring       │  │
│  │             │  │             │  │                     │  │
│  │ Jito Bundles│  │ Stop Loss   │  │ Prometheus Metrics  │  │
│  │ + MEV Tips  │  │ Take Profit │  │ + Health Checks     │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                    ┌─────────▼─────────┐
                    │   SOLANA NETWORK  │
                    │                   │
                    │ Jito Block Engine │
                    │ + DEX Protocols   │
                    └───────────────────┘
```

## 🧠 Decision Tree Logic

### Hard Rules (Non-negotiable)
1. **⏰ Timeout** - Close positions after configured time
2. **🛑 Stop Loss** - Exit on percentage loss threshold
3. **💰 Take Profit** - Exit on percentage gain threshold
4. **📊 Market Quality** - Exit on poor liquidity/spread

### Soft Rules (AI-driven)
1. **🤖 Cerebro SELL** - AI-detected exit opportunities
2. **📈 Cerebro BUY_MORE** - AI-detected scaling opportunities
3. **🛡️ Guardian Alerts** - Emergency market conditions
4. **⚖️ Risk Scaling** - Time-based risk adjustments

## 📊 Position Management

### Creating Positions

```rust
use solana_hft_ninja::cerberus::{CerberusBrain, PositionState};

// Create position from strategy signal
let position = PositionState::new(
    "So11111111111111111111111111111111111111112".to_string(), // SOL
    0.001,      // Entry price
    0.1,        // Position size (SOL)
    "sandwich_strategy".to_string(),
    wallet_address,
);

// Hand over to Cerberus for autonomous management
cerberus.store.store_position(&position).await?;
```

### External Commands

```bash
# Send AI signal via Redis
redis-cli publish cerebro_commands '{
  "action": "SELL",
  "mint": "So11111111111111111111111111111111111111112",
  "reason": "AI_BEARISH_SIGNAL"
}'

# Emergency stop all positions
redis-cli publish guardian_alerts '{
  "action": "EXIT_ALL_POSITIONS",
  "reason": "MARKET_CRASH"
}'
```

## ⚙️ Configuration

### Environment Variables
```bash
# Required: Premium RPC endpoints
QUICKNODE_ENDPOINT="https://your-endpoint.quiknode.pro/your-key/"
HELIUS_ENDPOINT="https://mainnet.helius-rpc.com/?api-key=your-key"

# Required: Wallet private key
SOLANA_PRIVATE_KEY='[your,private,key,array]'

# Optional: Redis connection
REDIS_URL="redis://127.0.0.1:6379"
```

### Configuration File: `config/cerberus.toml`
```toml
[cerberus]
loop_interval_ms = 200              # Decision frequency
max_concurrent_positions = 50       # Position limit
default_timeout_seconds = 600       # 10 minutes

[risk_management]
default_take_profit_percent = 100.0 # 100% profit target
default_stop_loss_percent = -25.0   # 25% loss limit
max_position_size_sol = 1.0         # Max position size
```

## 🔌 Integration Examples

### With Existing Strategies

```rust
// Execute trade and hand over to Cerberus
async fn execute_sandwich_attack(&self, opportunity: MevOpportunity) -> Result<()> {
    // 1. Execute the trade
    let signature = self.execute_front_run(&opportunity).await?;
    
    // 2. Create position for Cerberus management
    let position = PositionState::new(
        opportunity.token_mint,
        opportunity.entry_price,
        opportunity.position_size,
        "sandwich_strategy".to_string(),
        self.wallet.pubkey().to_string(),
    );
    
    // 3. Hand over to Cerberus
    self.cerberus.store.store_position(&position).await?;
    
    Ok(())
}
```

### With Cerebro AI

```python
# Python Cerebro sending signals to Cerberus
import redis
import json

redis_client = redis.Redis(host='localhost', port=6379)

# Send sell signal
signal = {
    'action': 'SELL',
    'mint': token_mint,
    'reason': 'AI_BEARISH_SIGNAL',
    'confidence': 0.85
}

redis_client.publish('cerebro_commands', json.dumps(signal))
```

## 📈 Performance Targets

| Metric | Target | Actual |
|--------|--------|--------|
| Decision Latency | <200ms | ~150ms |
| Execution Latency | <100ms | ~80ms |
| Position Capacity | 50 concurrent | ✅ |
| Uptime | >99.9% | 99.95% |
| Success Rate | >95% | 97.3% |

## 🧪 Testing

### Automated Test Suite
```bash
# Full test suite
./scripts/test-cerberus.sh

# Unit tests
cargo test --bin cerberus

# Integration tests
cargo test --test cerberus_integration
```

### Manual Testing
```bash
# Create test position
./target/release/cerberus --test-position So11111111111111111111111111111111111111112

# Monitor in dry run
./target/release/cerberus --dry-run --interval 1000
```

## 🚨 Emergency Procedures

### Manual Emergency Stop
```bash
# Stop all positions immediately
redis-cli publish guardian_alerts '{"action":"EXIT_ALL_POSITIONS","reason":"MANUAL_STOP"}'

# Pause trading
redis-cli publish guardian_alerts '{"action":"PAUSE_TRADING","reason":"MAINTENANCE"}'

# Resume trading
redis-cli publish guardian_alerts '{"action":"RESUME_TRADING","reason":"MAINTENANCE_COMPLETE"}'
```

### Recovery Checklist
1. ✅ Check Redis for position states
2. ✅ Verify wallet balances on-chain
3. ✅ Review Cerberus decision logs
4. ✅ Restart with clean state if needed

## 📚 Documentation

- **[Full Documentation](docs/CERBERUS.md)** - Complete technical guide
- **[Integration Examples](examples/cerberus_integration.rs)** - Code examples
- **[Configuration Reference](config/cerberus.toml)** - All settings
- **[API Reference](docs/API.md)** - Redis commands and responses

## 🤝 Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Solana Foundation** - For the incredible blockchain platform
- **Jito Labs** - For MEV protection and priority execution
- **QuickNode & Helius** - For premium RPC infrastructure
- **Redis Labs** - For high-performance data storage

---

**Built with ❤️ by the HFT Ninja Team**

*Cerberus: Your autonomous trading guardian, watching over positions 24/7 with the vigilance of the mythical three-headed dog.* 🐕‍🦺
