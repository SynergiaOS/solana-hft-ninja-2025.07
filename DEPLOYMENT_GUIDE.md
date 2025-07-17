# 🥷 **SOLANA HFT NINJA 2025.07 - COMPLETE MEV STRATEGY DEPLOYMENT GUIDE**

## 🎯 **QUICK START (1-DAY DEPLOYMENT)**

Your Solana HFT Ninja is **fully configured** with **5 MEV strategies** and **ready for deployment**!

### 🔑 Infisical Configuration
- **Project ID**: `73c2f3cb-c922-4a46-a333-7b96fbc6301a`
- **Environment**: `production`
- **Service Token**: ✅ Configured and validated
- **Secrets**: 2 secrets available (DRAGONFLY_API, WALLET_PRIVATE_KEY)

## 🎯 Quick Deployment Options

### Option 1: Direct Run with Infisical (Recommended)

```bash
# Simple one-command deployment
./scripts/run-with-infisical.sh
```

### Option 2: Manual Infisical Run

```bash
# Load environment
source .env.local

# Run with Infisical secrets injection
infisical run \
    --projectId=73c2f3cb-c922-4a46-a333-7b96fbc6301a \
    --env=production \
    --token="$INFISICAL_TOKEN" \
    -- ./target/release/hft_main \
    --dry-run \
    --enable-helius \
    --enable-mev \
    --enable-jito \
    --log-level info
```

### Option 3: Docker with Infisical

```bash
# Build and run with Infisical integration
docker-compose -f docker-compose.yml -f docker-compose.infisical.yml up -d

# Check logs
docker-compose logs -f hft-ninja

# Stop
docker-compose -f docker-compose.yml -f docker-compose.infisical.yml down
```

## 📊 What's Working

### ✅ Validated Components
1. **Infisical Integration**: Service token authenticated
2. **Secret Access**: 2 secrets accessible
3. **Helius WebSocket**: Connected and subscribed
4. **Bridge System**: Active mempool ↔ engine communication
5. **MEV Strategies**: Enabled and ready
6. **Jito Bundles**: Configured for execution
7. **Metrics Server**: Running on port 8080
8. **Docker Integration**: Ready for containerized deployment

### 🔐 Security Features
- **No secrets in code**: All sensitive data in Infisical
- **Service token authentication**: Secure API access
- **Environment separation**: Production/development isolation
- **Audit trail**: Complete access logging in Infisical

## 🛠️ Available Commands

```bash
# Validation and setup
./scripts/validate-infisical.sh          # Validate Infisical setup
./scripts/run-with-infisical.sh          # Run with Infisical

# Direct application runs
./target/release/hft_main --help         # See all options
./target/release/hft_main --dry-run --enable-helius --log-level debug

# Docker operations
docker-compose up -d                     # Standard deployment
docker-compose -f docker-compose.yml -f docker-compose.infisical.yml up -d  # With Infisical
docker-compose logs -f hft-ninja         # View logs
docker-compose down                      # Stop services

# Monitoring
curl http://localhost:8080/health         # Health check
curl http://localhost:8080/metrics       # Prometheus metrics
```

## 📈 Monitoring Endpoints

| Endpoint | Description | Status |
|----------|-------------|--------|
| `http://localhost:8080/health` | Health check | ✅ Active |
| `http://localhost:8080/metrics` | Prometheus metrics | ✅ Active |
| `http://localhost:3000` | Grafana dashboard | 🔄 Available with full stack |

## 🔧 Configuration

### Current Settings
- **Dry Run**: Enabled (safe testing mode)
- **Helius API**: Connected via Infisical secrets
- **MEV Detection**: Enabled
- **Jito Bundles**: Enabled
- **WebSocket**: `wss://mainnet.helius-rpc.com`

### Production Deployment
To switch to live trading:
1. Set `DRY_RUN=false` in Infisical
2. Ensure wallet has sufficient SOL
3. Monitor positions and risk limits
4. Enable alerting and monitoring

## 🆘 Troubleshooting

### Common Issues

1. **Port 8080 in use**
```bash
pkill -f hft_main
lsof -ti:8080 | xargs kill -9
```

2. **Infisical token issues**
```bash
./scripts/validate-infisical.sh
```

3. **Docker build issues**
```bash
docker-compose build --no-cache hft-ninja
```

4. **WebSocket connection issues**
- Check Helius API key in Infisical
- Verify network connectivity
- Check rate limits

### Debug Commands

```bash
# Check Infisical secrets
infisical secrets --projectId=73c2f3cb-c922-4a46-a333-7b96fbc6301a --env=production --token="$INFISICAL_TOKEN"

# Export secrets for debugging
infisical export --projectId=73c2f3cb-c922-4a46-a333-7b96fbc6301a --env=production --format=dotenv --token="$INFISICAL_TOKEN"

# Run with debug logging
RUST_LOG=debug ./scripts/run-with-infisical.sh
```

