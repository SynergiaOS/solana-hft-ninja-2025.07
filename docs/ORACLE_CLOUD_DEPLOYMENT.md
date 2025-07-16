# 🌐 Oracle Cloud Deployment Guide - Solana HFT Ninja

## 🎯 Your Oracle Cloud Configuration

**Network Details:**
- **Private IP**: `10.0.0.59`
- **Internal DNS**: `subnet07161247.vcn07161247.oraclevcn.com`
- **Network Type**: Oracle VCN (Virtual Cloud Network)

## 🚀 Quick Deployment (3 Steps)

### Step 1: Connect to Oracle Instance
```bash
# Connect via SSH
ssh opc@10.0.0.59

# Or via bastion host (if configured)
ssh -J bastion_user@bastion.oraclevcn.com opc@10.0.0.59
```

### Step 2: Deploy HFT Ninja with Infisical
```bash
# Clone and setup
git clone https://github.com/your-repo/solana-hft-ninja-2025.07.git
cd solana-hft-ninja-2025.07

# Setup Infisical environment
echo "INFISICAL_TOKEN=st.7ab7091a-ae4f-41ba-b31c-bde5bafa4599.47542cb1d455d61335eaca92b2f6abfa.941bf8d2786836054e1fec510dd5f86b" > .env.local
echo "INFISICAL_PROJECT_ID=73c2f3cb-c922-4a46-a333-7b96fbc6301a" >> .env.local
echo "INFISICAL_ENVIRONMENT=production" >> .env.local

# Build and run
cargo build --release --bin hft_main
nohup ./scripts/run-with-infisical.sh > hft.log 2>&1 &
```

### Step 3: Access Dashboard
```bash
# Dashboard available at:
# http://10.0.0.59:8080/health
# http://10.0.0.59:8080/metrics
# http://ninja.subnet07161247.vcn07161247.oraclevcn.com:8080 (if DNS configured)
```

## 🛡️ Oracle VCN Security Configuration

### Security List Rules (Inbound)
```bash
# Essential ports for HFT operations
Source CIDR: 0.0.0.0/0 → Port: 8080 (Metrics Dashboard)
Source CIDR: 10.0.0.0/16 → Port: 9100 (Prometheus)
Source CIDR: VALIDATOR_IPS → Port: 8900 (Solana RPC)
Source CIDR: 10.0.0.0/16 → Port: 22 (SSH)
```

### Security List Rules (Outbound)
```bash
# Allow all outbound for Helius/Solana connections
Destination: 0.0.0.0/0 → All Ports (HTTPS/WSS to Helius)
Destination: 10.0.0.0/16 → All Ports (VCN internal)
```

## ⚡ Performance Optimization for Oracle Cloud

### System Configuration
```bash
# Add to /etc/sysctl.conf for HFT performance
sudo tee -a /etc/sysctl.conf << EOF
# HFT Network Optimization
net.core.rmem_max = 2147483647
net.core.wmem_max = 2147483647
net.ipv4.tcp_rmem = 4096 87380 2147483647
net.ipv4.tcp_wmem = 4096 65536 2147483647
net.ipv4.udp_mem = 3145728 4194304 16777216
net.core.netdev_max_backlog = 30000
net.ipv4.tcp_congestion_control = bbr
EOF

# Apply settings
sudo sysctl -p
```

### Oracle-Specific Optimizations
```bash
# Disable Oracle Cloud Agent (reduces CPU overhead)
sudo systemctl stop oracle-cloud-agent
sudo systemctl disable oracle-cloud-agent

# Optimize for bare metal instances
echo 'performance' | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Set CPU affinity for HFT process
taskset -c 0-7 ./target/release/hft_main
```

## 🔒 Security Hardening

### Firewall Configuration
```bash
# Configure iptables for HFT security
sudo iptables -A INPUT -p tcp --dport 8080 -s 10.0.0.0/16 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 8080 -j DROP
sudo iptables -A INPUT -p tcp --dport 8899 -s 169.254.169.254 -j DROP
sudo iptables -A OUTPUT -d 10.0.0.0/8 -j ACCEPT

# Save rules
sudo iptables-save > /etc/iptables/rules.v4
```

### SELinux Configuration
```bash
# Set SELinux to enforcing for production
sudo setenforce 1
sudo sed -i 's/SELINUX=.*/SELINUX=enforcing/' /etc/selinux/config
```

## 📊 Monitoring Setup

### Performance Validation
```bash
# Test Solana RPC latency
time curl -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getVersion"}' \
  https://api.mainnet-beta.solana.com

# Network jitter test
ping -c 100 10.0.0.1 | awk -F '/' 'END {print "Avg: " $5 "ms | Jitter: " $7 "ms"}'

# Check HFT application status
curl -s http://10.0.0.59:8080/health | jq .
```

### Grafana Dashboard Access
```bash
# If using full monitoring stack
docker-compose -f docker-compose.yml -f docker-compose.monitoring.yml up -d

# Access points:
# Grafana: http://10.0.0.59:3000
# Prometheus: http://10.0.0.59:9090
# HFT Metrics: http://10.0.0.59:8080/metrics
```

## 🚨 Emergency Procedures

### Kill Switch Sequence
```bash
#!/bin/bash
# Emergency shutdown script

echo "🚨 EMERGENCY SHUTDOWN INITIATED"

# 1. Stop HFT application
pkill -f hft_main

# 2. Block all network traffic
sudo iptables -P INPUT DROP
sudo iptables -P OUTPUT DROP

# 3. Secure sensitive data
sudo shred -uz /opt/hft/config/*.json

echo "✅ Emergency shutdown complete"
```

### Compromise Detection
```bash
# Monitor for suspicious activity
sudo tail -f /var/log/auth.log | grep -E 'Failed|Invalid'

# Check HFT process integrity
ps aux | grep hft_main
netstat -tulpn | grep :8080
```

## 🔧 Oracle Cloud Specific Commands

### Instance Management
```bash
# Get instance metadata
curl -H "Authorization: Bearer Oracle" http://169.254.169.254/opc/v2/instance/

# Monitor instance metrics
oci monitoring metric-data summarize-metrics-data \
  --namespace oci_computeagent \
  --compartment-id <your-compartment-id>
```

### Network Diagnostics
```bash
# Test VCN connectivity
ping -c 5 10.0.0.1  # VCN gateway
nslookup subnet07161247.vcn07161247.oraclevcn.com

# Check security list rules
oci network security-list list --compartment-id <compartment-id>
```

## 📈 Production Deployment Checklist

### Pre-Deployment
- [ ] Oracle instance optimized (BM.Optimized3.128 recommended)
- [ ] VCN security lists configured
- [ ] Infisical secrets configured
- [ ] System performance tuned
- [ ] Monitoring stack deployed

### Post-Deployment
- [ ] Health check responding: `curl http://10.0.0.59:8080/health`
- [ ] Metrics collecting: `curl http://10.0.0.59:8080/metrics`
- [ ] Helius WebSocket connected
- [ ] MEV strategies active
- [ ] Jito bundles enabled

### Monitoring
- [ ] Grafana dashboard accessible
- [ ] Prometheus scraping metrics
- [ ] Log aggregation working
- [ ] Alert rules configured

## 🎯 Quick Commands Reference

```bash
# Start HFT Ninja with Infisical
./scripts/run-with-infisical.sh

# Check status
curl http://10.0.0.59:8080/health

# View logs
tail -f hft.log

# Stop application
pkill -f hft_main

# Restart with new configuration
./scripts/run-with-infisical.sh
```

---

**🥷 Your Solana HFT Ninja is now optimized for Oracle Cloud Infrastructure!**
