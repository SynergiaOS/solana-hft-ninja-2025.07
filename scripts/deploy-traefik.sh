#!/bin/bash

# 🐳 Traefik v3 Deployment Script for Solana HFT Ninja
# Complete Docker-based reverse proxy with automatic SSL and container discovery

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DOMAIN="${DOMAIN:-hft-ninja.com}"
API_DOMAIN="${API_DOMAIN:-api.hft-ninja.com}"
EMAIL="${EMAIL:-admin@hft-ninja.com}"
ENVIRONMENT="${ENVIRONMENT:-production}"

echo -e "${BLUE}🐳 Deploying Traefik v3 for Solana HFT Ninja${NC}"
echo -e "${GREEN}Domain: $DOMAIN${NC}"
echo -e "${GREEN}API Domain: $API_DOMAIN${NC}"
echo -e "${GREEN}Email: $EMAIL${NC}"
echo -e "${GREEN}Environment: $ENVIRONMENT${NC}"
echo ""

# Function to print status
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    print_error "Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

# Create necessary directories
print_status "Creating directories..."
mkdir -p traefik/logs
mkdir -p traefik/certificates
mkdir -p monitoring/prometheus
mkdir -p monitoring/grafana/dashboards
mkdir -p monitoring/grafana/datasources
mkdir -p logs

# Set proper permissions
chmod 600 traefik/certificates 2>/dev/null || true

# Create environment file
print_status "Creating environment configuration..."
cat > .env.traefik << EOF
# Traefik Environment Configuration
DOMAIN=$DOMAIN
API_DOMAIN=$API_DOMAIN
EMAIL=$EMAIL
ENVIRONMENT=$ENVIRONMENT

# Solana Configuration
SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_WS_URL=wss://api.devnet.solana.com

# DragonflyDB Configuration
DRAGONFLY_URL=rediss://default:password@your-dragonfly-url:6385

# Security
TRAEFIK_DASHBOARD_PASSWORD=admin123
GRAFANA_ADMIN_PASSWORD=admin123

# Logging
LOG_LEVEL=INFO
EOF

# Create Prometheus configuration
print_status "Creating Prometheus configuration..."
cat > monitoring/prometheus/prometheus.yml << 'EOF'
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files: []

scrape_configs:
  - job_name: 'traefik'
    static_configs:
      - targets: ['traefik:8080']
    metrics_path: /metrics
    scrape_interval: 5s

  - job_name: 'hft-ninja'
    static_configs:
      - targets: ['hft-ninja:8080']
    metrics_path: /metrics
    scrape_interval: 10s

  - job_name: 'deepseek-math'
    static_configs:
      - targets: ['deepseek-math:8003']
    metrics_path: /metrics
    scrape_interval: 30s

  - job_name: 'cerebro-bff'
    static_configs:
      - targets: ['cerebro-bff:8002']
    metrics_path: /metrics
    scrape_interval: 15s

  - job_name: 'redis'
    static_configs:
      - targets: ['redis:6379']
    scrape_interval: 30s
EOF

# Create Grafana datasource
print_status "Creating Grafana datasource..."
cat > monitoring/grafana/datasources/prometheus.yml << 'EOF'
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: true
EOF

# Create basic Grafana dashboard
print_status "Creating Grafana dashboard..."
cat > monitoring/grafana/dashboards/dashboard.yml << 'EOF'
apiVersion: 1

providers:
  - name: 'default'
    orgId: 1
    folder: ''
    type: file
    disableDeletion: false
    updateIntervalSeconds: 10
    allowUiUpdates: true
    options:
      path: /etc/grafana/provisioning/dashboards
EOF

# Create health check script
print_status "Creating health check script..."
cat > scripts/traefik-health-check.sh << 'EOF'
#!/bin/bash

# Traefik Health Check Script
SERVICES=(
    "https://api.hft-ninja.com/health:API"
    "https://hft-ninja.com:Frontend"
    "http://localhost:8080/ping:Traefik"
)

LOG_FILE="/var/log/traefik-health.log"

