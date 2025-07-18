#!/bin/bash

# 🚨 Oracle Free Tier Deployment Script
# Deploy Solana HFT Ninja + DeepSeek-Math AI on Oracle ARM Ampere
# 4 OCPU + 24 GB RAM - $0/month forever!

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
ORACLE_REGION="us-ashburn-1"
INSTANCE_SHAPE="VM.Standard.A1.Flex"
OCPU_COUNT=4
MEMORY_GB=24
STORAGE_GB=200

echo -e "${BLUE}🚨 Oracle Free Tier Deployment - Solana HFT Ninja 2025.07${NC}"
echo -e "${GREEN}Target: 4 OCPU + 24 GB RAM ARM Ampere - $0/month forever!${NC}"
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

# Check if running on Oracle instance
check_oracle_instance() {
    print_status "Checking if running on Oracle Cloud instance..."
    
    if curl -s --connect-timeout 5 http://169.254.169.254/opc/v1/instance/ > /dev/null 2>&1; then
        print_status "✅ Running on Oracle Cloud instance"
        INSTANCE_METADATA=$(curl -s http://169.254.169.254/opc/v1/instance/)
        echo "Instance metadata: $INSTANCE_METADATA"
    else
        print_warning "Not running on Oracle Cloud instance - continuing anyway"
    fi
}

# Check system requirements
check_system_requirements() {
    print_status "Checking system requirements..."
    
    # Check architecture
    ARCH=$(uname -m)
    if [ "$ARCH" != "aarch64" ]; then
        print_error "This script is optimized for ARM64 (aarch64) architecture"
        print_error "Current architecture: $ARCH"
        exit 1
    fi
    print_status "✅ ARM64 architecture detected"
    
    # Check memory
    TOTAL_MEM=$(free -g | awk '/^Mem:/{print $2}')
    if [ "$TOTAL_MEM" -lt 20 ]; then
        print_warning "Available memory: ${TOTAL_MEM}GB (recommended: 24GB)"
    else
        print_status "✅ Memory: ${TOTAL_MEM}GB"
    fi
    
    # Check CPU cores
    CPU_CORES=$(nproc)
    if [ "$CPU_CORES" -lt 4 ]; then
        print_warning "Available CPU cores: $CPU_CORES (recommended: 4)"
    else
        print_status "✅ CPU cores: $CPU_CORES"
    fi
}

# Update system and install dependencies
install_dependencies() {
    print_status "Installing system dependencies..."
    
    # Update system
    sudo apt update && sudo apt upgrade -y
    
    # Install Docker and dependencies
    sudo apt install -y \
        docker.io \
        docker-compose \
        git \
        curl \
        wget \
        vim \
        htop \
        iotop \
        nethogs \
        unzip \
        build-essential \
        software-properties-common \
        apt-transport-https \
        ca-certificates \
        gnupg \
        lsb-release
    
    # Add user to docker group
    sudo usermod -aG docker $USER
    
    # Enable Docker service
    sudo systemctl enable docker
    sudo systemctl start docker
    
    print_status "✅ Dependencies installed"
}

# Configure Docker for ARM64
configure_docker() {
    print_status "Configuring Docker for ARM64..."
    
    # Create Docker daemon configuration
    sudo tee /etc/docker/daemon.json > /dev/null <<EOF
{
    "log-driver": "json-file",
    "log-opts": {
        "max-size": "10m",
        "max-file": "3"
    },
    "default-runtime": "runc",
    "experimental": true,
    "features": {
        "buildkit": true
    }
}
EOF
    
    # Restart Docker
    sudo systemctl restart docker
    
    # Verify Docker installation
    docker --version
    docker-compose --version
    
    print_status "✅ Docker configured for ARM64"
}

# Clone repository
clone_repository() {
    print_status "Cloning Solana HFT Ninja repository..."
    
    if [ -d "solana-hft-ninja-2025.07" ]; then
        print_warning "Repository already exists, updating..."
        cd solana-hft-ninja-2025.07
        git pull origin main
    else
        git clone https://github.com/SynergiaOS/solana-hft-ninja-2025.07.git
        cd solana-hft-ninja-2025.07
    fi
    
    print_status "✅ Repository ready"
}

# Setup configuration
setup_configuration() {
    print_status "Setting up Oracle-specific configuration..."
    
    # Copy Oracle configuration
    if [ -f "config/oracle-cloud.toml" ]; then
        cp config/oracle-cloud.toml config/config.toml
        print_status "✅ Oracle configuration applied"
    else
        print_warning "Oracle configuration not found, using default"
        cp config/config.toml.example config/config.toml 2>/dev/null || true
    fi
    
    # Create environment file
    cat > .env.oracle << EOF
# Oracle Free Tier Configuration
DEPLOYMENT_TYPE=oracle-free-tier
ARCHITECTURE=arm64
TOTAL_MEMORY_GB=24
TOTAL_OCPU=4

# Resource allocation
HFT_NINJA_MEMORY=2G
DEEPSEEK_MEMORY=6G
DASHBOARD_MEMORY=512M
REDIS_MEMORY=128M
KESTRA_MEMORY=800M

# Performance optimizations
ARM_OPTIMIZATION=true
NEON_OPTIMIZATION=true
USE_QUANTIZATION=true
CACHE_SIZE_MB=512

# Solana configuration
SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_WS_URL=wss://api.devnet.solana.com

# AI configuration
MODEL_NAME=deepseek-ai/deepseek-math-7b-instruct
MAX_DAILY_AI_COST=1.0
PREFER_CACHE=true
EOF
    
    print_status "✅ Configuration setup complete"
}

# Build ARM64 images
build_images() {
    print_status "Building ARM64 optimized images..."
    
    # Set build environment
    export DOCKER_BUILDKIT=1
    export COMPOSE_DOCKER_CLI_BUILD=1
    
    # Build images with ARM64 optimization
    docker-compose -f docker-compose.oracle-arm.yml build --parallel
    
    print_status "✅ ARM64 images built"
}

# Deploy services
deploy_services() {
    print_status "Deploying services on Oracle Free Tier..."
    
    # Start services
    docker-compose -f docker-compose.oracle-arm.yml up -d
    
    # Wait for services to start
    print_status "Waiting for services to start..."
    sleep 30
    
    print_status "✅ Services deployed"
}

# Verify deployment
verify_deployment() {
    print_status "Verifying deployment..."
    
    # Check service health
    services=(
        "http://localhost:8080/health:HFT Ninja"
        "http://localhost:8003/health:DeepSeek-Math AI"
        "http://localhost:3000:React Dashboard"
        "http://localhost:6379:Redis"
        "http://localhost:8085/health:Kestra"
    )
    
    for service in "${services[@]}"; do
        IFS=':' read -r url name <<< "$service"
        if curl -f -s "$url" > /dev/null 2>&1; then
            print_status "✅ $name: OK"
        else
            print_warning "⚠️  $name: Not responding"
        fi
    done
    
    # Show resource usage
    print_status "Resource usage:"
    docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}"
}

