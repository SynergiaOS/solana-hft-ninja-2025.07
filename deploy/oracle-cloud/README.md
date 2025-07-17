# 🚀 Oracle Cloud Deployment Guide - Solana HFT Ninja 2025.07

Kompletny przewodnik deployment systemu HFT Ninja na Oracle Cloud Infrastructure (OCI).

## 📋 **Wymagania**

### **Oracle Cloud Infrastructure**
- **Compute Instance**: VM.Standard.E4.Flex (2 OCPU, 16GB RAM)
- **Block Storage**: 100GB Boot Volume + 200GB Block Volume
- **Network**: VCN z publicznym subnetem
- **Load Balancer**: Network Load Balancer (opcjonalnie)
- **DNS**: Oracle DNS lub zewnętrzny provider

### **Software Requirements**
- **Docker** 24.0+
- **Docker Compose** 2.0+
- **Git**
- **OpenSSL** (dla SSL certificates)
- **curl** (dla health checks)

## 🏗️ **Architektura Deployment**

```
Internet
    ↓
[Load Balancer] (Oracle Cloud)
    ↓
[Nginx Proxy] (SSL Termination)
    ↓
┌─────────────────────────────────────┐
│  Docker Network (172.20.0.0/16)    │
├─────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐   │
│  │ HFT Ninja   │  │ Frontend    │   │
│  │ (Rust)      │  │ (React)     │   │
│  │ Port: 8080  │  │ Port: 80    │   │
│  └─────────────┘  └─────────────┘   │
│  ┌─────────────┐  ┌─────────────┐   │
│  │ Prometheus  │  │ Grafana     │   │
│  │ Port: 9090  │  │ Port: 3000  │   │
│  └─────────────┘  └─────────────┘   │
│  ┌─────────────┐                    │
│  │ Redis       │                    │
│  │ Port: 6379  │                    │
│  └─────────────┘                    │
└─────────────────────────────────────┘
```

## 🚀 **Quick Start Deployment**

### **1. Przygotowanie serwera**
```bash
# Połącz się z Oracle Cloud instance
ssh -i ~/.ssh/oci_key ubuntu@your-instance-ip

# Aktualizuj system
sudo apt update && sudo apt upgrade -y

# Zainstaluj Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Zainstaluj Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Restart session
exit
ssh -i ~/.ssh/oci_key ubuntu@your-instance-ip
```

### **2. Clone repository**
```bash
git clone https://github.com/SynergiaOS/solana-hft-ninja-2025.07.git
cd solana-hft-ninja-2025.07/deploy/oracle-cloud
```

### **3. Konfiguracja**
```bash
# Skopiuj i edytuj konfigurację
cp .env.example .env
nano .env

# Ustaw swoje wartości:
HELIUS_KEY=your_helius_api_key
DOMAIN=your-domain.com
GRAFANA_ADMIN_PASSWORD=secure_password
```

### **4. Deploy**
```bash
# Uruchom deployment
./deploy.sh production deploy

# Sprawdź status
./deploy.sh production status
```

## ⚙️ **Konfiguracja szczegółowa**

### **Environment Variables (.env)**
```bash
# === PODSTAWOWA KONFIGURACJA ===
ENVIRONMENT=production
DOMAIN=hft.yourdomain.com

# === HELIUS API ===
HELIUS_KEY=your_helius_api_key_here

# === SOLANA CONFIGURATION ===
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
JITO_BLOCK_ENGINE_URL=https://mainnet.block-engine.jito.wtf

# === SECURITY ===
GRAFANA_ADMIN_PASSWORD=your_secure_password_here

# === SSL CERTIFICATES ===
SSL_CERT_PATH=./nginx/ssl/cert.pem
SSL_KEY_PATH=./nginx/ssl/key.pem

# === ORACLE CLOUD ===
OCI_REGION=us-ashburn-1
OCI_COMPARTMENT_ID=your_compartment_id
```

### **Firewall Rules (Oracle Cloud Security Lists)**
```bash
# HTTP/HTTPS
Ingress: 0.0.0.0/0 -> Port 80,443 (TCP)

# SSH (tylko z twojego IP)
Ingress: YOUR_IP/32 -> Port 22 (TCP)

# Monitoring (opcjonalnie, tylko z VPN)
Ingress: VPN_SUBNET -> Port 9090,3000 (TCP)
```

