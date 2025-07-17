#!/bin/bash

# Solana HFT Ninja 2025.07 - Oracle Cloud Deployment Script
# Usage: ./deploy.sh [environment] [action]
# Example: ./deploy.sh production deploy

set -e

# Configuration
ENVIRONMENT=${1:-production}
ACTION=${2:-deploy}
PROJECT_NAME="solana-hft-ninja"
DOMAIN=${DOMAIN:-"hft.yourdomain.com"}
DOCKER_COMPOSE_FILE="docker-compose.prod.yml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if Docker is installed
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    # Check if Docker Compose is installed
    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose is not installed. Please install Docker Compose first."
        exit 1
    fi
    
    # Check if .env file exists
    if [ ! -f ".env" ]; then
        log_warning ".env file not found. Creating template..."
        create_env_template
    fi
    
    log_success "Prerequisites check completed"
}

# Create environment template
create_env_template() {
    cat > .env << EOF
# Solana HFT Ninja Environment Configuration
ENVIRONMENT=production
DOMAIN=hft.yourdomain.com

# Helius API
HELIUS_KEY=your_helius_api_key_here

# Solana Configuration
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
JITO_BLOCK_ENGINE_URL=https://mainnet.block-engine.jito.wtf

# Security
GRAFANA_ADMIN_PASSWORD=your_secure_password_here

# SSL (if using custom certificates)
SSL_CERT_PATH=./ssl/cert.pem
SSL_KEY_PATH=./ssl/key.pem

# Oracle Cloud specific
OCI_REGION=us-ashburn-1
OCI_COMPARTMENT_ID=your_compartment_id
EOF
    
    log_warning "Please edit .env file with your actual configuration before deploying"
}

# Generate SSL certificates (self-signed for development)
generate_ssl_certs() {
    log_info "Generating SSL certificates..."
    
    mkdir -p nginx/ssl
    
    if [ ! -f "nginx/ssl/cert.pem" ] || [ ! -f "nginx/ssl/key.pem" ]; then
        openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
            -keyout nginx/ssl/key.pem \
            -out nginx/ssl/cert.pem \
            -subj "/C=US/ST=State/L=City/O=Organization/CN=${DOMAIN}"
        
        log_success "SSL certificates generated"
    else
        log_info "SSL certificates already exist"
    fi
}

# Build and deploy
deploy() {
    log_info "Starting deployment for environment: ${ENVIRONMENT}"
    
    # Load environment variables
    if [ -f ".env" ]; then
        export $(cat .env | grep -v '^#' | xargs)
    fi
    
    # Generate SSL certificates
    generate_ssl_certs
    
    # Build and start services
    log_info "Building and starting services..."
    docker-compose -f ${DOCKER_COMPOSE_FILE} build --no-cache
    docker-compose -f ${DOCKER_COMPOSE_FILE} up -d
    
    # Wait for services to be healthy
    log_info "Waiting for services to be healthy..."
    sleep 30
    
    # Check service health
    check_health
    
    log_success "Deployment completed successfully!"
    log_info "Services available at:"
    log_info "  - Frontend: https://${DOMAIN}"
    log_info "  - API: https://${DOMAIN}/api"
    log_info "  - Grafana: https://${DOMAIN}/grafana"
    log_info "  - Health: https://${DOMAIN}/health"
}

# Check service health
check_health() {
    log_info "Checking service health..."
    
    # Check if containers are running
    if docker-compose -f ${DOCKER_COMPOSE_FILE} ps | grep -q "Up"; then
        log_success "Containers are running"
    else
        log_error "Some containers are not running"
        docker-compose -f ${DOCKER_COMPOSE_FILE} ps
        return 1
    fi
    
    # Check health endpoints
    local max_attempts=10
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if curl -f -s http://localhost:8080/health > /dev/null 2>&1; then
            log_success "HFT service is healthy"
            break
        else
            log_warning "Attempt $attempt/$max_attempts: HFT service not ready yet..."
            sleep 10
            ((attempt++))
        fi
    done
    
    if [ $attempt -gt $max_attempts ]; then
        log_error "HFT service failed to become healthy"
        return 1
    fi
}

# Stop services
stop() {
    log_info "Stopping services..."
    docker-compose -f ${DOCKER_COMPOSE_FILE} down
    log_success "Services stopped"
}

# Restart services
restart() {
    log_info "Restarting services..."
    stop
    deploy
}

# Show logs
logs() {
    local service=${3:-""}
    if [ -n "$service" ]; then
        docker-compose -f ${DOCKER_COMPOSE_FILE} logs -f $service
    else
        docker-compose -f ${DOCKER_COMPOSE_FILE} logs -f
    fi
}

# Show status
status() {
    log_info "Service status:"
    docker-compose -f ${DOCKER_COMPOSE_FILE} ps
    
    log_info "Resource usage:"
    docker stats --no-stream
}

# Cleanup
cleanup() {
    log_info "Cleaning up..."
    docker-compose -f ${DOCKER_COMPOSE_FILE} down -v
    docker system prune -f
    log_success "Cleanup completed"
}

# Main execution
main() {
    case $ACTION in
        "deploy")
            check_prerequisites
            deploy
            ;;
        "stop")
            stop
            ;;
        "restart")
            restart
            ;;
        "logs")
            logs
            ;;
        "status")
            status
            ;;
        "cleanup")
            cleanup
            ;;
        *)
            echo "Usage: $0 [environment] [action]"
            echo "Actions: deploy, stop, restart, logs, status, cleanup"
            echo "Example: $0 production deploy"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
