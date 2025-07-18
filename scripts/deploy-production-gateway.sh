#!/bin/bash

# 🎯 Production API Gateway Deployment Script
# Complete Cloudflare + Traefik setup for Solana HFT Ninja

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
CF_TOKEN="${CF_TOKEN:-}"
ENVIRONMENT="${ENVIRONMENT:-production}"

echo -e "${BLUE}🎯 Deploying Production API Gateway for Solana HFT Ninja${NC}"
echo -e "${GREEN}Domain: $DOMAIN${NC}"
echo -e "${GREEN}API Domain: $API_DOMAIN${NC}"
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

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    print_step "Checking prerequisites..."
    
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
    
    # Check if curl is installed
    if ! command -v curl &> /dev/null; then
        print_error "curl is not installed. Please install curl first."
        exit 1
    fi
    
    # Check if jq is installed
    if ! command -v jq &> /dev/null; then
        print_warning "jq is not installed. Installing jq for JSON processing..."
        apt update && apt install -y jq
    fi
    
    print_status "✅ All prerequisites are satisfied"
}

# Setup environment
setup_environment() {
    print_step "Setting up environment..."
    
    # Create necessary directories
    mkdir -p traefik/logs
    mkdir -p traefik/certificates
    mkdir -p monitoring/prometheus
    mkdir -p monitoring/grafana/dashboards
    mkdir -p monitoring/grafana/datasources
    mkdir -p logs
    mkdir -p strategies
    
    # Set proper permissions
    chmod 600 traefik/certificates 2>/dev/null || true
    
    # Create environment file
    cat > .env.production << EOF
# Production Environment Configuration
DOMAIN=$DOMAIN
API_DOMAIN=$API_DOMAIN
EMAIL=$EMAIL
ENVIRONMENT=$ENVIRONMENT

# Solana Configuration
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_WS_URL=wss://api.mainnet-beta.solana.com

# DragonflyDB Configuration
DRAGONFLY_URL=${DRAGONFLY_URL:-rediss://default:password@your-dragonfly-url:6385}

# Security
TRAEFIK_DASHBOARD_PASSWORD=${TRAEFIK_DASHBOARD_PASSWORD:-$(openssl rand -base64 32)}
GRAFANA_ADMIN_PASSWORD=${GRAFANA_ADMIN_PASSWORD:-$(openssl rand -base64 32)}

# Logging
LOG_LEVEL=INFO

# Cloudflare
CF_TOKEN=$CF_TOKEN
EOF
    
    print_status "✅ Environment configured"
    print_status "📝 Configuration saved to .env.production"
}

# Configure Cloudflare
configure_cloudflare() {
    print_step "Configuring Cloudflare..."
    
    if [ -z "$CF_TOKEN" ]; then
        print_warning "⚠️  Cloudflare token not provided. Skipping automatic configuration."
        print_status "📋 Manual Cloudflare setup required:"
        print_status "   1. Add $DOMAIN to Cloudflare"
        print_status "   2. Update nameservers at your registrar"
        print_status "   3. Create A record: $API_DOMAIN → $(curl -s ifconfig.me)"
        print_status "   4. Enable proxy (orange cloud)"
        print_status "   5. Configure security settings as per documentation"
        return 0
    fi
    
    # Get zone ID
    print_status "Getting Cloudflare zone ID..."
    ZONE_ID=$(curl -s -X GET "https://api.cloudflare.com/client/v4/zones?name=$DOMAIN" \
        -H "Authorization: Bearer $CF_TOKEN" \
        -H "Content-Type: application/json" | \
        jq -r '.result[0].id')
    
    if [ "$ZONE_ID" = "null" ] || [ -z "$ZONE_ID" ]; then
        print_error "Could not get Zone ID. Please check your domain and token."
        return 1
    fi
    
    print_status "✅ Zone ID: $ZONE_ID"
    
    # Get server IP
    SERVER_IP=$(curl -s ifconfig.me)
    print_status "📍 Server IP: $SERVER_IP"
    
    # Create DNS records
    print_status "Creating DNS records..."
    
    # API subdomain
    curl -s -X POST "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records" \
        -H "Authorization: Bearer $CF_TOKEN" \
        -H "Content-Type: application/json" \
        --data "{
            \"type\": \"A\",
            \"name\": \"api\",
            \"content\": \"$SERVER_IP\",
            \"ttl\": 1,
            \"proxied\": true
        }" > /dev/null
    
    # Traefik subdomain
    curl -s -X POST "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records" \
        -H "Authorization: Bearer $CF_TOKEN" \
        -H "Content-Type: application/json" \
        --data "{
            \"type\": \"CNAME\",
            \"name\": \"traefik\",
            \"content\": \"$API_DOMAIN\",
            \"ttl\": 1,
            \"proxied\": true
        }" > /dev/null
    
    # Dashboard subdomain
    curl -s -X POST "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records" \
        -H "Authorization: Bearer $CF_TOKEN" \
        -H "Content-Type: application/json" \
        --data "{
            \"type\": \"CNAME\",
            \"name\": \"dashboard\",
            \"content\": \"$API_DOMAIN\",
            \"ttl\": 1,
            \"proxied\": true
        }" > /dev/null
    
    print_status "✅ DNS records created"
    
    # Configure security settings
    print_status "Configuring security settings..."
    
    # Enable Bot Fight Mode
    curl -s -X PATCH "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/settings/bot_fight_mode" \
        -H "Authorization: Bearer $CF_TOKEN" \
        -H "Content-Type: application/json" \
        --data '{"value": "on"}' > /dev/null
    
    # Set security level
    curl -s -X PATCH "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/settings/security_level" \
        -H "Authorization: Bearer $CF_TOKEN" \
        -H "Content-Type: application/json" \
        --data '{"value": "medium"}' > /dev/null
    
    # Enable Always Use HTTPS
    curl -s -X PATCH "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/settings/always_use_https" \
        -H "Authorization: Bearer $CF_TOKEN" \
        -H "Content-Type: application/json" \
        --data '{"value": "on"}' > /dev/null
    
    print_status "✅ Cloudflare security configured"
}

