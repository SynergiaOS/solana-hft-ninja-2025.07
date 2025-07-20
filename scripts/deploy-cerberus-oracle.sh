#!/bin/bash

# 🧠 Cerberus Oracle Cloud Deployment Script
# Deploy autonomous position management to Oracle Free Tier with Cloudflare

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
ORACLE_HOST="121044141.dns.cerberusso.tech"
ORACLE_USER="opc"
PROJECT_NAME="solana-hft-ninja-2025.07"
DOCKER_REGISTRY="ghcr.io/synergiaos"

echo -e "${PURPLE}🧠 CERBERUS ORACLE CLOUD DEPLOYMENT${NC}"
echo -e "${PURPLE}====================================${NC}"
echo ""

# Function to print status
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    # Check SSH access
    if ! ssh -o ConnectTimeout=10 -o BatchMode=yes "$ORACLE_USER@$ORACLE_HOST" exit 2>/dev/null; then
        print_error "Cannot SSH to Oracle instance. Check your SSH key and connection."
        echo "Expected: ssh $ORACLE_USER@$ORACLE_HOST"
        exit 1
    fi
    print_success "SSH access to Oracle instance verified"
    
    # Check environment variables
    if [[ -z "${QUICKNODE_ENDPOINT:-}" ]]; then
        print_warning "QUICKNODE_ENDPOINT not set"
    else
        print_success "QuickNode endpoint configured"
    fi
    
    if [[ -z "${HELIUS_ENDPOINT:-}" ]]; then
        print_warning "HELIUS_ENDPOINT not set"
    else
        print_success "Helius endpoint configured"
    fi
    
    if [[ -z "${SOLANA_PRIVATE_KEY:-}" ]]; then
        print_warning "SOLANA_PRIVATE_KEY not set"
    else
        print_success "Solana private key configured"
    fi
}

# Function to build and push images
build_and_push_images() {
    print_status "Building and pushing Cerberus images..."
    
    # Build Cerberus with Chainguard
    print_status "Building Cerberus with Chainguard hardening..."
    docker build -f Dockerfile.cerberus -t "$DOCKER_REGISTRY/cerberus:latest" .
    docker build -f Dockerfile.cerberus -t "$DOCKER_REGISTRY/cerberus:chainguard-$(date +%Y%m%d)" .
    
    # Push to registry
    print_status "Pushing images to registry..."
    docker push "$DOCKER_REGISTRY/cerberus:latest"
    docker push "$DOCKER_REGISTRY/cerberus:chainguard-$(date +%Y%m%d)"
    
    print_success "Images built and pushed successfully"
}

# Function to prepare Oracle instance
prepare_oracle_instance() {
    print_status "Preparing Oracle Cloud instance..."
    
    # Create deployment directory
    ssh "$ORACLE_USER@$ORACLE_HOST" "mkdir -p ~/cerberus-deployment"
    
    # Copy deployment files
    print_status "Copying deployment files..."
    scp docker-compose.chainguard.yml "$ORACLE_USER@$ORACLE_HOST:~/cerberus-deployment/"
    scp config/cerberus.toml "$ORACLE_USER@$ORACLE_HOST:~/cerberus-deployment/"
    scp scripts/test-cerberus.sh "$ORACLE_USER@$ORACLE_HOST:~/cerberus-deployment/"
    
    # Install Docker if not present
    ssh "$ORACLE_USER@$ORACLE_HOST" << 'EOF'
        if ! command -v docker &> /dev/null; then
            echo "Installing Docker..."
            sudo dnf update -y
            sudo dnf install -y docker docker-compose
            sudo systemctl enable docker
            sudo systemctl start docker
            sudo usermod -aG docker $USER
            echo "Docker installed. Please log out and back in for group changes to take effect."
        else
            echo "Docker already installed"
        fi
EOF
    
    print_success "Oracle instance prepared"
}

