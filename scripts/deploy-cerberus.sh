#!/bin/bash

# 🧠 Cerberus Trade Execution Brain - Production Deployment Script
# Deploy autonomous position management system for Solana HFT

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                🧠 CERBERUS DEPLOYMENT 🧠                     ║"
echo "║                Solana HFT Ninja 2025.07                     ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
CERBERUS_BIN="./target/release/cerberus"
CONFIG_FILE="config/cerberus.toml"
LOG_FILE="/tmp/cerberus-deploy.log"

# Functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

log_step() {
    echo -e "${PURPLE}🔄 $1${NC}"
}

check_prerequisites() {
    log_step "Checking prerequisites..."
    
    # Check if running on correct architecture
    ARCH=$(uname -m)
    if [[ "$ARCH" != "x86_64" && "$ARCH" != "aarch64" ]]; then
        log_warning "Unsupported architecture: $ARCH"
    else
        log_success "Architecture: $ARCH"
    fi
    
    # Check Rust installation
    if ! command -v cargo &> /dev/null; then
        log_error "Rust/Cargo not found. Install from https://rustup.rs/"
        exit 1
    fi
    log_success "Rust/Cargo found: $(cargo --version)"
    
    # Check Redis availability
    if ! command -v redis-cli &> /dev/null; then
        log_warning "Redis CLI not found. Install Redis or use Docker:"
        echo "  docker run -d --name redis -p 6379:6379 redis:alpine"
    else
        if redis-cli ping > /dev/null 2>&1; then
            log_success "Redis is running"
        else
            log_warning "Redis is not responding"
        fi
    fi
    
    # Check environment variables
    if [[ -z "$QUICKNODE_ENDPOINT" ]]; then
        log_warning "QUICKNODE_ENDPOINT not set"
    else
        log_success "QuickNode endpoint configured"
    fi
    
    if [[ -z "$HELIUS_ENDPOINT" ]]; then
        log_warning "HELIUS_ENDPOINT not set"
    else
        log_success "Helius endpoint configured"
    fi
    
    if [[ -z "$SOLANA_PRIVATE_KEY" ]]; then
        log_warning "SOLANA_PRIVATE_KEY not set"
    else
        log_success "Solana private key configured"
    fi
}

build_cerberus() {
    log_step "Building Cerberus..."
    
    if cargo build --release --bin cerberus > "$LOG_FILE" 2>&1; then
        log_success "Cerberus built successfully"
    else
        log_error "Build failed. Check log: $LOG_FILE"
        tail -20 "$LOG_FILE"
        exit 1
    fi
    
    # Check binary size
    if [[ -f "$CERBERUS_BIN" ]]; then
        BINARY_SIZE=$(du -h "$CERBERUS_BIN" | cut -f1)
        log_success "Binary size: $BINARY_SIZE"
    fi
}

run_tests() {
    log_step "Running test suite..."
    
    if [[ -f "scripts/test-cerberus.sh" ]]; then
        if bash scripts/test-cerberus.sh > "$LOG_FILE" 2>&1; then
            log_success "All tests passed"
        else
            log_warning "Some tests failed. Check log: $LOG_FILE"
        fi
    else
        log_warning "Test script not found, skipping tests"
    fi
}

setup_configuration() {
    log_step "Setting up configuration..."
    
    # Create config directory if it doesn't exist
    mkdir -p config
    
    # Check if config file exists
    if [[ ! -f "$CONFIG_FILE" ]]; then
        log_warning "Config file not found, using defaults"
    else
        log_success "Configuration file found: $CONFIG_FILE"
    fi
    
    # Validate configuration
    if [[ -f "$CONFIG_FILE" ]]; then
        if grep -q "loop_interval_ms" "$CONFIG_FILE"; then
            INTERVAL=$(grep "loop_interval_ms" "$CONFIG_FILE" | cut -d'=' -f2 | tr -d ' ')
            log_info "Decision interval: ${INTERVAL}ms"
        fi
    fi
}

start_dependencies() {
    log_step "Starting dependencies..."
    
    # Start Redis if not running
    if ! redis-cli ping > /dev/null 2>&1; then
        log_info "Starting Redis with Docker..."
        if command -v docker &> /dev/null; then
            docker run -d --name cerberus-redis -p 6379:6379 redis:alpine > /dev/null 2>&1 || true
            sleep 2
            if redis-cli ping > /dev/null 2>&1; then
                log_success "Redis started successfully"
            else
                log_error "Failed to start Redis"
                exit 1
            fi
        else
            log_error "Docker not found and Redis not running"
            exit 1
        fi
    fi
}