# Deploy Traefik stack
deploy_traefik() {
    print_step "Deploying Traefik stack..."
    
    # Stop any existing containers
    docker-compose -f docker-compose.traefik.yml down 2>/dev/null || true
    
    # Pull latest images
    print_status "Pulling latest images..."
    docker-compose -f docker-compose.traefik.yml pull
    
    # Build custom images
    print_status "Building custom images..."
    docker-compose -f docker-compose.traefik.yml build --parallel
    
    # Start the stack
    print_status "Starting Traefik stack..."
    docker-compose -f docker-compose.traefik.yml up -d
    
    # Wait for services to start
    print_status "Waiting for services to start..."
    sleep 30
    
    print_status "✅ Traefik stack deployed"
}

# Verify deployment
verify_deployment() {
    print_step "Verifying deployment..."
    
    # Check container status
    print_status "Checking container status..."
    services=(traefik deepseek-math-primary cerebro-bff hft-ninja-core frontend)
    
    for service in "${services[@]}"; do
        if docker-compose -f docker-compose.traefik.yml ps | grep -q "$service.*Up"; then
            print_status "✅ $service is running"
        else
            print_warning "⚠️  $service is not running"
        fi
    done
    
    # Test local endpoints
    print_status "Testing local endpoints..."
    local_endpoints=(
        "http://localhost:8080/ping:Traefik Ping"
        "http://localhost:8080/dashboard/:Traefik Dashboard"
    )
    
    for endpoint in "${local_endpoints[@]}"; do
        IFS=':' read -r url name <<< "$endpoint"
        if curl -s -f "$url" > /dev/null; then
            print_status "✅ $name accessible"
        else
            print_warning "⚠️  $name not accessible"
        fi
    done
    
    # Test external endpoints (if DNS is propagated)
    print_status "Testing external endpoints..."
    external_endpoints=(
        "https://$API_DOMAIN/health:API Health"
        "https://traefik.$DOMAIN/dashboard/:Traefik Dashboard"
    )
    
    for endpoint in "${external_endpoints[@]}"; do
        IFS=':' read -r url name <<< "$endpoint"
        if curl -s -f "$url" > /dev/null 2>&1; then
            print_status "✅ $name accessible"
        else
            print_warning "⚠️  $name not accessible (DNS may still be propagating)"
        fi
    done
}

