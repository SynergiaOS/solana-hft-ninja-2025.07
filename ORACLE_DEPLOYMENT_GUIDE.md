# 🧠 Cerberus Oracle Cloud Deployment Guide

## 🎯 **COMPLETE ENTERPRISE DEPLOYMENT**

This guide shows how to deploy **Cerberus Trade Execution Brain** to Oracle Cloud Free Tier with enterprise-grade security, monitoring, and Cloudflare protection.

---

## 🏗️ **ARCHITECTURE OVERVIEW**

```
Internet → Cloudflare → Oracle Cloud (Free Tier) → Traefik → Services
         ↓
    🛡️ DDoS Protection
    🔒 SSL Termination  
    🌐 Global CDN
                        ↓
                   📍 121044141.dns.cerberusso.tech
                        ↓
                   🔀 Traefik Reverse Proxy
                        ↓
        ┌─────────────────┼─────────────────┐
        ▼                 ▼                 ▼
    🧠 Cerberus      📊 Grafana       🤖 Cerebro AI
    (Position Mgmt)  (Monitoring)     (Intelligence)
        ▼                 ▼                 ▼
    📊 Redis         📈 Prometheus    🎯 HFT Engine
    (Storage)        (Metrics)       (Trading)
```

---

## 🚀 **DEPLOYMENT STEPS**

### **1. Prerequisites**

```bash
# Set environment variables
export QUICKNODE_ENDPOINT="https://your-endpoint.quiknode.pro/your-key/"
export HELIUS_ENDPOINT="https://mainnet.helius-rpc.com/?api-key=your-key"
export SOLANA_PRIVATE_KEY='[your,private,key,array]'

# Verify SSH access to Oracle
ssh opc@121044141.dns.cerberusso.tech
```

### **2. Deploy to Oracle Cloud**

```bash
# Run automated deployment
./scripts/deploy-cerberus-oracle.sh

# Or step by step:
./scripts/deploy-cerberus-oracle.sh deploy   # Full deployment
./scripts/deploy-cerberus-oracle.sh test     # Run tests only
./scripts/deploy-cerberus-oracle.sh monitor  # Setup monitoring
./scripts/deploy-cerberus-oracle.sh status   # Check status
```

### **3. Configure Cloudflare DNS**

Add these DNS records in your Cloudflare dashboard:

| Type | Name | Target | Proxy |
|------|------|--------|-------|
| CNAME | origin | 121044141.dns.cerberusso.tech | 🔘 DNS Only |
| CNAME | app | origin.cerberusso.tech | 🟠 Proxied |
| CNAME | api | origin.cerberusso.tech | 🟠 Proxied |
| CNAME | cerberus | origin.cerberusso.tech | 🟠 Proxied |
| CNAME | grafana | origin.cerberusso.tech | 🟠 Proxied |
| CNAME | ai | origin.cerberusso.tech | 🟠 Proxied |
| CNAME | dashboard | origin.cerberusso.tech | 🟠 Proxied |

### **4. SSL/TLS Configuration**

1. Go to **SSL/TLS → Overview**
2. Set mode to **Full (strict)**
3. Enable **Always Use HTTPS**
4. Enable **HSTS** with 6 months

---

## 🔗 **ACCESS POINTS**

### **Public Endpoints (via Cloudflare)**
- 🎯 **Main App**: https://app.cerberusso.tech
- 🔌 **API**: https://api.cerberusso.tech
- 🧠 **Cerberus**: https://cerberus.cerberusso.tech
- 📊 **Grafana**: https://grafana.cerberusso.tech
- 🤖 **Cerebro AI**: https://ai.cerberusso.tech
- 📱 **Dashboard**: https://dashboard.cerberusso.tech

### **Direct Access (for debugging)**
- 🏠 **Origin**: https://origin.cerberusso.tech
- 🔀 **Traefik**: https://origin.cerberusso.tech:8080

---

## 🛡️ **SECURITY FEATURES**

### **Chainguard Hardening**
- ✅ Zero-vulnerability base images
- ✅ Distroless containers where possible
- ✅ Non-root execution by default
- ✅ Minimal attack surface

### **Container Security**
- ✅ Read-only filesystems
- ✅ Dropped capabilities (CAP_DROP: ALL)
- ✅ No new privileges
- ✅ Security contexts

### **Network Security**
- ✅ Cloudflare DDoS protection
- ✅ SSL/TLS encryption end-to-end
- ✅ Isolated Docker networks
- ✅ Traefik reverse proxy

---

