# 🔐 **INFISICAL DEPLOYMENT GUIDE**
## **Solana HFT Ninja 2025.07 - Secure Production Deployment**

Kompletny przewodnik wdrożenia systemu **Solana HFT Ninja 2025.07** z **Infisical** do bezpiecznego zarządzania sekretami na **Oracle Cloud**.

---

## 🚀 **QUICK START - Oracle Cloud Deployment**

### **1. Połączenie z Oracle Cloud**
```bash
# Połącz się z instancją Oracle Cloud
ssh -i ~/.ssh/oracle_cloud opc@130.61.104.45

# Lub użyj publicznego IP jeśli dostępny
ssh -i ~/.ssh/oracle_cloud opc@<PUBLIC_IP>
```

### **2. Klonowanie Repo**
```bash
# Klonuj najnowszą wersję
git clone https://github.com/SynergiaOS/solana-hft-ninja-2025.07.git
cd solana-hft-ninja-2025.07

# Sprawdź najnowszy commit
git log --oneline -5
```

### **3. Automatyczne Wdrożenie z Infisical**
```bash
# Uruchom automatyczny deployment
chmod +x scripts/deploy-oracle-cloud.sh
./scripts/deploy-oracle-cloud.sh
```

---

## 🔐 **INFISICAL CONFIGURATION**

### **Project Details**
- **Project ID**: `73c2f3cb-c922-4a46-a333-7b96fbc6301a`
- **Environment**: `production`
- **Token**: `st.7ab7091a-ae4f-41ba-b31c-bde5bafa4599.47542cb1d455d61335eaca92b2f6abfa.941bf8d2786836054e1fec510dd5f86b`

### **Required Secrets in Infisical**
```bash
# Solana Configuration
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_WS_URL=wss://api.mainnet-beta.solana.com
SOLANA_PRIVATE_KEY=<your_wallet_private_key>

# Helius API
HELIUS_API_KEY=<your_helius_api_key>
HELIUS_RPC_URL=https://mainnet.helius-rpc.com

# Jito Configuration
JITO_BLOCK_ENGINE_URL=https://amsterdam.mainnet.block-engine.jito.wtf
JITO_RELAYER_URL=https://amsterdam.mainnet.relayer.jito.wtf

# Trading Configuration
ENABLE_MEV=true
ENABLE_JITO=true
DRY_RUN=false
MIN_PROFIT_SOL=0.005
MAX_POSITION_SIZE=1.0

# Monitoring
PROMETHEUS_PORT=8080
LOG_LEVEL=info
```

---

## 🛠️ **MANUAL SETUP (Alternative)**

### **1. Install Dependencies**
```bash
# Update system
sudo dnf update -y

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# Install Infisical CLI
curl -1sLf 'https://dl.cloudsmith.io/public/infisical/infisical-cli/setup.rpm.sh' | sudo -E bash
sudo dnf install -y infisical
```

### **2. Setup Infisical**
```bash
# Create local environment file
cat > .env.local << EOF
INFISICAL_TOKEN=st.7ab7091a-ae4f-41ba-b31c-bde5bafa4599.47542cb1d455d61335eaca92b2f6abfa.941bf8d2786836054e1fec510dd5f86b
INFISICAL_PROJECT_ID=73c2f3cb-c922-4a46-a333-7b96fbc6301a
INFISICAL_ENVIRONMENT=production
EOF

# Test Infisical connection
source .env.local
infisical secrets --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENVIRONMENT" --token="$INFISICAL_TOKEN"
```

### **3. Build Application**
```bash
# Build release version
cargo build --release --bin hft_main

# Verify build
ls -la target/release/hft_main
```

