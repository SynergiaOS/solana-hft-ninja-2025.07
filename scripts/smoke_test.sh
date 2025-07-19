#!/bin/bash
# 🚀 FAZA 3/3: Smoke Test na Devnet Arena

set -e

echo "🚀 FAZA 3/3: Smoke Test na Devnet Arena"
echo "======================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default parameters
DURATION=900  # 15 minutes
CAPITAL=0.01  # 0.01 SOL
STRATEGIES="arbitrage,sandwich"
DRY_RUN=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --duration=*)
            DURATION="${1#*=}"
            shift
            ;;
        --capital=*)
            CAPITAL="${1#*=}"
            shift
            ;;
        --strategies=*)
            STRATEGIES="${1#*=}"
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        *)
            echo "Unknown option $1"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}📋 Test Configuration:${NC}"
echo "  Duration: ${DURATION} seconds"
echo "  Capital: ${CAPITAL} SOL"
echo "  Strategies: ${STRATEGIES}"
echo "  Dry Run: ${DRY_RUN}"

# Results file
RESULTS_FILE="smoke_test_results.txt"
echo "# Smoke Test Results - $(date)" > $RESULTS_FILE

# Function to check if service is healthy
check_health() {
    local service_name="$1"
    local health_url="$2"
    local max_attempts=30
    local attempt=1
    
    echo -e "${YELLOW}🔍 Checking $service_name health...${NC}"
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s "$health_url" > /dev/null 2>&1; then
            echo -e "${GREEN}✅ $service_name is healthy${NC}"
            return 0
        fi
        
        echo -n "  Attempt $attempt/$max_attempts... "
        sleep 2
        attempt=$((attempt + 1))
    done
    
    echo -e "${RED}❌ $service_name health check failed${NC}"
    return 1
}

# Function to run devnet trader
run_devnet_trader() {
    local duration=$1
    local capital=$2
    local strategies=$3
    local dry_run_flag=""

    if [ "$DRY_RUN" = true ]; then
        dry_run_flag="--dry-run"
    fi

    echo -e "${BLUE}🚀 Starting devnet trader...${NC}"
    echo "Command: cargo run --release --bin devnet_trader -- --strategy $strategies --duration $duration --max-position $capital $dry_run_flag"

    # Run in background and capture PID
    cargo run --release --bin devnet_trader -- \
        --strategy "$strategies" \
        --duration "$duration" \
        --max-position "$capital" \
        $dry_run_flag > devnet_trader.log 2>&1 &
    
    local trader_pid=$!
    echo "Trader PID: $trader_pid"
    
    # Wait for completion or timeout
    local elapsed=0
    local check_interval=10
    
    while [ $elapsed -lt $duration ]; do
        if ! kill -0 $trader_pid 2>/dev/null; then
            echo -e "${YELLOW}⚠️  Trader process ended early${NC}"
            break
        fi
        
        echo -e "${BLUE}📊 Trading progress: ${elapsed}/${duration}s${NC}"
        sleep $check_interval
        elapsed=$((elapsed + check_interval))
    done
    
    # Stop trader if still running
    if kill -0 $trader_pid 2>/dev/null; then
        echo -e "${YELLOW}⏹️  Stopping trader...${NC}"
        kill $trader_pid
        sleep 2
    fi
    
    return 0
}

# Function to analyze results
analyze_results() {
    echo -e "${BLUE}📊 Analyzing results...${NC}"
    
    if [ ! -f "devnet_trader.log" ]; then
        echo -e "${RED}❌ No trader log found${NC}"
        return 1
    fi
    
    # Extract metrics from log
    local total_trades=$(grep -c "Trade executed" devnet_trader.log || echo "0")
    local successful_trades=$(grep -c "Trade successful" devnet_trader.log || echo "0")
    local failed_trades=$(grep -c "Trade failed" devnet_trader.log || echo "0")
    local total_profit=$(grep "Total profit" devnet_trader.log | tail -1 | awk '{print $3}' || echo "0")
    local max_drawdown=$(grep "Max drawdown" devnet_trader.log | tail -1 | awk '{print $3}' || echo "0")
    local avg_latency=$(grep "Average latency" devnet_trader.log | tail -1 | awk '{print $3}' | sed 's/ms//' || echo "0")
    local p99_latency=$(grep "P99 latency" devnet_trader.log | tail -1 | awk '{print $3}' | sed 's/ms//' || echo "0")
    local error_count=$(grep -c "ERROR" devnet_trader.log || echo "0")
    
    echo "Results Summary:"
    echo "  Total trades: $total_trades"
    echo "  Successful trades: $successful_trades"
    echo "  Failed trades: $failed_trades"
    echo "  Total profit: $total_profit SOL"
    echo "  Max drawdown: $max_drawdown SOL"
    echo "  Average latency: ${avg_latency}ms"
    echo "  P99 latency: ${p99_latency}ms"
    echo "  Error count: $error_count"
    
    # Save to results file
    cat >> $RESULTS_FILE << EOF
SMOKE_TEST_RESULTS:
total_trades=$total_trades
successful_trades=$successful_trades
failed_trades=$failed_trades
total_profit=$total_profit
max_drawdown=$max_drawdown
avg_latency_ms=$avg_latency
p99_latency_ms=$p99_latency
error_count=$error_count
EOF
    
    return 0
}

# Main execution
echo -e "${YELLOW}🔧 Starting smoke test...${NC}"

# 1. Check if devnet_trader binary exists
if [ ! -f "target/release/devnet_trader" ]; then
    echo -e "${YELLOW}🔨 Building devnet_trader...${NC}"
    cargo build --release --bin devnet_trader
fi

# 2. Check Cerebro orchestrator (optional)
if [ -f "cerebro/target/release/cerebro-orchestrator" ]; then
    echo -e "${YELLOW}🧠 Starting Cerebro orchestrator...${NC}"
    cerebro/target/release/cerebro-orchestrator > cerebro.log 2>&1 &
    CEREBRO_PID=$!
    sleep 5
    
    # Check if Cerebro is healthy
    if check_health "Cerebro" "http://localhost:9091/metrics"; then
        echo "Cerebro metrics available"
    else
        echo -e "${YELLOW}⚠️  Cerebro not responding, continuing without it${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Cerebro not built, running without AI optimization${NC}"
fi

# 3. Run the actual trading test
echo -e "${BLUE}🎯 Starting trading simulation...${NC}"
run_devnet_trader "$DURATION" "$CAPITAL" "$STRATEGIES"

# 4. Analyze results
analyze_results

# 5. Cleanup
if [ ! -z "$CEREBRO_PID" ] && kill -0 $CEREBRO_PID 2>/dev/null; then
    echo -e "${YELLOW}🛑 Stopping Cerebro...${NC}"
    kill $CEREBRO_PID
fi

echo -e "${GREEN}✅ Smoke test completed!${NC}"
echo "Results saved to: $RESULTS_FILE"
echo "Trader log: devnet_trader.log"

# Show final summary
if [ -f "$RESULTS_FILE" ]; then
    echo -e "\n${BLUE}📊 Final Summary:${NC}"
    grep "=" $RESULTS_FILE | while IFS='=' read -r key value; do
        echo "  $key: $value"
    done
fi