deploy_production() {
    log_step "Deploying to production..."
    
    # Create deployment directory
    DEPLOY_DIR="/opt/cerberus"
    if [[ ! -d "$DEPLOY_DIR" ]]; then
        log_info "Creating deployment directory: $DEPLOY_DIR"
        sudo mkdir -p "$DEPLOY_DIR" 2>/dev/null || mkdir -p "./deploy"
        DEPLOY_DIR="./deploy"
    fi
    
    # Copy binary
    cp "$CERBERUS_BIN" "$DEPLOY_DIR/" 2>/dev/null || {
        log_warning "Cannot copy to $DEPLOY_DIR, using local directory"
        DEPLOY_DIR="."
    }
    
    # Copy configuration
    cp "$CONFIG_FILE" "$DEPLOY_DIR/" 2>/dev/null || true
    
    log_success "Deployed to: $DEPLOY_DIR"
}

create_systemd_service() {
    log_step "Creating systemd service..."
    
    SERVICE_FILE="/etc/systemd/system/cerberus.service"
    
    if [[ -w "/etc/systemd/system" ]] || sudo -n true 2>/dev/null; then
        cat > /tmp/cerberus.service << EOF
[Unit]
Description=Cerberus Trade Execution Brain
After=network.target redis.service
Wants=redis.service

[Service]
Type=simple
User=cerberus
WorkingDirectory=/opt/cerberus
ExecStart=/opt/cerberus/cerberus --quicknode \${QUICKNODE_ENDPOINT} --helius \${HELIUS_ENDPOINT}
Restart=always
RestartSec=10
Environment=QUICKNODE_ENDPOINT=${QUICKNODE_ENDPOINT}
Environment=HELIUS_ENDPOINT=${HELIUS_ENDPOINT}
Environment=SOLANA_PRIVATE_KEY=${SOLANA_PRIVATE_KEY}
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF
        
        sudo cp /tmp/cerberus.service "$SERVICE_FILE" 2>/dev/null && {
            log_success "Systemd service created"
            log_info "Enable with: sudo systemctl enable cerberus"
            log_info "Start with: sudo systemctl start cerberus"
        } || {
            log_warning "Cannot create systemd service (no sudo access)"
        }
    else
        log_warning "Cannot create systemd service (no write access)"
    fi
}

show_deployment_info() {
    echo
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║                🎉 DEPLOYMENT COMPLETE 🎉                    ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo
    
    log_success "Cerberus Trade Execution Brain is ready!"
    echo
    
    echo "📋 DEPLOYMENT SUMMARY:"
    echo "  Binary: $CERBERUS_BIN"
    echo "  Config: $CONFIG_FILE"
    echo "  Log: $LOG_FILE"
    echo
    
    echo "🚀 QUICK START:"
    echo "  # Dry run (safe testing)"
    echo "  $CERBERUS_BIN --dry-run"
    echo
    echo "  # Production (with premium endpoints)"
    echo "  $CERBERUS_BIN \\"
    echo "    --quicknode \$QUICKNODE_ENDPOINT \\"
    echo "    --helius \$HELIUS_ENDPOINT"
    echo
    
    echo "🔧 CONFIGURATION:"
    echo "  export QUICKNODE_ENDPOINT='https://your-endpoint.quiknode.pro/your-key/'"
    echo "  export HELIUS_ENDPOINT='https://mainnet.helius-rpc.com/?api-key=your-key'"
    echo "  export SOLANA_PRIVATE_KEY='[your,private,key,array]'"
    echo
    
    echo "📊 MONITORING:"
    echo "  # Check positions"
    echo "  redis-cli scard active_positions"
    echo
    echo "  # Send test command"
    echo "  redis-cli publish cerebro_commands '{\"action\":\"SELL\",\"mint\":\"test\"}'"
    echo
    
    echo "🆘 EMERGENCY STOP:"
    echo "  redis-cli publish guardian_alerts '{\"action\":\"EXIT_ALL_POSITIONS\",\"reason\":\"MANUAL\"}'"
    echo
}

# Main deployment flow
main() {
    echo "🚀 Starting Cerberus deployment..."
    echo
    
    check_prerequisites
    echo
    
    build_cerberus
    echo
    
    setup_configuration
    echo
    
    start_dependencies
    echo
    
    run_tests
    echo
    
    deploy_production
    echo
    
    create_systemd_service
    echo
    
    show_deployment_info
}

# Handle command line arguments
case "${1:-deploy}" in
    "deploy")
        main
        ;;
    "test")
        log_info "Running tests only..."
        run_tests
        ;;
    "build")
        log_info "Building only..."
        build_cerberus
        ;;
    "check")
        log_info "Checking prerequisites only..."
        check_prerequisites
        ;;
    *)
        echo "Usage: $0 [deploy|test|build|check]"
        echo "  deploy - Full deployment (default)"
        echo "  test   - Run tests only"
        echo "  build  - Build binary only"
        echo "  check  - Check prerequisites only"
        exit 1
        ;;
esac