# Setup monitoring
setup_monitoring() {
    print_status "Setting up monitoring..."
    
    # Create monitoring script
    cat > monitor-oracle.sh << 'EOF'
#!/bin/bash
echo "=== Oracle Free Tier Resource Monitor ==="
echo "Date: $(date)"
echo ""

echo "=== System Resources ==="
echo "Memory Usage:"
free -h
echo ""

echo "CPU Usage:"
top -bn1 | grep "Cpu(s)" | awk '{print $2 $3 $4 $5 $6 $7 $8}'
echo ""

echo "Disk Usage:"
df -h /
echo ""

echo "=== Docker Containers ==="
docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}"
echo ""

echo "=== Service Health ==="
services=("8080:HFT-Ninja" "8003:DeepSeek-Math" "3000:Dashboard" "6379:Redis")
for service in "${services[@]}"; do
    IFS=':' read -r port name <<< "$service"
    if netstat -tlnp | grep ":$port " > /dev/null; then
        echo "✅ $name (port $port): Running"
    else
        echo "❌ $name (port $port): Not running"
    fi
done
EOF
    
    chmod +x monitor-oracle.sh
    
    print_status "✅ Monitoring setup complete"
    print_status "Run './monitor-oracle.sh' to check system status"
}

# Main deployment function
main() {
    echo -e "${BLUE}Starting Oracle Free Tier deployment...${NC}"
    echo ""
    
    check_oracle_instance
    check_system_requirements
    install_dependencies
    configure_docker
    clone_repository
    setup_configuration
    build_images
    deploy_services
    verify_deployment
    setup_monitoring
    
    echo ""
    echo -e "${GREEN}🎉 Deployment Complete!${NC}"
    echo ""
    echo -e "${BLUE}Access your services:${NC}"
    echo "• HFT Ninja API: http://$(curl -s ifconfig.me):8080"
    echo "• DeepSeek-Math AI: http://$(curl -s ifconfig.me):8003"
    echo "• React Dashboard: http://$(curl -s ifconfig.me):3000"
    echo "• Kestra Workflows: http://$(curl -s ifconfig.me):8085"
    echo ""
    echo -e "${YELLOW}Resource Usage:${NC}"
    echo "• Total RAM: ~9.5 GB / 24 GB (40% utilization)"
    echo "• Total CPU: ~4.0 OCPU / 4 OCPU (100% allocation)"
    echo "• Monthly Cost: $0 (Oracle Free Tier)"
    echo ""
    echo -e "${GREEN}🚨 Your AI trading system is now running for FREE!${NC}"
}

# Run main function
main "$@"
