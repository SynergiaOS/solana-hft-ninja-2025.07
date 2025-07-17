#!/bin/bash

# 🥷 Solana HFT Ninja 2025.07 - Strategy Execution Script
# Complete MEV strategy testing and deployment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Logging functions
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

log_strategy() {
    echo -e "${PURPLE}🎯 $1${NC}"
}

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG_FILE="$PROJECT_ROOT/config/strategies.toml"
WALLET_FILE="$PROJECT_ROOT/config/wallet.json"
INITIAL_BALANCE=${INITIAL_BALANCE:-8.0}
DRY_RUN=${DRY_RUN:-true}
DURATION=${DURATION:-3600} # 1 hour default

# Strategy flags
ENABLE_SANDWICH=${ENABLE_SANDWICH:-true}
ENABLE_ARBITRAGE=${ENABLE_ARBITRAGE:-true}
ENABLE_SNIPING=${ENABLE_SNIPING:-true}
ENABLE_JUPITER_ARB=${ENABLE_JUPITER_ARB:-true}
ENABLE_LIQUIDATION=${ENABLE_LIQUIDATION:-true}

echo "🥷 SOLANA HFT NINJA 2025.07 - MEV STRATEGY EXECUTION"
echo "=================================================="

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust."
        exit 1
    fi
    
    # Check if config file exists
    if [ ! -f "$CONFIG_FILE" ]; then
        log_error "Strategy config file not found: $CONFIG_FILE"
        exit 1
    fi
    
    # Check if wallet exists
    if [ ! -f "$WALLET_FILE" ]; then
        log_warning "Wallet file not found. Will use default keypair."
    fi
    
    log_success "Prerequisites check passed"
}

# Run unit tests
run_unit_tests() {
    log_info "Running MEV strategy unit tests..."
    
    echo "🧪 Testing individual strategies..."
    
    # Test sandwich strategy
    log_strategy "Testing Sandwich Strategy"
    cargo test --test mev_strategy_test test_sandwich_strategy_detection -- --nocapture
    
    # Test arbitrage strategy
    log_strategy "Testing Arbitrage Strategy"
    cargo test --test mev_strategy_test test_arbitrage_strategy_detection -- --nocapture
    
    # Test token launch sniping
    log_strategy "Testing Token Launch Sniping"
    cargo test --test mev_strategy_test test_token_launch_sniping -- --nocapture
    
    # Test risk management
    log_strategy "Testing Risk Management"
    cargo test --test mev_strategy_test test_mev_risk_management -- --nocapture
    
    # Test profitability filtering
    log_strategy "Testing Profitability Filtering"
    cargo test --test mev_strategy_test test_mev_profitability_filtering -- --nocapture
    
    log_success "All unit tests passed!"
}

# Run integration tests
run_integration_tests() {
    log_info "Running integration tests..."
    
    # Bridge communication test
    log_strategy "Testing Bridge Communication"
    cargo test --test integration_bridge_test -- --nocapture
    
    # Helius WebSocket test
    log_strategy "Testing Helius Integration"
    cargo test --test integration_helius_test -- --nocapture
    
    # Jito bundle execution test
    log_strategy "Testing Jito Bundle Execution"
    cargo test --test integration_jito_test -- --nocapture
    
    log_success "All integration tests passed!"
}

# Run strategy in isolation
run_strategy_isolated() {
    local strategy=$1
    local duration=${2:-300} # 5 minutes default
    
    log_strategy "Running $strategy strategy in isolation for ${duration}s..."
    
    local cmd="cargo run --release -- \
        --strategies $strategy \
        --initial-balance $INITIAL_BALANCE \
        --duration $duration \
        --config $CONFIG_FILE"
    
    if [ "$DRY_RUN" = "true" ]; then
        cmd="$cmd --dry-run"
    fi
    
    echo "Command: $cmd"
    eval $cmd
    
    # Check results
    local exit_code=$?
    if [ $exit_code -eq 0 ]; then
        log_success "$strategy strategy completed successfully"
    else
        log_error "$strategy strategy failed with exit code $exit_code"
        return $exit_code
    fi
}