### **4. Run with Infisical**
```bash
# Run with secrets injection
./scripts/run-with-infisical.sh

# Or manually
infisical run \
    --projectId="73c2f3cb-c922-4a46-a333-7b96fbc6301a" \
    --env="production" \
    --token="st.7ab7091a-ae4f-41ba-b31c-bde5bafa4599.47542cb1d455d61335eaca92b2f6abfa.941bf8d2786836054e1fec510dd5f86b" \
    -- ./target/release/hft_main --enable-helius --enable-mev --enable-jito
```

---

## 🎯 **PRODUCTION CHECKLIST**

### **✅ Pre-Deployment**
- [ ] Oracle Cloud instance running (130.61.104.45)
- [ ] SSH access configured
- [ ] Infisical project setup with secrets
- [ ] Helius API key obtained
- [ ] Solana wallet funded (minimum 8 SOL)

### **✅ Deployment**
- [ ] Repository cloned on Oracle Cloud
- [ ] Dependencies installed (Rust, Infisical CLI)
- [ ] Application built successfully
- [ ] Infisical connection verified
- [ ] Systemd service created

### **✅ Post-Deployment**
- [ ] Application running (systemctl status solana-hft-ninja)
- [ ] Health endpoint responding (curl localhost:8080/health)
- [ ] Metrics available (curl localhost:8080/metrics)
- [ ] Logs monitoring (journalctl -u solana-hft-ninja -f)

---

## 📊 **MONITORING & MANAGEMENT**

### **Service Management**
```bash
# Check status
sudo systemctl status solana-hft-ninja

# View logs
sudo journalctl -u solana-hft-ninja -f

# Restart service
sudo systemctl restart solana-hft-ninja

# Stop service
sudo systemctl stop solana-hft-ninja
```

### **Health Checks**
```bash
# Application health
curl http://localhost:8080/health

# Prometheus metrics
curl http://localhost:8080/metrics

# System resources
htop
iotop
```

### **Access Points**
- **Health Check**: http://130.61.104.45:8080/health
- **Metrics**: http://130.61.104.45:8080/metrics
- **Internal DNS**: http://ninja.subnet07161247.vcn07161247.oraclevcn.com:8080

---

## 🔧 **TROUBLESHOOTING**

### **Common Issues**

#### **Infisical Connection Failed**
```bash
# Check token validity
echo $INFISICAL_TOKEN

# Test connection
infisical secrets --projectId="73c2f3cb-c922-4a46-a333-7b96fbc6301a" --env="production"

# Re-authenticate if needed
infisical login
```

#### **Application Won't Start**
```bash
# Check logs
sudo journalctl -u solana-hft-ninja -n 50

# Check configuration
cat .env.local

# Test binary directly
./target/release/hft_main --help
```

#### **Network Issues**
```bash
# Check firewall
sudo iptables -L

# Test connectivity
curl -I https://api.mainnet-beta.solana.com
curl -I https://mainnet.helius-rpc.com
```

---

## 🚀 **MAINNET ACTIVATION**

### **Final Steps for Live Trading**
1. **Verify all secrets** in Infisical production environment
2. **Set DRY_RUN=false** in Infisical
3. **Fund wallet** with sufficient SOL (minimum 8 SOL)
4. **Monitor first transactions** closely
5. **Set up alerts** for critical events

### **Go Live Command**
```bash
# Update Infisical to disable dry-run
infisical secrets set DRY_RUN false --projectId="73c2f3cb-c922-4a46-a333-7b96fbc6301a" --env="production"

# Restart service with live trading
sudo systemctl restart solana-hft-ninja

# Monitor startup
sudo journalctl -u solana-hft-ninja -f
```

---

## 🎉 **SUCCESS INDICATORS**

When deployment is successful, you should see:
- ✅ **Service Status**: `active (running)`
- ✅ **Health Check**: Returns `OK`
- ✅ **Metrics**: Prometheus metrics available
- ✅ **Logs**: No error messages in logs
- ✅ **Wallet**: Connected and funded
- ✅ **Strategies**: MEV strategies active

**System is ready for high-frequency trading on Solana mainnet!** 🥷💰
