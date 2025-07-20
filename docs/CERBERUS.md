# 🧠 Cerberus Trade Execution Brain

**Autonomous Position Management System for Solana HFT Ninja 2025.07**

## Overview

Cerberus is an intelligent trade execution brain that autonomously manages open positions with sub-second decision making. It combines hard rules (timeout, stop-loss, take-profit) with AI-driven signals from Cerebro to optimize trading outcomes.

## 🎯 Key Features

### **Autonomous Decision Making**
- **200ms decision loop** - Analyzes every position 5 times per second
- **Hard rules enforcement** - Non-negotiable safety limits
- **AI signal integration** - Responds to Cerebro intelligence
- **Emergency stop capability** - Instant position closure on alerts

### **Dual RPC Architecture**
- **Primary**: QuickNode premium endpoint
- **Fallback**: Helius premium endpoint  
- **Health monitoring** - Automatic failover
- **Sub-100ms latency** - Optimized for speed

### **Jito Bundle Execution**
- **MEV protection** - Bundles with dynamic tips
- **Priority execution** - Front-of-block placement
- **Slippage minimization** - Atomic transaction groups
- **Gas optimization** - Smart tip calculation

### **Redis/DragonflyDB Storage**
- **Position persistence** - Survives restarts
- **Real-time updates** - Live position tracking
- **Command channels** - External signal integration
- **Performance metrics** - Comprehensive analytics

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   QuickNode     │    │     Helius      │    │   DragonflyDB   │
│   (Primary)     │    │   (Fallback)    │    │   (Storage)     │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          └──────────┬───────────┘                      │
                     │                                  │
         ┌───────────▼──────────────────────────────────▼───────┐
         │                CERBERUS BRAIN                       │
         │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
         │  │ RPC Manager │  │Decision Tree│  │   Storage   │  │
         │  └─────────────┘  └─────────────┘  └─────────────┘  │
         │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
         │  │  Executor   │  │ Risk Mgmt   │  │ Monitoring  │  │
         │  └─────────────┘  └─────────────┘  └─────────────┘  │
         └─────────────────────┬───────────────────────────────┘
                               │
         ┌─────────────────────▼───────────────────────────────┐
         │                JITO BUNDLES                         │
         │     ┌─────────┐  ┌─────────┐  ┌─────────┐          │
         │     │  SELL   │  │BUY MORE │  │  TIPS   │          │
         │     └─────────┘  └─────────┘  └─────────┘          │
         └─────────────────────────────────────────────────────┘
```

## 🔧 Configuration

### Environment Variables
```bash
# Premium RPC Endpoints
export QUICKNODE_ENDPOINT="https://your-endpoint.quiknode.pro/your-key/"
export HELIUS_ENDPOINT="https://mainnet.helius-rpc.com/?api-key=your-key"

# Wallet Configuration
export SOLANA_PRIVATE_KEY='[your,private,key,array]'

# Optional: Redis URL
export REDIS_URL="redis://127.0.0.1:6379"
```

### Configuration File: `config/cerberus.toml`
```toml
[cerberus]
loop_interval_ms = 200
max_concurrent_positions = 50
default_timeout_seconds = 600

[risk_management]
default_take_profit_percent = 100.0
default_stop_loss_percent = -25.0
max_position_size_sol = 1.0
```

## 🚀 Usage

### Basic Commands

```bash
# Start Cerberus with premium endpoints
./target/release/cerberus \
  --quicknode $QUICKNODE_ENDPOINT \
  --helius $HELIUS_ENDPOINT

# Dry run mode (safe testing)
./target/release/cerberus --dry-run

# Create test position
./target/release/cerberus --test-position So11111111111111111111111111111111111111112

# Custom interval (default: 200ms)
./target/release/cerberus --interval 100
```

### Advanced Usage

```bash
# Production deployment with monitoring
./target/release/cerberus \
  --quicknode $QUICKNODE_ENDPOINT \
  --helius $HELIUS_ENDPOINT \
  --redis redis://production:6379 \
  --jito https://mainnet.block-engine.jito.wtf
