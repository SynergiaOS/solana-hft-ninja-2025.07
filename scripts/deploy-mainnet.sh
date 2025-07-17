#!/bin/bash

# 🚀 MAINNET DEPLOYMENT SCRIPT
# Gradual rollout: Paper trading → Small amounts → Full deployment

set -euo pipefail

DEPLOYMENT_PHASE="${1:-check}"
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
LOG_FILE="logs/mainnet-deployment-$(date +%Y%m%d_%H%M%S).log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to log with timestamp and color
log() {
    local color="$1"
    local message="$2"
    echo -e "${color}[$(date '+%H:%M:%S')] $message${NC}" | tee -a "$LOG_FILE"
}

# Function to check prerequisites
check_prerequisites() {
    log "$BLUE" "🔍 Checking prerequisites..."
    
    local errors=0
    
    # Check if wallet exists and has funds
    if [ ! -f "${WALLET_PRIVATE_KEY_PATH:-}" ]; then
        log "$RED" "❌ Wallet file not found: ${WALLET_PRIVATE_KEY_PATH:-}"
        ((errors++))
    else
        log "$GREEN" "✅ Wallet file found"
        
        # Check wallet balance
        local balance=$(solana balance "${WALLET_ADDRESS}" --url https://api.mainnet-beta.solana.com 2>/dev/null || echo "0")
        local balance_num=$(echo "$balance" | grep -o '[0-9.]*' | head -1)
        
        if (( $(echo "$balance_num < 1.0" | bc -l) )); then
            log "$RED" "❌ Insufficient wallet balance: $balance (minimum 1 SOL required)"
            ((errors++))
        else
            log "$GREEN" "✅ Wallet balance: $balance"
        fi
    fi
    
    # Check environment configuration
    if [ ! -f ".env.mainnet" ]; then
        log "$RED" "❌ .env.mainnet not found"
        ((errors++))
    else
        source .env.mainnet
        
        if [ -z "${HELIUS_API_KEY:-}" ] || [ "$HELIUS_API_KEY" = "your_helius_api_key_here" ]; then
            log "$RED" "❌ Helius API key not configured"
            ((errors++))
        else
            log "$GREEN" "✅ Helius API key configured"
        fi
        
        if [ "${TRADING_ENABLED:-}" = "true" ]; then
            log "$YELLOW" "⚠️ Trading is enabled - will be disabled for safe deployment"
        fi
    fi
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        log "$RED" "❌ Docker not installed"
        ((errors++))
    else
        log "$GREEN" "✅ Docker available"
    fi
    
    # Check if ports are available
    local ports=(5432 6379 6380 8000 8080 8081 3001 9090 3000)
    for port in "${ports[@]}"; do
        if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
            log "$YELLOW" "⚠️ Port $port is in use"
        fi
    done
    
    if [ $errors -gt 0 ]; then
        log "$RED" "❌ $errors prerequisite check(s) failed"
        return 1
    else
        log "$GREEN" "✅ All prerequisites passed"
        return 0
    fi
}

# Function to run security audit
run_security_audit() {
    log "$BLUE" "🛡️ Running security audit..."
    
    if [ -f "scripts/security-audit.sh" ]; then
        if ./scripts/security-audit.sh; then
            log "$GREEN" "✅ Security audit passed"
            return 0
        else
            log "$RED" "❌ Security audit failed"
            return 1
        fi
    else
        log "$YELLOW" "⚠️ Security audit script not found"
        return 1
    fi
}

# Phase 1: Paper Trading
deploy_paper_trading() {
    log "$BLUE" "📄 Phase 1: Deploying paper trading mode..."
    
    # Ensure trading is disabled
    sed -i 's/TRADING_ENABLED=.*/TRADING_ENABLED=false/' .env.mainnet
    
    # Start services
    log "$BLUE" "🐳 Starting Docker services..."
    docker-compose -f docker-compose.mainnet.yml up -d
    
    # Wait for services to be ready
    log "$BLUE" "⏳ Waiting for services to be ready..."
    sleep 30
    
    # Health checks
    local services=("postgres" "redis" "dragonflydb" "cerebro-api" "hft-engine")
    for service in "${services[@]}"; do
        if docker ps | grep -q "$service"; then
            log "$GREEN" "✅ $service is running"
        else
            log "$RED" "❌ $service failed to start"
            return 1
        fi
    done
    
    # Test API connectivity
    if curl -s -f "http://localhost:8000/health" > /dev/null; then
        log "$GREEN" "✅ API health check passed"
    else
        log "$RED" "❌ API health check failed"
        return 1
    fi
    
    # Test HFT engine
    if curl -s -f "http://localhost:8080/health" > /dev/null; then
        log "$GREEN" "✅ HFT engine health check passed"
    else
        log "$RED" "❌ HFT engine health check failed"
        return 1
    fi
    
    log "$GREEN" "✅ Paper trading mode deployed successfully"
    log "$BLUE" "📊 Dashboard available at: http://localhost:3001"
    log "$BLUE" "📈 Monitoring available at: http://localhost:3000"
    
    return 0
}

# Phase 2: Small Amount Trading
deploy_small_amount() {
    log "$BLUE" "💰 Phase 2: Enabling small amount trading..."
    
    # Update limits for small trading
    docker exec cerebro-api-mainnet curl -s -X POST "http://localhost:8000/api/risk/limits" \
        -H "Content-Type: application/json" \
        -d '{
            "max_position_size_sol": 0.1,
            "max_daily_loss_sol": 0.05,
            "stop_loss_percentage": 3.0,
            "max_slippage_percentage": 1.0,
            "min_liquidity_usd": 50000,
            "max_trades_per_minute": 2
        }' || return 1
    
    # Enable trading with small limits
    docker exec cerebro-hft-mainnet curl -s -X POST "http://localhost:8080/api/trading/enable" \
        -H "Content-Type: application/json" \
        -d '{"reason": "Phase 2: Small amount testing"}' || return 1
    
    log "$GREEN" "✅ Small amount trading enabled"
    log "$YELLOW" "⚠️ Limits: 0.1 SOL max position, 0.05 SOL max daily loss"
    
    return 0
}