## 🔧 **Zarządzanie systemem**

### **Podstawowe komendy**
```bash
# Deploy/Update
./deploy.sh production deploy

# Restart wszystkich serwisów
./deploy.sh production restart

# Stop wszystkich serwisów
./deploy.sh production stop

# Sprawdź status
./deploy.sh production status

# Zobacz logi
./deploy.sh production logs

# Zobacz logi konkretnego serwisu
./deploy.sh production logs hft-ninja

# Cleanup (usuwa wszystko!)
./deploy.sh production cleanup
```

### **Monitoring i debugging**
```bash
# Sprawdź health wszystkich serwisów
curl -f https://your-domain.com/health

# Sprawdź metryki Prometheus
curl -f https://your-domain.com/metrics

# Sprawdź logi konkretnego kontenera
docker logs solana-hft-ninja-prod -f

# Sprawdź resource usage
docker stats

# Wejdź do kontenera
docker exec -it solana-hft-ninja-prod /bin/bash
```

## 🔐 **SSL/TLS Configuration**

### **Self-signed certificates (development)**
```bash
# Automatycznie generowane przez deploy.sh
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
    -keyout nginx/ssl/key.pem \
    -out nginx/ssl/cert.pem \
    -subj "/C=US/ST=State/L=City/O=Organization/CN=your-domain.com"
```

### **Let's Encrypt (production)**
```bash
# Zainstaluj certbot
sudo apt install certbot python3-certbot-nginx

# Wygeneruj certyfikat
sudo certbot --nginx -d your-domain.com

# Auto-renewal
sudo crontab -e
# Dodaj: 0 12 * * * /usr/bin/certbot renew --quiet
```

## 📊 **Monitoring URLs**

Po deployment system będzie dostępny pod:

- **🌐 Frontend**: https://your-domain.com
- **🔌 API**: https://your-domain.com/api
- **📊 Grafana**: https://your-domain.com/grafana
- **💓 Health**: https://your-domain.com/health
- **📈 Metrics**: https://your-domain.com/metrics (internal only)

## 🚨 **Troubleshooting**

### **Częste problemy**

1. **Containers nie startują**
   ```bash
   # Sprawdź logi
   docker-compose -f docker-compose.prod.yml logs
   
   # Sprawdź resources
   free -h && df -h
   ```

2. **SSL certificate errors**
   ```bash
   # Regeneruj certyfikaty
   rm -rf nginx/ssl/*
   ./deploy.sh production deploy
   ```

3. **Health check fails**
   ```bash
   # Sprawdź czy HFT service odpowiada
   docker exec solana-hft-ninja-prod curl -f http://localhost:8080/health
   ```

4. **High memory usage**
   ```bash
   # Restart serwisów
   ./deploy.sh production restart
   
   # Sprawdź memory leaks
   docker stats --no-stream
   ```

## 🔄 **Updates i Maintenance**

### **Update aplikacji**
```bash
# Pull latest changes
git pull origin main

# Rebuild i redeploy
./deploy.sh production deploy
```

### **Backup danych**
```bash
# Backup Grafana dashboards
docker exec grafana-prod tar czf - /var/lib/grafana | gzip > grafana-backup-$(date +%Y%m%d).tar.gz

# Backup Prometheus data
docker exec prometheus-prod tar czf - /prometheus | gzip > prometheus-backup-$(date +%Y%m%d).tar.gz
```

### **Monitoring resource usage**
```bash
# Setup monitoring alerts
# TODO: Implement Oracle Cloud Monitoring integration
```

## 🎯 **Production Checklist**

- [ ] Oracle Cloud instance configured
- [ ] Domain DNS pointing to instance
- [ ] SSL certificates configured
- [ ] Helius API key set
- [ ] Firewall rules configured
- [ ] Monitoring dashboards working
- [ ] Health checks passing
- [ ] Backup strategy implemented
- [ ] Log rotation configured
- [ ] Resource monitoring setup

## 📞 **Support**

W przypadku problemów:
1. Sprawdź logi: `./deploy.sh production logs`
2. Sprawdź status: `./deploy.sh production status`
3. Sprawdź dokumentację Oracle Cloud
4. Otwórz issue na GitHub

**System gotowy do production deployment! 🚀**