## 📊 **MONITORING & MANAGEMENT**

### **Health Monitoring**
```bash
# SSH to Oracle instance
ssh opc@121044141.dns.cerberusso.tech

# Run health check
cd ~/cerberus-deployment && ./monitor-cerberus.sh

# View real-time logs
docker-compose -f docker-compose.oracle.yml logs -f cerberus
```

### **Grafana Dashboards**
- **URL**: https://grafana.cerberusso.tech
- **Login**: admin / cerberus2025
- **Metrics**: Trading performance, system health, position tracking

### **Prometheus Metrics**
- **URL**: https://metrics.cerberusso.tech
- **Data**: Real-time metrics from all services
- **Retention**: 200 hours

---

## 🧠 **CERBERUS OPERATIONS**

### **Position Management**
```bash
# Check active positions
docker exec redis-hardened redis-cli scard active_positions

# Send AI signal
docker exec redis-hardened redis-cli publish cerebro_commands '{
  "action": "SELL",
  "mint": "So11111111111111111111111111111111111111112",
  "reason": "AI_BEARISH_SIGNAL"
}'

# Emergency stop all positions
docker exec redis-hardened redis-cli publish guardian_alerts '{
  "action": "EXIT_ALL_POSITIONS",
  "reason": "MANUAL_STOP"
}'
```

### **Configuration Updates**
```bash
# Update Cerberus config
ssh opc@121044141.dns.cerberusso.tech
cd ~/cerberus-deployment
nano config/cerberus.toml

# Restart services
docker-compose -f docker-compose.oracle.yml restart cerberus
```

---

## 🔧 **TROUBLESHOOTING**

### **Common Issues**

**1. SSL Certificate Issues**
```bash
# Check certificate status
docker exec traefik-hardened cat /letsencrypt/acme.json

# Force certificate renewal
docker-compose -f docker-compose.oracle.yml restart traefik
```

**2. Cerberus Not Starting**
```bash
# Check logs
docker logs cerberus-hardened

# Verify environment variables
docker exec cerberus-hardened env | grep -E "(QUICKNODE|HELIUS|SOLANA)"
```

**3. Redis Connection Issues**
```bash
# Test Redis connectivity
docker exec redis-hardened redis-cli ping

# Check Redis logs
docker logs redis-hardened
```

### **Performance Optimization**

**1. Oracle Cloud Resources**
- **CPU**: 4 OCPU (ARM Ampere)
- **Memory**: 24 GB RAM
- **Storage**: 200 GB Block Volume
- **Network**: 480 Mbps

**2. Container Resource Limits**
```yaml
# Add to docker-compose.oracle.yml
deploy:
  resources:
    limits:
      cpus: '1.0'
      memory: 2G
    reservations:
      cpus: '0.5'
      memory: 1G
```

---

## 📈 **SCALING & OPTIMIZATION**

### **Horizontal Scaling**
- Add more Oracle instances
- Use Docker Swarm or Kubernetes
- Load balance with Traefik

### **Vertical Scaling**
- Upgrade to paid Oracle Cloud tiers
- Increase container resource limits
- Optimize database queries

### **Performance Monitoring**
- Set up alerts in Grafana
- Monitor key metrics:
  - Decision latency (<200ms)
  - Execution latency (<100ms)
  - Position count
  - Memory usage
  - CPU utilization

---

## 🎯 **PRODUCTION CHECKLIST**

- [ ] ✅ Oracle Cloud instance configured
- [ ] ✅ Cloudflare DNS records added
- [ ] ✅ SSL/TLS certificates working
- [ ] ✅ All services running
- [ ] ✅ Monitoring configured
- [ ] ✅ Health checks passing
- [ ] ✅ Environment variables set
- [ ] ✅ Backup procedures in place
- [ ] ✅ Emergency procedures tested
- [ ] ✅ Performance baselines established

---

## 🚨 **EMERGENCY PROCEDURES**

### **Immediate Actions**
1. **Stop Trading**: Emergency stop via Redis command
2. **Check Positions**: Verify all positions closed
3. **Review Logs**: Check for errors or issues
4. **Contact Support**: If needed

### **Recovery Steps**
1. **Restart Services**: `docker-compose restart`
2. **Check Configuration**: Verify all settings
3. **Test Connectivity**: Ensure RPC endpoints working
4. **Resume Trading**: Only after verification

---

**🧠 Cerberus is now deployed and ready to guard your positions with enterprise-grade security and monitoring!**

*Deployment completed with zero-cost Oracle Cloud Free Tier + Cloudflare protection*