check_service() {
    local url=$1
    local name=$2
    
    response=$(curl -s -o /dev/null -w "%{http_code}" --max-time 10 "$url")
    
    if [ "$response" = "200" ]; then
        echo "$(date): $name OK (HTTP $response)" | tee -a "$LOG_FILE"
        return 0
    else
        echo "$(date): $name FAILED (HTTP $response)" | tee -a "$LOG_FILE"
        return 1
    fi
}

echo "=== Traefik Health Check $(date) ==="

failed=0
for service in "${SERVICES[@]}"; do
    IFS=':' read -r url name <<< "$service"
    if ! check_service "$url" "$name"; then
        failed=$((failed + 1))
    fi
done

if [ $failed -eq 0 ]; then
    echo "✅ All services healthy"
    exit 0
else
    echo "❌ $failed services failed"
    exit 1
fi
EOF

chmod +x scripts/traefik-health-check.sh

# Stop existing containers
print_status "Stopping existing containers..."
docker-compose -f docker-compose.traefik.yml down 2>/dev/null || true

# Build and start services
print_status "Building and starting Traefik stack..."
docker-compose -f docker-compose.traefik.yml build --parallel
docker-compose -f docker-compose.traefik.yml up -d

# Wait for services to start
print_status "Waiting for services to start..."
sleep 30

# Check service status
print_status "Checking service status..."
services=(traefik deepseek-math cerebro-bff frontend redis prometheus grafana)

for service in "${services[@]}"; do
    if docker-compose -f docker-compose.traefik.yml ps | grep -q "$service.*Up"; then
        print_status "✅ $service is running"
    else
        print_warning "⚠️  $service is not running"
    fi
done

# Test endpoints
print_status "Testing endpoints..."
endpoints=(
    "http://localhost:8080/ping:Traefik Ping"
    "http://localhost:8080/dashboard/:Traefik Dashboard"
)

for endpoint in "${endpoints[@]}"; do
    IFS=':' read -r url name <<< "$endpoint"
    if curl -s -f "$url" > /dev/null; then
        print_status "✅ $name accessible"
    else
        print_warning "⚠️  $name not accessible"
    fi
done

# Create monitoring cron job
print_status "Setting up monitoring cron job..."
(crontab -l 2>/dev/null; echo "*/5 * * * * $(pwd)/scripts/traefik-health-check.sh") | crontab -

# Display final information
echo ""
echo -e "${GREEN}🎉 Traefik deployment completed!${NC}"
echo ""
echo -e "${BLUE}📋 Service URLs:${NC}"
echo "  • API: https://$API_DOMAIN"
echo "  • Frontend: https://$DOMAIN"
echo "  • Traefik Dashboard: https://traefik.$DOMAIN (admin/admin123)"
echo "  • Grafana: https://dashboard.$DOMAIN (admin/admin123)"
echo "  • Prometheus: https://metrics.$DOMAIN"
echo ""
echo -e "${BLUE}🔧 Management Commands:${NC}"
echo "  • View logs: docker-compose -f docker-compose.traefik.yml logs -f"
echo "  • Restart: docker-compose -f docker-compose.traefik.yml restart"
echo "  • Stop: docker-compose -f docker-compose.traefik.yml down"
echo "  • Health check: ./scripts/traefik-health-check.sh"
echo ""
echo -e "${BLUE}📊 Monitoring:${NC}"
echo "  • Traefik metrics: http://localhost:8080/metrics"
echo "  • Container stats: docker stats"
echo "  • Service status: docker-compose -f docker-compose.traefik.yml ps"
echo ""
echo -e "${YELLOW}⚠️  Next Steps:${NC}"
echo "  1. Update DNS records to point to this server"
echo "  2. Wait for SSL certificates to be issued (may take a few minutes)"
echo "  3. Test all endpoints: curl https://$API_DOMAIN/health"
echo "  4. Configure monitoring alerts"
echo "  5. Update default passwords in production"
echo ""
echo -e "${GREEN}🛡️ Your API is now protected by Traefik reverse proxy!${NC}"
echo -e "${GREEN}🔒 Automatic SSL certificates will be issued by Let's Encrypt${NC}"
echo -e "${GREEN}📊 Monitoring and metrics are available through Grafana${NC}"
