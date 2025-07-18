# 🚀 Oracle Free Tier Quick Deploy Guide

## 🎯 **1-DAY DEPLOYMENT PLAN**

Deploy **Solana HFT Ninja + DeepSeek-Math AI** on Oracle Free Tier for **$0/month forever**!

### **⏱️ Timeline**
- **Setup Oracle Account**: 15 minutes
- **Create ARM Instance**: 10 minutes  
- **Deploy HFT Ninja**: 30 minutes
- **Verify & Test**: 15 minutes
- **Total**: **70 minutes to AI trading!**

## 🏗️ **STEP 1: Oracle Account Setup (15 min)**

### **Create Free Tier Account**

1. **Visit Oracle Cloud**: [oracle.com/cloud/free](https://www.oracle.com/cloud/free)
2. **Sign up** with email and phone verification
3. **Get $300 credit** for 30 days + Always Free resources
4. **Verify identity** with credit card (no charges)

### **Key Benefits**
- ✅ **$300 free credit** for first 30 days
- ✅ **Always Free Tier** - no time limits
- ✅ **4 OCPU + 24 GB RAM** ARM Ampere forever
- ✅ **200 GB storage** + 10 TB network/month

## 🖥️ **STEP 2: Create ARM Instance (10 min)**

### **Instance Configuration**

```bash
# Recommended settings:
Region: us-ashburn-1 (best availability)
Availability Domain: Any available
Shape: VM.Standard.A1.Flex
OCPU: 4 (maximum free)
Memory: 24 GB (maximum free)
OS: Ubuntu 22.04 LTS
Storage: 200 GB (maximum free)
```

### **Network Setup**

1. **Create VCN** (Virtual Cloud Network)
2. **Configure Security List**:
   ```
   Ingress Rules:
   - Port 22 (SSH)
   - Port 80 (HTTP)
   - Port 443 (HTTPS)
   - Port 8080 (HFT Ninja)
   - Port 3000 (Dashboard)
   - Port 8003 (AI API)
   ```

3. **Generate SSH Key Pair**
4. **Launch Instance**

## 🚀 **STEP 3: Automated Deployment (30 min)**

### **Connect to Instance**

```bash
# SSH into your Oracle instance
ssh -i your-oracle-key.pem ubuntu@your-oracle-ip

# Verify ARM64 architecture
uname -m  # Should show: aarch64
```

### **One-Command Deployment**

```bash
# Download and run deployment script
curl -fsSL https://raw.githubusercontent.com/SynergiaOS/solana-hft-ninja-2025.07/main/scripts/deploy-oracle-free-tier.sh | bash

# Or manual deployment:
git clone https://github.com/SynergiaOS/solana-hft-ninja-2025.07.git
cd solana-hft-ninja-2025.07
chmod +x scripts/deploy-oracle-free-tier.sh
./scripts/deploy-oracle-free-tier.sh
```

### **What the Script Does**

1. ✅ **System Update** - Updates Ubuntu and installs dependencies
2. ✅ **Docker Setup** - Installs Docker with ARM64 optimizations
3. ✅ **Repository Clone** - Downloads latest HFT Ninja code
4. ✅ **ARM64 Build** - Builds optimized containers
5. ✅ **Service Deploy** - Starts all services
6. ✅ **Health Check** - Verifies deployment

## ✅ **STEP 4: Verification & Testing (15 min)**

### **Check Service Health**

```bash
# Verify all services are running
curl http://localhost:8080/health  # HFT Ninja
curl http://localhost:8003/health  # DeepSeek-Math AI
curl http://localhost:3000         # React Dashboard
curl http://localhost:6379         # Redis (should connect)

# Check resource usage
docker stats --no-stream
```

### **Test AI Calculations**

```bash
# Test position sizing calculation
curl -X POST http://localhost:8003/calculate/position-size \
  -H "Content-Type: application/json" \
  -d '{
    "capital": 8.0,
    "risk_tolerance": 0.05,
    "expected_return": 0.15,
    "volatility": 0.3,
    "strategy": "wallet_tracker"
  }'

# Expected response:
# {
#   "result": {
#     "position_size": 0.8,
#     "risk_score": 0.25,
#     "confidence": 0.94
#   },
#   "latency_ms": 180,
#   "cost_usd": 0.000001
# }
```

### **Access Web Dashboard**

```bash
# Get your public IP
curl -s ifconfig.me

# Access dashboard at:
# http://YOUR_ORACLE_IP:3000
```

## 📊 **PERFORMANCE VERIFICATION**

### **Expected Metrics**

| Metric | Target | Typical | Status |
|--------|--------|---------|--------|
| **AI Latency** | <500ms | ~300ms | ✅ |
| **Trading Latency** | <100ms | ~80ms | ✅ |
| **Memory Usage** | <60% | ~40% | ✅ |
| **CPU Usage** | <80% | ~50% | ✅ |
| **Daily Cost** | $0 | $0 | ✅ |

### **Resource Monitor**

```bash
# Run continuous monitoring
./monitor-oracle.sh

# Sample output:
# === Oracle Free Tier Resource Monitor ===
# Memory Usage: 9.5 GB / 24 GB (40%)
# CPU Usage: 2.1 OCPU / 4 OCPU (52%)
# Disk Usage: 45 GB / 200 GB (22%)
# Network: 1.2 GB / 10 TB monthly (0.01%)
```

## 🌐 **STEP 5: Public Access Setup (Optional)**

### **Domain Configuration**

```bash
# Option 1: Use Oracle IP directly
http://YOUR_ORACLE_IP:3000

# Option 2: Setup custom domain with Cloudflare
# 1. Add domain to Cloudflare
# 2. Create A record pointing to Oracle IP
# 3. Enable SSL/TLS encryption
# 4. Configure caching rules
```

### **SSL Certificate (Let's Encrypt)**

```bash
# Install Certbot
sudo apt install certbot python3-certbot-nginx

# Get SSL certificate
sudo certbot --nginx -d your-domain.com

# Auto-renewal
sudo crontab -e
# Add: 0 12 * * * /usr/bin/certbot renew --quiet
```

## 🛠️ **TROUBLESHOOTING**

### **Common Issues**

1. **Instance Creation Failed**
   ```bash
   # Try different availability domain
   # Check quota limits in Oracle console
   # Wait 5-10 minutes and retry
   ```

2. **ARM64 Build Errors**
   ```bash
   # Verify architecture
   uname -m  # Must be aarch64
   
   # Clear Docker cache
   docker system prune -af
   
   # Rebuild with verbose output
   docker-compose -f docker-compose.oracle-arm.yml build --no-cache
   ```

3. **Memory Issues**
   ```bash
   # Check memory usage
   free -h
   
   # Restart services if needed
   docker-compose restart
   
   # Optimize memory settings
   echo 'vm.swappiness=10' | sudo tee -a /etc/sysctl.conf
   ```

4. **Network Connectivity**
   ```bash
   # Check security list rules
   # Verify ports are open
   netstat -tlnp | grep :8080
   
   # Test internal connectivity
   docker network ls
   ```

### **Performance Optimization**

```bash
# ARM64 system optimizations
echo 'vm.dirty_ratio = 5' | sudo tee -a /etc/sysctl.conf
echo 'vm.dirty_background_ratio = 2' | sudo tee -a /etc/sysctl.conf
echo 'net.core.rmem_max = 16777216' | sudo tee -a /etc/sysctl.conf
sudo sysctl -p

# Docker optimizations
sudo tee /etc/docker/daemon.json > /dev/null <<EOF
{
    "log-driver": "json-file",
    "log-opts": {
        "max-size": "10m",
        "max-file": "3"
    },
    "experimental": true,
    "features": {
        "buildkit": true
    }
}
EOF

sudo systemctl restart docker
```

## 📈 **SCALING & MAINTENANCE**

### **Backup Strategy**

```bash
# Create instance snapshot
# Oracle Console > Compute > Instances > Create Snapshot

# Backup configuration
tar -czf hft-ninja-backup.tar.gz \
    config/ \
    docker-compose.oracle-arm.yml \
    .env.oracle

# Upload to Oracle Object Storage (free 20 GB)
```

### **Update Procedure**

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Update HFT Ninja
cd solana-hft-ninja-2025.07
git pull origin main
docker-compose -f docker-compose.oracle-arm.yml pull
docker-compose -f docker-compose.oracle-arm.yml up -d
```

### **Monitoring Setup**

```bash
# Setup log rotation
sudo tee /etc/logrotate.d/docker > /dev/null <<EOF
/var/lib/docker/containers/*/*.log {
    rotate 7
    daily
    compress
    size=1M
    missingok
    delaycompress
    copytruncate
}
EOF

# Setup monitoring alerts
# Use Oracle Cloud Monitoring (free tier included)
```

## 🎉 **SUCCESS CHECKLIST**

- [ ] Oracle Free Tier account created
- [ ] ARM Ampere instance (4 OCPU + 24 GB) launched
- [ ] All services deployed and healthy
- [ ] AI calculations working (<300ms latency)
- [ ] Trading engine operational (<80ms latency)
- [ ] Dashboard accessible
- [ ] Resource usage <60% (sustainable)
- [ ] Monthly cost = $0

## 💡 **NEXT STEPS**

1. **Configure Trading Strategies** - Customize MEV strategies
2. **Setup Monitoring** - Implement comprehensive monitoring
3. **Domain & SSL** - Setup custom domain with SSL
4. **Backup Strategy** - Implement automated backups
5. **Performance Tuning** - Optimize for your trading patterns

---

**🚨 CONGRATULATIONS!** You now have enterprise-grade AI trading running for **$0/month** on Oracle Free Tier!