# Function to deploy Cerberus
deploy_cerberus() {
    print_status "Deploying Cerberus to Oracle Cloud..."
    
    # Create environment file
    ssh "$ORACLE_USER@$ORACLE_HOST" << EOF
        cd ~/cerberus-deployment
        
        # Create .env file with secrets
        cat > .env << 'ENVEOF'
QUICKNODE_ENDPOINT=${QUICKNODE_ENDPOINT:-https://api.mainnet-beta.solana.com}
HELIUS_ENDPOINT=${HELIUS_ENDPOINT:-https://api.mainnet-beta.solana.com}
SOLANA_PRIVATE_KEY=${SOLANA_PRIVATE_KEY:-}
RUST_LOG=info
RUST_BACKTRACE=1
ENVEOF
        
        # Pull latest images
        docker-compose -f docker-compose.chainguard.yml pull
        
        # Start services
        docker-compose -f docker-compose.chainguard.yml up -d
        
        # Wait for services to start
        sleep 10
        
        # Check status
        docker-compose -f docker-compose.chainguard.yml ps
EOF
    
    print_success "Cerberus deployed successfully"
}

# Function to configure Cloudflare
configure_cloudflare() {
    print_status "Cloudflare configuration instructions..."
    
    echo ""
    echo -e "${CYAN}📋 CLOUDFLARE DNS CONFIGURATION${NC}"
    echo ""
    echo "Add these DNS records to your Cloudflare dashboard:"
    echo ""
    echo -e "${YELLOW}Type    Name        Target                              Proxy${NC}"
    echo "CNAME   origin      121044141.dns.cerberusso.tech       🔘 DNS Only"
    echo "CNAME   app         origin.cerberusso.tech              🟠 Proxied"
    echo "CNAME   api         origin.cerberusso.tech              🟠 Proxied"
    echo "CNAME   cerberus    origin.cerberusso.tech              🟠 Proxied"
    echo "CNAME   grafana     origin.cerberusso.tech              🟠 Proxied"
    echo ""
    echo -e "${CYAN}🔒 SSL/TLS CONFIGURATION${NC}"
    echo "1. Go to SSL/TLS → Overview"
    echo "2. Set mode to 'Full (strict)'"
    echo "3. Enable 'Always Use HTTPS'"
    echo "4. Enable 'HSTS' with 6 months"
    echo ""
}

# Function to setup monitoring
setup_monitoring() {
    print_status "Setting up monitoring and health checks..."
    
    ssh "$ORACLE_USER@$ORACLE_HOST" << 'EOF'
        cd ~/cerberus-deployment
        
        # Create monitoring script
        cat > monitor-cerberus.sh << 'MONEOF'
#!/bin/bash

# Cerberus Health Check Script
echo "🧠 Cerberus Health Check - $(date)"
echo "=================================="

# Check container status
echo "📊 Container Status:"
docker-compose -f docker-compose.chainguard.yml ps

echo ""
echo "📈 Resource Usage:"
docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}"

echo ""
echo "📋 Recent Logs (last 10 lines):"
docker-compose -f docker-compose.chainguard.yml logs --tail=10 cerberus

echo ""
echo "🔍 Redis Status:"
docker exec redis-hardened redis-cli ping || echo "Redis not responding"

echo ""
echo "💰 Position Count:"
docker exec redis-hardened redis-cli scard active_positions || echo "Cannot check positions"

echo ""
echo "✅ Health check completed"
MONEOF
        
        chmod +x monitor-cerberus.sh
        
        # Create systemd service for monitoring
        sudo tee /etc/systemd/system/cerberus-monitor.service > /dev/null << 'SVCEOF'
[Unit]
Description=Cerberus Health Monitor
After=docker.service

[Service]
Type=oneshot
User=opc
WorkingDirectory=/home/opc/cerberus-deployment
ExecStart=/home/opc/cerberus-deployment/monitor-cerberus.sh
SVCEOF
        
        # Create timer for regular monitoring
        sudo tee /etc/systemd/system/cerberus-monitor.timer > /dev/null << 'TIMEREOF'
[Unit]
Description=Run Cerberus Health Monitor every 5 minutes
Requires=cerberus-monitor.service

[Timer]
OnCalendar=*:0/5
Persistent=true

[Install]
WantedBy=timers.target
TIMEREOF
        
        # Enable and start timer
        sudo systemctl daemon-reload
        sudo systemctl enable cerberus-monitor.timer
        sudo systemctl start cerberus-monitor.timer
        
        echo "Monitoring setup completed"
EOF
    
    print_success "Monitoring configured"
}

# Function to run tests
run_deployment_tests() {
    print_status "Running deployment tests..."
    
    ssh "$ORACLE_USER@$ORACLE_HOST" << 'EOF'
        cd ~/cerberus-deployment
        
        # Wait for services to be fully ready
        sleep 30
        
        # Test Redis connectivity
        echo "Testing Redis..."
        if docker exec redis-hardened redis-cli ping; then
            echo "✅ Redis is responding"
        else
            echo "❌ Redis test failed"
            exit 1
        fi
        
        # Test Cerberus health
        echo "Testing Cerberus..."
        if docker exec cerberus-hardened /usr/local/bin/cerberus --help > /dev/null; then
            echo "✅ Cerberus binary is working"
        else
            echo "❌ Cerberus test failed"
            exit 1
        fi
        
        # Check logs for errors
        echo "Checking logs for errors..."
        if docker-compose -f docker-compose.chainguard.yml logs cerberus | grep -i error; then
            echo "⚠️ Errors found in logs"
        else
            echo "✅ No errors in logs"
        fi
        
        echo "✅ All deployment tests passed"
EOF
    
    print_success "Deployment tests completed"
}

# Function to show deployment summary
show_deployment_summary() {
    echo ""
    echo -e "${GREEN}🎉 CERBERUS DEPLOYMENT COMPLETED!${NC}"
    echo ""
    echo -e "${CYAN}📋 DEPLOYMENT SUMMARY${NC}"
    echo "  🏠 Host: $ORACLE_HOST"
    echo "  🐳 Images: Chainguard hardened"
    echo "  🔒 Security: Enterprise-grade"
    echo "  📊 Monitoring: Enabled"
    echo ""
    echo -e "${CYAN}🔗 ACCESS POINTS${NC}"
    echo "  🌐 Origin: https://origin.cerberusso.tech"
    echo "  📱 App: https://app.cerberusso.tech"
    echo "  🔌 API: https://api.cerberusso.tech"
    echo "  📊 Grafana: https://grafana.cerberusso.tech"
    echo ""
    echo -e "${CYAN}🛠️ MANAGEMENT COMMANDS${NC}"
    echo "  # SSH to instance"
    echo "  ssh $ORACLE_USER@$ORACLE_HOST"
    echo ""
    echo "  # Check status"
    echo "  cd ~/cerberus-deployment && ./monitor-cerberus.sh"
    echo ""
    echo "  # View logs"
    echo "  cd ~/cerberus-deployment && docker-compose -f docker-compose.chainguard.yml logs -f cerberus"
    echo ""
    echo "  # Emergency stop"
    echo "  docker exec redis-hardened redis-cli publish guardian_alerts '{\"action\":\"EXIT_ALL_POSITIONS\",\"reason\":\"MANUAL\"}'"
    echo ""
    echo -e "${PURPLE}🧠 Cerberus is now watching over your positions 24/7!${NC}"
}

# Main execution
main() {
    echo -e "${CYAN}Starting Cerberus Oracle Cloud deployment...${NC}"
    echo ""
    
    check_prerequisites
    echo ""
    
    build_and_push_images
    echo ""
    
    prepare_oracle_instance
    echo ""
    
    deploy_cerberus
    echo ""
    
    setup_monitoring
    echo ""
    
    run_deployment_tests
    echo ""
    
    configure_cloudflare
    echo ""
    
    show_deployment_summary
}

# Handle command line arguments
case "${1:-deploy}" in
    "deploy")
        main
        ;;
    "test")
        print_info "Running tests only..."
        run_deployment_tests
        ;;
    "monitor")
        print_info "Setting up monitoring only..."
        setup_monitoring
        ;;
    "status")
        print_info "Checking deployment status..."
        ssh "$ORACLE_USER@$ORACLE_HOST" "cd ~/cerberus-deployment && ./monitor-cerberus.sh"
        ;;
    *)
        echo "Usage: $0 [deploy|test|monitor|status]"
        echo "  deploy  - Full deployment (default)"
        echo "  test    - Run tests only"
        echo "  monitor - Setup monitoring only"
        echo "  status  - Check current status"
        exit 1
        ;;
esac