## 🎯 Next Steps

1. **Test in dry-run mode** (current setup)
2. **Monitor metrics** and performance
3. **Add additional secrets** to Infisical as needed
4. **Scale with Docker** for production
5. **Set up alerting** for critical events

## 🏆 Success Metrics

Your deployment is successful when you see:
- ✅ `Injecting 2 Infisical secrets into your application process`
- ✅ `Connected to Helius WebSocket`
- ✅ `Subscribed to mempool transactions`
- ✅ `Bridge initialized - mempool ↔ engine communication ready`
- ✅ `System status: 🟢 Ready`

---

**🥷 Your Solana HFT Ninja is ready for action with enterprise-grade security!**

---

## 🥷 **COMPLETE MEV STRATEGY IMPLEMENTATION**

### **📊 Strategy Configuration Matrix**

| Strategy | Min Profit | Max Position | Slippage | Priority | Status |
|----------|------------|--------------|----------|----------|---------|
| **🥪 Sandwich** | 0.015 SOL | 0.8 SOL | 4% | Highest | ✅ Active |
| **⚖️ Arbitrage** | 0.007 SOL | 1.2 SOL | 3% | High | ✅ Active |
| **🚀 Sniping** | 0.005 SOL | 0.5 SOL | 2.5% | Medium | ✅ Active |
| **🔄 Jupiter Arb** | 0.005 SOL | 1.0 SOL | 2.5% | High | ✅ Active |
| **💧 Liquidation** | 0.02 SOL | 2.0 SOL | 5% | Medium | ✅ Active |

### **🧪 Comprehensive Testing Commands**

```bash
# Run all MEV strategy tests
./scripts/run_strategies.sh test

# Test individual strategies
./scripts/run_strategies.sh sandwich    # Test sandwich attacks
./scripts/run_strategies.sh arbitrage   # Test cross-DEX arbitrage
./scripts/run_strategies.sh sniping     # Test token launch sniping
./scripts/run_strategies.sh jupiter_arbitrage  # Test Jupiter routes
./scripts/run_strategies.sh liquidation # Test liquidation opportunities

# Integration tests
cargo test --test integration_bridge_test -- --nocapture
cargo test --test integration_helius_test -- --nocapture
cargo test --test integration_jito_test -- --nocapture
```

### **🚀 Production Deployment**

```bash
# Launch all strategies with 8 SOL capital
INITIAL_BALANCE=8.0 DRY_RUN=false ./scripts/run_strategies.sh all

# Monitor real-time performance
watch -n 1 "curl -s http://localhost:8080/strategy/stats | jq ."

# Check Prometheus metrics
curl http://localhost:9090/api/v1/query?query=hft_mev_profit_sol
```

### **📈 Performance Monitoring**

**Grafana Dashboard**: http://localhost:3000
- Import: `monitoring/grafana/dashboards/mev-strategies-dashboard.json`
- Username: `admin` / Password: `admin`

**Key Metrics URLs**:
- **Prometheus**: http://localhost:9090
- **HFT API**: http://localhost:8080
- **Health Check**: http://localhost:8080/health
- **Strategy Stats**: http://localhost:8080/strategy/stats

### **🛡️ Risk Management & Emergency Procedures**

```bash
# Emergency stop all strategies
./scripts/run_strategies.sh emergency

# Manual strategy control
curl -X POST http://localhost:8080/strategy/sandwich/disable
curl -X POST http://localhost:8080/positions/close-all
curl -X POST http://localhost:8080/risk/reset

# Check wallet status
curl -X GET http://localhost:8080/wallet/status
```

### **⚡ Performance Targets (8 SOL Capital)**

| Metric | Target | Monitoring Command |
|--------|--------|-------------------|
| **Daily Profit** | 0.4 SOL (5% ROI) | `curl http://localhost:8080/strategy/metrics` |
| **Max Daily Loss** | 1.6 SOL (20%) | `curl http://localhost:9090/api/v1/query?query=hft_daily_loss_ratio` |
| **Sandwich Success** | >85% | `curl http://localhost:8080/strategy/sandwich/stats` |
| **Arbitrage Profit/Op** | >0.003 SOL | `curl http://localhost:8080/strategy/arbitrage/stats` |
| **Snipe Latency** | <100ms | `curl http://localhost:9090/api/v1/query?query=hft_snipe_latency_ms` |

### **✅ Production Readiness Checklist**

- [x] **MEV Engine**: Complete implementation with 5 strategies
- [x] **Configuration**: Optimized parameters in `config/strategies.toml`
- [x] **Testing**: Comprehensive unit and integration tests
- [x] **Monitoring**: Grafana dashboard and Prometheus metrics
- [x] **Risk Management**: Circuit breakers and position limits
- [x] **Emergency Procedures**: Automated and manual stop mechanisms
- [x] **Documentation**: Complete deployment and operation guides

🚀💰 **READY TO GENERATE MEV PROFITS!** 💰🚀