# Monitor strategy performance
monitor_strategy() {
    local strategy=$1
    
    log_info "Monitoring $strategy strategy performance..."
    
    # Get strategy stats
    local stats=$(curl -s http://localhost:8080/strategy/$strategy/stats 2>/dev/null || echo "{}")
    echo "📊 Strategy Stats: $stats"
    
    # Get system metrics
    local metrics=$(curl -s http://localhost:8080/metrics 2>/dev/null | grep "hft_" | head -10 || echo "Metrics not available")
    echo "📈 System Metrics:"
    echo "$metrics"
}

# Run all strategies
run_all_strategies() {
    log_info "Running all enabled strategies..."
    
    local strategies=()
    
    if [ "$ENABLE_SANDWICH" = "true" ]; then
        strategies+=("sandwich")
    fi
    
    if [ "$ENABLE_ARBITRAGE" = "true" ]; then
        strategies+=("arbitrage")
    fi
    
    if [ "$ENABLE_SNIPING" = "true" ]; then
        strategies+=("sniping")
    fi
    
    if [ "$ENABLE_JUPITER_ARB" = "true" ]; then
        strategies+=("jupiter_arbitrage")
    fi
    
    if [ "$ENABLE_LIQUIDATION" = "true" ]; then
        strategies+=("liquidation")
    fi
    
    local strategy_list=$(IFS=,; echo "${strategies[*]}")
    
    log_strategy "Launching strategies: $strategy_list"
    
    local cmd="cargo run --release -- \
        --strategies $strategy_list \
        --initial-balance $INITIAL_BALANCE \
        --risk-mode moderate \
        --log-level info \
        --prometheus-port 9090 \
        --enable-jito \
        --enable-helius \
        --config $CONFIG_FILE"
    
    if [ "$DRY_RUN" = "true" ]; then
        cmd="$cmd --dry-run"
    fi
    
    echo "🚀 Final command: $cmd"
    eval $cmd
}

# Performance analysis
analyze_performance() {
    log_info "Analyzing strategy performance..."
    
    # Generate performance report
    cargo run --release -- --generate-report performance
    
    # Collect metrics
    curl -s http://localhost:8080/strategy/metrics > metrics.json
    
    # Analyze with Python if available
    if command -v python3 &> /dev/null; then
        python3 -c "
import json
import sys

try:
    with open('metrics.json') as f:
        data = json.load(f)
    
    total_profit = data.get('total_profit', 0)
    max_drawdown = max(data.get('max_drawdown', 0.001), 0.001)
    total_trades = max(data.get('total_trades', 1), 1)
    winning_trades = data.get('winning_trades', 0)
    
    profit_ratio = total_profit / max_drawdown
    win_rate = winning_trades / total_trades
    sharpe_ratio = data.get('sharpe_ratio', 0)
    
    print(f'📊 PERFORMANCE ANALYSIS:')
    print(f'   Profit Ratio: {profit_ratio:.2f}')
    print(f'   Win Rate: {win_rate:.2%}')
    print(f'   Sharpe Ratio: {sharpe_ratio:.2f}')
    print(f'   Total Profit: {total_profit:.4f} SOL')
    print(f'   Max Drawdown: {max_drawdown:.4f} SOL')
    print(f'   Total Trades: {total_trades}')
    
    # Performance targets
    if profit_ratio > 2.0:
        print('✅ Profit ratio target met')
    else:
        print('⚠️  Profit ratio below target (2.0)')
        
    if win_rate > 0.6:
        print('✅ Win rate target met')
    else:
        print('⚠️  Win rate below target (60%)')
        
except Exception as e:
    print(f'❌ Error analyzing metrics: {e}')
    sys.exit(1)
"
    else
        log_warning "Python3 not available for detailed analysis"
        echo "📊 Basic metrics from metrics.json:"
        cat metrics.json | head -20
    fi
}

# Emergency procedures
emergency_stop() {
    log_error "EMERGENCY STOP INITIATED"
    
    # Stop all strategies
    curl -X POST http://localhost:8080/strategy/sandwich/disable 2>/dev/null || true
    curl -X POST http://localhost:8080/strategy/arbitrage/disable 2>/dev/null || true
    curl -X POST http://localhost:8080/strategy/sniping/disable 2>/dev/null || true
    curl -X POST http://localhost:8080/strategy/jupiter_arbitrage/disable 2>/dev/null || true
    curl -X POST http://localhost:8080/strategy/liquidation/disable 2>/dev/null || true
    
    # Close all positions
    curl -X POST http://localhost:8080/positions/close-all 2>/dev/null || true
    
    # Reset circuit breaker
    curl -X POST http://localhost:8080/risk/reset 2>/dev/null || true
    
    log_success "Emergency stop completed"
}

# Main execution
main() {
    case "${1:-all}" in
        "test")
            check_prerequisites
            run_unit_tests
            run_integration_tests
            ;;
        "sandwich")
            check_prerequisites
            run_strategy_isolated "sandwich" 300
            monitor_strategy "sandwich"
            ;;
        "arbitrage")
            check_prerequisites
            run_strategy_isolated "arbitrage" 300
            monitor_strategy "arbitrage"
            ;;
        "sniping")
            check_prerequisites
            run_strategy_isolated "sniping" 300
            monitor_strategy "sniping"
            ;;
        "jupiter_arbitrage")
            check_prerequisites
            run_strategy_isolated "jupiter_arbitrage" 300
            monitor_strategy "jupiter_arbitrage"
            ;;
        "liquidation")
            check_prerequisites
            run_strategy_isolated "liquidation" 300
            monitor_strategy "liquidation"
            ;;
        "all")
            check_prerequisites
            run_unit_tests
            run_integration_tests
            run_all_strategies
            ;;
        "analyze")
            analyze_performance
            ;;
        "emergency")
            emergency_stop
            ;;
        *)
            echo "Usage: $0 {test|sandwich|arbitrage|sniping|jupiter_arbitrage|liquidation|all|analyze|emergency}"
            echo ""
            echo "Environment variables:"
            echo "  INITIAL_BALANCE=8.0    # Starting balance in SOL"
            echo "  DRY_RUN=true          # Run in simulation mode"
            echo "  DURATION=3600         # Run duration in seconds"
            echo "  ENABLE_SANDWICH=true  # Enable sandwich strategy"
            echo "  ENABLE_ARBITRAGE=true # Enable arbitrage strategy"
            echo "  ENABLE_SNIPING=true   # Enable sniping strategy"
            echo "  ENABLE_JUPITER_ARB=true # Enable Jupiter arbitrage"
            echo "  ENABLE_LIQUIDATION=true # Enable liquidation strategy"
            exit 1
            ;;
    esac
}

# Trap for emergency stop
trap emergency_stop SIGINT SIGTERM

# Run main function
main "$@"