# Phase 3: Full Deployment
deploy_full() {
    log "$BLUE" "🚀 Phase 3: Full deployment..."
    
    # Update to production limits
    docker exec cerebro-api-mainnet curl -s -X POST "http://localhost:8000/api/risk/limits" \
        -H "Content-Type: application/json" \
        -d '{
            "max_position_size_sol": 1.0,
            "max_daily_loss_sol": 0.5,
            "stop_loss_percentage": 5.0,
            "max_slippage_percentage": 2.0,
            "min_liquidity_usd": 10000,
            "max_trades_per_minute": 10
        }' || return 1
    
    log "$GREEN" "✅ Full production limits applied"
    log "$BLUE" "🎯 Target: 5% daily ROI (0.4 SOL from 8 SOL)"
    
    return 0
}

# Function to monitor deployment
monitor_deployment() {
    local duration="${1:-300}" # 5 minutes default
    log "$BLUE" "📊 Monitoring deployment for $duration seconds..."
    
    local start_time=$(date +%s)
    local end_time=$((start_time + duration))
    
    while [ $(date +%s) -lt $end_time ]; do
        # Check system health
        local api_status=$(curl -s "http://localhost:8000/health" | jq -r '.status' 2>/dev/null || echo "error")
        local hft_status=$(curl -s "http://localhost:8080/health" | jq -r '.status' 2>/dev/null || echo "error")
        
        # Check trading metrics
        local metrics=$(curl -s "http://localhost:8080/api/metrics" 2>/dev/null || echo '{}')
        local trades_count=$(echo "$metrics" | jq -r '.trades_today // 0' 2>/dev/null || echo "0")
        local pnl=$(echo "$metrics" | jq -r '.daily_pnl // 0' 2>/dev/null || echo "0")
        
        log "$BLUE" "Status: API=$api_status, HFT=$hft_status, Trades=$trades_count, P&L=$pnl SOL"
        
        sleep 30
    done
    
    log "$GREEN" "✅ Monitoring completed"
}

# Main deployment logic
case "$DEPLOYMENT_PHASE" in
    "check")
        echo "🔍 MAINNET DEPLOYMENT - PREREQUISITE CHECK"
        echo "=========================================="
        if check_prerequisites && run_security_audit; then
            echo ""
            echo "✅ System ready for mainnet deployment!"
            echo ""
            echo "Next steps:"
            echo "1. Paper trading:    ./scripts/deploy-mainnet.sh paper"
            echo "2. Small amounts:    ./scripts/deploy-mainnet.sh small"
            echo "3. Full deployment:  ./scripts/deploy-mainnet.sh full"
            echo ""
            exit 0
        else
            echo ""
            echo "❌ System not ready for deployment"
            echo "Fix the issues above and run again"
            exit 1
        fi
        ;;
        
    "paper")
        echo "📄 MAINNET DEPLOYMENT - PAPER TRADING"
        echo "====================================="
        if check_prerequisites && deploy_paper_trading; then
            monitor_deployment 300
            echo ""
            echo "✅ Paper trading deployed successfully!"
            echo "📊 Monitor at: http://localhost:3001"
            echo ""
            echo "Next: ./scripts/deploy-mainnet.sh small"
        else
            echo "❌ Paper trading deployment failed"
            exit 1
        fi
        ;;
        
    "small")
        echo "💰 MAINNET DEPLOYMENT - SMALL AMOUNTS"
        echo "====================================="
        if deploy_small_amount; then
            monitor_deployment 600
            echo ""
            echo "✅ Small amount trading enabled!"
            echo "⚠️ Monitor carefully for 1 hour before full deployment"
            echo ""
            echo "Next: ./scripts/deploy-mainnet.sh full"
        else
            echo "❌ Small amount deployment failed"
            exit 1
        fi
        ;;
        
    "full")
        echo "🚀 MAINNET DEPLOYMENT - FULL PRODUCTION"
        echo "======================================="
        read -p "Are you sure you want to enable full production trading? (yes/no): " confirm
        if [ "$confirm" = "yes" ]; then
            if deploy_full; then
                monitor_deployment 900
                echo ""
                echo "🎉 FULL PRODUCTION DEPLOYMENT COMPLETE!"
                echo "======================================="
                echo "🎯 Target: 5% daily ROI"
                echo "📊 Dashboard: http://localhost:3001"
                echo "📈 Monitoring: http://localhost:3000"
                echo "🚨 Emergency stop: ./scripts/emergency-shutdown.sh"
                echo ""
                echo "🧠 Cerebro HFT Ninja is now live on mainnet!"
            else
                echo "❌ Full deployment failed"
                exit 1
            fi
        else
            echo "Deployment cancelled"
            exit 0
        fi
        ;;
        
    *)
        echo "Usage: $0 {check|paper|small|full}"
        echo ""
        echo "Phases:"
        echo "  check  - Check prerequisites and security"
        echo "  paper  - Deploy in paper trading mode"
        echo "  small  - Enable small amount trading"
        echo "  full   - Full production deployment"
        exit 1
        ;;
esac
