# 🚀 **CEREBRO HFT NINJA - MAINNET DEPLOYMENT GUIDE**

**Production deployment guide for live Solana trading**

![Mainnet](https://img.shields.io/badge/Solana-Mainnet-purple)
![Production](https://img.shields.io/badge/Status-Production%20Ready-brightgreen)
![Security](https://img.shields.io/badge/Security-Audited-blue)

---

## 🎯 **DEPLOYMENT OVERVIEW**

### **Gradual Rollout Strategy**
1. **Phase 1**: Paper Trading (Risk-free testing)
2. **Phase 2**: Small Amounts (0.1 SOL positions)
3. **Phase 3**: Full Production (1.0 SOL positions)

### **Safety Features**
- ✅ **Emergency shutdown** system
- ✅ **Circuit breakers** and stop-loss
- ✅ **Real-time monitoring** and alerts
- ✅ **Risk management** with position limits
- ✅ **Gradual rollout** with testing phases

---

## 📋 **PRE-DEPLOYMENT CHECKLIST**

### **🔐 Security Requirements**
```bash
# 1. Run wallet setup
./scripts/mainnet-wallet-setup.sh

# 2. Run security audit
./scripts/security-audit.sh

# 3. Verify all checks pass
./scripts/deploy-mainnet.sh check
```

### **💰 Wallet Requirements**
- ✅ **Minimum balance**: 1.0 SOL
- ✅ **Recommended**: 8+ SOL for full strategies
- ✅ **Backup created**: Secure offline storage
- ✅ **Private key secured**: 600 permissions

### **🌐 Infrastructure Requirements**
- ✅ **Helius API**: $99/month premium account
- ✅ **QuickNode**: $49/month backup RPC
- ✅ **Docker**: Latest version installed
- ✅ **Monitoring**: Prometheus + Grafana ready

---

## 🚀 **DEPLOYMENT STEPS**

### **Step 1: Environment Setup**
```bash
# Copy and configure environment
cp .env.mainnet.template .env.mainnet

# Edit with your values
nano .env.mainnet

# Key configurations:
HELIUS_API_KEY=your_helius_key
QUICKNODE_ENDPOINT=your_quicknode_endpoint
WALLET_PRIVATE_KEY_PATH=/path/to/wallet.json
TRADING_ENABLED=false  # Start disabled!
```

### **Step 2: Security Verification**
```bash
# Run comprehensive security check
./scripts/security-audit.sh

# Expected output:
# ✅ .env.mainnet has secure permissions (600)
# ✅ Wallet file has secure permissions (600)
# ✅ Helius API key configured
# ✅ Circuit breaker enabled
# 🎉 AUDIT PASSED - System ready for mainnet deployment!
```

### **Step 3: Phase 1 - Paper Trading**
```bash
# Deploy in paper trading mode
./scripts/deploy-mainnet.sh paper

# This will:
# - Start all Docker services
# - Keep trading disabled
# - Enable monitoring
# - Run health checks

# Monitor at:
# Dashboard: http://localhost:3001
# Monitoring: http://localhost:3000
```

### **Step 4: Phase 2 - Small Amount Testing**
```bash
# Enable small amount trading
./scripts/deploy-mainnet.sh small

# Limits applied:
# - Max position: 0.1 SOL
# - Max daily loss: 0.05 SOL
# - Stop loss: 3%
# - Max trades: 2/minute

# Monitor for 1 hour minimum!
```

### **Step 5: Phase 3 - Full Production**
```bash
# Enable full production trading
./scripts/deploy-mainnet.sh full

# Production limits:
# - Max position: 1.0 SOL
# - Max daily loss: 0.5 SOL
# - Stop loss: 5%
# - Max trades: 10/minute

# Target: 5% daily ROI (0.4 SOL from 8 SOL)
```

---

## 📊 **MONITORING & ALERTS**

### **Real-time Dashboards**
```bash
# Access monitoring interfaces
Dashboard:    http://localhost:3001
Grafana:      http://localhost:3000
Prometheus:   http://localhost:9090
API Health:   http://localhost:8000/health
HFT Status:   http://localhost:8080/health
```

### **Alert Channels**
- 🔔 **Telegram**: Instant notifications
- 📧 **Email**: Critical alerts
- 💬 **Discord**: Team notifications
- 📱 **SMS**: Emergency contacts

### **Key Metrics to Monitor**
```typescript
// Critical metrics
const monitoringMetrics = {
  trading: {
    daily_pnl: "Target: +5% (+0.4 SOL)",
    win_rate: "Target: >85%",
    trades_count: "Monitor frequency",
    avg_latency: "Target: <100ms"
  },
  
  system: {
    api_health: "Must be 'healthy'",
    rpc_latency: "Target: <50ms",
    memory_usage: "Alert if >80%",
    error_rate: "Alert if >1%"
  },
  
  risk: {
    position_utilization: "Monitor vs limits",
    daily_loss: "Alert at 80% of limit",
    drawdown: "Alert if >10%",
    circuit_breaker: "Monitor triggers"
  }
};
```

---

## 🚨 **EMERGENCY PROCEDURES**

### **Emergency Shutdown**
```bash
# Immediate halt of all trading
./scripts/emergency-shutdown.sh "Reason for shutdown"

# This will:
# - Disable all trading
# - Cancel open orders
# - Close positions
# - Stop workflows
# - Send alerts
# - Create backup
```

### **Emergency Contacts**
```bash
# Configure in .env.mainnet
EMERGENCY_CONTACT_EMAIL=admin@your-domain.com
EMERGENCY_CONTACT_PHONE=+1234567890
TELEGRAM_BOT_TOKEN=your_bot_token
TELEGRAM_CHAT_ID=your_chat_id
```

### **Recovery Procedures**
```bash
# After emergency shutdown
# 1. Investigate the issue
# 2. Review logs and backups
# 3. Fix the problem
# 4. Run restart script
./scripts/restart-after-emergency.sh
```

---

## 💰 **COST BREAKDOWN**

### **Monthly Operational Costs**
| Service | Cost | Purpose |
|---------|------|---------|
| Helius RPC Pro | $99 | Primary blockchain data |
| QuickNode Backup | $49 | Fallback RPC provider |
| AWS Infrastructure | $150 | Database, monitoring |
| **Total** | **$298** | **Monthly operational** |

### **Expected Returns**
- **Daily Target**: 5% ROI (0.4 SOL from 8 SOL)
- **Monthly Profit**: ~$3,000 (after costs)
- **Break-even**: 3 days of successful trading
- **Annual ROI**: ~900% on infrastructure investment

---

## 🎯 **SUCCESS CRITERIA**

### **Technical KPIs**
- ⚡ **Latency**: <100ms average response time
- 🔄 **Uptime**: 99.9% system availability
- 📊 **Accuracy**: 99.9% data consistency
- 🚀 **Throughput**: 1000+ operations/second

### **Trading KPIs**
- 💰 **Daily ROI**: 5% target
- 🎯 **Win Rate**: >85% successful trades
- 📉 **Max Drawdown**: <10% risk limit
- 📈 **Profit Factor**: >2.0 risk-adjusted returns

### **Risk Management KPIs**
- 🛡️ **Position Limits**: Never exceeded
- 🚨 **Stop Loss**: Triggered when needed
- ⚡ **Circuit Breaker**: Functional
- 🔄 **Recovery Time**: <5 minutes

---

## 🔧 **TROUBLESHOOTING**

### **Common Issues**

#### **RPC Connection Errors**
```bash
# Check RPC status
curl -s "http://localhost:8080/api/rpc/status"

# Switch to backup RPC
docker exec cerebro-hft-mainnet curl -X POST \
  "http://localhost:8080/api/rpc/switch" \
  -d '{"provider": "quicknode"}'
```

#### **High Latency**
```bash
# Check network latency
ping api.mainnet-beta.solana.com

# Monitor RPC performance
curl -s "http://localhost:9090/api/v1/query?query=rpc_latency_seconds"
```

#### **Trading Disabled**
```bash
# Check risk status
curl -s "http://localhost:8080/api/risk/status"

# Re-enable if safe
curl -X POST "http://localhost:8080/api/trading/enable" \
  -d '{"reason": "Manual re-enable after investigation"}'
```

### **Log Locations**
```bash
# Application logs
tail -f logs/hft/trading.log
tail -f logs/api/application.log

# System logs
docker logs cerebro-hft-mainnet
docker logs cerebro-api-mainnet

# Emergency logs
ls -la logs/emergency-shutdown-*.log
```

---

## 📈 **PERFORMANCE OPTIMIZATION**

### **Latency Optimization**
```bash
# Monitor latency metrics
curl -s "http://localhost:9090/api/v1/query?query=trading_latency_p95"

# Optimize RPC connections
# - Use premium endpoints
# - Enable connection pooling
# - Implement request batching
```

### **Memory Optimization**
```bash
# Monitor memory usage
docker stats cerebro-hft-mainnet

# Optimize if needed
# - Increase container memory
# - Tune garbage collection
# - Implement memory profiling
```

---

## 🏆 **PRODUCTION CHECKLIST**

### **Before Going Live**
- [ ] Security audit passed
- [ ] Wallet funded and secured
- [ ] RPC providers configured
- [ ] Monitoring stack deployed
- [ ] Alert channels tested
- [ ] Emergency procedures tested
- [ ] Risk limits configured
- [ ] Paper trading successful

### **During Deployment**
- [ ] Phase 1: Paper trading verified
- [ ] Phase 2: Small amounts tested
- [ ] Phase 3: Full production enabled
- [ ] Monitoring active
- [ ] Alerts functioning
- [ ] Performance within targets

### **Post-Deployment**
- [ ] 24-hour monitoring
- [ ] Performance review
- [ ] Risk assessment
- [ ] Optimization opportunities
- [ ] Documentation updated

---

## 🎉 **DEPLOYMENT COMPLETE**

**Congratulations! Cerebro HFT Ninja is now live on Solana Mainnet!**

### **What's Next?**
1. **Monitor closely** for first 24 hours
2. **Optimize strategies** based on performance
3. **Scale gradually** as confidence grows
4. **Maintain vigilance** with risk management

### **Support & Maintenance**
- **Daily monitoring** of key metrics
- **Weekly performance** reviews
- **Monthly optimization** cycles
- **Quarterly security** audits

---

**🧠 "From development to production - Cerebro HFT Ninja is ready to dominate Solana markets!"** 🚀

**Live trading activated. May the profits be with you!** ✨
