#!/bin/bash

# Solana HFT Ninja 2025.07 - Monitoring Stack Startup Script
# Usage: ./start-monitoring.sh [action]
# Actions: start, stop, restart, status, logs

set -e

# Configuration
COMPOSE_FILE="docker-compose.monitoring.yml"
PROJECT_NAME="hft-monitoring"

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
# Monitoring Stack Environment Configuration
GRAFANA_ADMIN_PASSWORD=admin123
DOMAIN=localhost

# Alerting Configuration
SMTP_HOST=localhost
SMTP_PORT=587
SMTP_USER=alerts@hft-ninja.com
SMTP_PASSWORD=your_email_password
ALERT_EMAIL=admin@hft-ninja.com

# Slack Integration (optional)
SLACK_WEBHOOK_URL=https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK
EOF
    
    log_warning "Please edit .env file with your actual configuration"
}

# Start monitoring stack
start_monitoring() {
    log_info "Starting monitoring stack..."
    
    # Load environment variables
    if [ -f ".env" ]; then
        export $(cat .env | grep -v '^#' | xargs)
    fi
    
    # Create necessary directories
    mkdir -p prometheus grafana/data alertmanager/data loki/data
    
    # Set proper permissions for Grafana
    sudo chown -R 472:472 grafana/data 2>/dev/null || true
    
    # Start services
    docker-compose -f ${COMPOSE_FILE} -p ${PROJECT_NAME} up -d
    
    # Wait for services to be ready
    log_info "Waiting for services to be ready..."
    sleep 30
    
    # Check service health
    check_services_health
    
    log_success "Monitoring stack started successfully!"
    show_service_urls
}

# Stop monitoring stack
stop_monitoring() {
    log_info "Stopping monitoring stack..."
    docker-compose -f ${COMPOSE_FILE} -p ${PROJECT_NAME} down
    log_success "Monitoring stack stopped"
}

# Restart monitoring stack
restart_monitoring() {
    log_info "Restarting monitoring stack..."
    stop_monitoring
    start_monitoring
}

# Check service health
check_services_health() {
    local services=("prometheus:9090" "grafana:3000" "alertmanager:9093")
    local max_attempts=10
    
    for service in "${services[@]}"; do
        local name=$(echo $service | cut -d: -f1)
        local port=$(echo $service | cut -d: -f2)
        local attempt=1
        
        log_info "Checking $name health..."
        
        while [ $attempt -le $max_attempts ]; do
            if curl -f -s http://localhost:$port/api/health > /dev/null 2>&1 || \
               curl -f -s http://localhost:$port/-/healthy > /dev/null 2>&1 || \
               curl -f -s http://localhost:$port/ > /dev/null 2>&1; then
                log_success "$name is healthy"
                break
            else
                log_warning "Attempt $attempt/$max_attempts: $name not ready yet..."
                sleep 5
                ((attempt++))
            fi
        done
        
        if [ $attempt -gt $max_attempts ]; then
            log_error "$name failed to become healthy"
        fi
    done
}

# Show service URLs
show_service_urls() {
    log_info "Monitoring services are available at:"
    echo "  📊 Grafana:      http://localhost:3000"
    echo "  📈 Prometheus:   http://localhost:9090"
    echo "  🚨 Alertmanager: http://localhost:9093"
    echo "  📋 Node Exporter: http://localhost:9100"
    echo "  🐳 cAdvisor:     http://localhost:8080"
    echo "  📝 Loki:         http://localhost:3100"
    echo ""
    echo "Default Grafana credentials: admin / ${GRAFANA_ADMIN_PASSWORD:-admin123}"
}

# Show service status
show_status() {
    log_info "Monitoring stack status:"
    docker-compose -f ${COMPOSE_FILE} -p ${PROJECT_NAME} ps
    
    log_info "Resource usage:"
    docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}\t{{.BlockIO}}"
}

# Show logs
show_logs() {
    local service=${2:-""}
    if [ -n "$service" ]; then
        docker-compose -f ${COMPOSE_FILE} -p ${PROJECT_NAME} logs -f $service
    else
        docker-compose -f ${COMPOSE_FILE} -p ${PROJECT_NAME} logs -f
    fi
}

# Cleanup
cleanup() {
    log_info "Cleaning up monitoring stack..."
    docker-compose -f ${COMPOSE_FILE} -p ${PROJECT_NAME} down -v
    docker system prune -f
    log_success "Cleanup completed"
}

# Main execution
main() {
    local action=${1:-"start"}
    
    case $action in
        "start")
            check_prerequisites
            start_monitoring
            ;;
        "stop")
            stop_monitoring
            ;;
        "restart")
            restart_monitoring
            ;;
        "status")
            show_status
            ;;
        "logs")
            show_logs "$@"
            ;;
        "cleanup")
            cleanup
            ;;
        "urls")
            show_service_urls
            ;;
        *)
            echo "Usage: $0 [action]"
            echo "Actions: start, stop, restart, status, logs [service], cleanup, urls"
            echo "Example: $0 start"
            echo "Example: $0 logs grafana"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