```

## 🧠 Decision Tree Logic

### Hard Rules (Non-negotiable)
1. **Timeout Check** - Close positions after configured time
2. **Stop Loss** - Exit on percentage loss threshold  
3. **Take Profit** - Exit on percentage gain threshold
4. **Market Quality** - Exit on poor liquidity/spread

### Soft Rules (AI-driven)
1. **Cerebro SELL signals** - AI-detected exit opportunities
2. **Cerebro BUY_MORE signals** - AI-detected scaling opportunities
3. **Guardian alerts** - Emergency market conditions
4. **Risk scaling** - Time-based risk adjustments

### Decision Priority
```
1. Emergency Stop (Guardian)
2. Timeout
3. Stop Loss  
4. Take Profit
5. Market Quality Issues
6. AI Signals (Cerebro)
7. Risk Management
8. HOLD (default)
```

## 📊 Position Management

### Position States
- **Open** - Active position being monitored
- **Closed** - Position exited with reason
- **Pending** - Position being created/modified
- **Failed** - Position creation/exit failed

### Position Data Structure
```rust
pub struct PositionState {
    pub mint: String,                    // Token mint address
    pub entry_price: f64,               // Entry price in SOL
    pub position_size_sol: f64,         // Position size in SOL
    pub take_profit_target_percent: f64, // Take profit %
    pub stop_loss_target_percent: f64,   // Stop loss %
    pub timeout_seconds: u64,           // Position timeout
    pub current_price: Option<f64>,     // Live price
    pub pnl_unrealized_percent: Option<f64>, // Current PnL
}
```

## 🔌 External Integration

### Guardian Alerts (Redis Channel: `guardian_alerts`)
```json
{
  "action": "EXIT_ALL_POSITIONS",
  "reason": "MARKET_CRASH"
}
```

### Cerebro Commands (Redis Channel: `cerebro_commands`)
```json
{
  "action": "SELL",
  "mint": "So11111111111111111111111111111111111111112",
  "reason": "AI_SIGNAL_BEARISH"
}
```

## 📈 Performance Metrics

### Target Performance
- **Decision Latency**: <200ms average
- **Execution Latency**: <100ms (Jito bundles)
- **Position Capacity**: 50 concurrent positions
- **Uptime**: >99.9%
- **Success Rate**: >95% for valid signals

### Monitoring
```bash
# Position statistics
redis-cli get "cerberus:stats"

# Active positions count
redis-cli scard "active_positions"

# Recent decisions
redis-cli lrange "cerberus:decisions" 0 10
```

## 🛡️ Risk Management

### Built-in Safeguards
- **Position size limits** - Maximum SOL per position
- **Total exposure limits** - Maximum total SOL at risk
- **Time-based scaling** - Tighter stops over time
- **Market quality checks** - Liquidity and spread validation
- **Emergency stops** - Instant exit on alerts

### Risk Parameters
```toml
[risk_management]
max_position_size_sol = 1.0
max_total_exposure_sol = 5.0
min_liquidity_multiplier = 10.0
max_spread_percent = 5.0
time_based_stop_loss_factor = 0.8
```

## 🧪 Testing

### Run Test Suite
```bash
# Full test suite
./scripts/test-cerberus.sh

# Individual tests
cargo test --bin cerberus
```

### Test Scenarios
1. **Position Creation** - Verify Redis storage
2. **Decision Tree** - Test all exit conditions
3. **RPC Failover** - Primary/fallback switching
4. **Command Handling** - External signal processing
5. **Performance** - Latency and throughput

## 🚨 Emergency Procedures

### Manual Emergency Stop
```bash
# Stop all positions immediately
redis-cli publish guardian_alerts '{"action":"EXIT_ALL_POSITIONS","reason":"MANUAL_STOP"}'

# Pause trading
redis-cli publish guardian_alerts '{"action":"PAUSE_TRADING","reason":"MAINTENANCE"}'
```

### Recovery Procedures
1. **Check position status** in Redis
2. **Verify wallet balances** on-chain
3. **Review decision logs** for issues
4. **Restart with clean state** if needed

## 📚 Integration Examples

### With Existing HFT Ninja
```rust
// Create position from strategy
let position = PositionState::new(
    mint.to_string(),
    entry_price,
    position_size,
    strategy_id,
    wallet_address,
);

// Store in Cerberus
cerberus.store.store_position(&position).await?;
```

### With Cerebro AI
```python
# Send AI signal to Cerberus
redis_client.publish('cerebro_commands', json.dumps({
    'action': 'SELL',
    'mint': mint_address,
    'reason': 'AI_BEARISH_SIGNAL',
    'confidence': 0.85
}))
```

## 🔮 Future Enhancements

- **Machine Learning Integration** - Adaptive decision parameters
- **Cross-DEX Execution** - Multi-venue order routing  
- **Advanced Risk Models** - Portfolio-level risk management
- **Real-time Analytics** - Live performance dashboards
- **Mobile Alerts** - Push notifications for critical events

---

**Cerberus** - Your autonomous trading guardian, watching over positions 24/7 with the vigilance of the mythical three-headed dog. 🐕‍🦺