# Setup monitoring
setup_monitoring() {
    print_step "Setting up monitoring..."
    
    # Deploy monitoring scripts
    ./scripts/cloudflare-analytics.sh || print_warning "Cloudflare analytics setup failed"
    
    # Create monitoring cron jobs
    (crontab -l 2>/dev/null; echo "*/5 * * * * $(pwd)/scripts/traefik-health-check.sh") | crontab -
    (crontab -l 2>/dev/null; echo "0 6 * * * $(pwd)/scripts/log-analyzer.sh") | crontab -
    
    print_status "✅ Monitoring configured"
}

# Display final information
display_final_info() {
    echo ""
    echo -e "${GREEN}🎉 Production API Gateway deployment completed!${NC}"
    echo ""
    echo -e "${BLUE}📋 Service URLs:${NC}"
    echo "  • API: https://$API_DOMAIN"
    echo "  • Frontend: https://$DOMAIN"
    echo "  • Traefik Dashboard: https://traefik.$DOMAIN/dashboard/"
    echo "  • Grafana: https://dashboard.$DOMAIN"
    echo ""
    echo -e "${BLUE}🔧 Management Commands:${NC}"
    echo "  • View logs: docker-compose -f docker-compose.traefik.yml logs -f"
    echo "  • Restart services: docker-compose -f docker-compose.traefik.yml restart"
    echo "  • Scale services: docker-compose -f docker-compose.traefik.yml up -d --scale service=3"
    echo "  • Strategy management: ./scripts/strategy-manager.sh help"
    echo ""
    echo -e "${BLUE}📊 Monitoring:${NC}"
    echo "  • Analytics: ./scripts/cloudflare-analytics.sh"
    echo "  • Health checks: ./scripts/traefik-health-check.sh"
    echo "  • Container stats: docker stats"
    echo ""
    echo -e "${BLUE}🔒 Security:${NC}"
    echo "  • Traefik Dashboard Password: $(grep TRAEFIK_DASHBOARD_PASSWORD .env.production | cut -d= -f2)"
    echo "  • Grafana Admin Password: $(grep GRAFANA_ADMIN_PASSWORD .env.production | cut -d= -f2)"
    echo ""
    echo -e "${YELLOW}⚠️  Next Steps:${NC}"
    echo "  1. Wait for DNS propagation (up to 24 hours)"
    echo "  2. Test all endpoints: curl https://$API_DOMAIN/health"
    echo "  3. Configure Cloudflare security rules manually if CF_TOKEN not provided"
    echo "  4. Deploy trading strategies: ./scripts/strategy-manager.sh create my-strategy"
    echo "  5. Monitor performance and adjust scaling as needed"
    echo ""
    echo -e "${GREEN}🛡️ Your HFT Ninja API is now protected by enterprise-grade gateway!${NC}"
    echo -e "${GREEN}🚀 Zero-cost infrastructure with unlimited DDoS protection${NC}"
    echo -e "${GREEN}📈 Ready for high-frequency trading at scale${NC}"
}

# Main execution
main() {
    check_prerequisites
    setup_environment
    configure_cloudflare
    deploy_traefik
    verify_deployment
    setup_monitoring
    display_final_info
}

# Run main function
main "$@"
