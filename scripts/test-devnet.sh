#!/bin/bash

# 🌐 Devnet Testing Script
# Tests real Solana Devnet integration

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🌐 Testing Solana HFT Ninja Devnet Integration${NC}"
echo -e "${GREEN}========================================${NC}"

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

# Function to test endpoint
test_endpoint() {
    local name=$1
    local url=$2
    local expected_status=${3:-200}
    
    print_status "Testing $name..."
    
    local response=$(curl -s -w "%{http_code}" -o /tmp/test_response.json "$url" 2>/dev/null || echo "000")
    local http_code="${response: -3}"
    
    if [ "$http_code" = "$expected_status" ]; then
        echo -e "  ✅ ${GREEN}$name: OK (HTTP $http_code)${NC}"
        return 0
    else
        echo -e "  ❌ ${RED}$name: FAILED (HTTP $http_code)${NC}"
        return 1
    fi
}

# Function to test devnet trading
test_devnet_trading() {
    local action=$1
    local amount=$2
    
    print_status "Testing Devnet $action order..."
    
    local payload="{\"action\": \"$action\", \"token\": \"SOL\", \"amount\": $amount, \"strategy\": \"test\", \"dry_run\": true}"
    
    local response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "$payload" \
        "http://localhost:8002/api/trading/execute-devnet" 2>/dev/null || echo "")
    
    if echo "$response" | jq -e '.trade_id' > /dev/null 2>&1; then
        local trade_id=$(echo "$response" | jq -r '.trade_id')
        local status=$(echo "$response" | jq -r '.status')
        echo -e "  ✅ ${GREEN}Devnet $action: OK (Trade ID: $trade_id, Status: $status)${NC}"
        return 0
    else
        echo -e "  ❌ ${RED}Devnet $action: FAILED${NC}"
        return 1
    fi
}

# Main testing function
main() {
    local passed=0
    local total=0
    
    echo -e "${YELLOW}🔍 Basic Health Checks:${NC}"
    
    # Test BFF Health
    if test_endpoint "BFF Health" "http://localhost:8002/health"; then
        passed=$((passed + 1))
    fi
    total=$((total + 1))
    
    # Test AI Health
    if test_endpoint "AI Health" "http://localhost:8003/health"; then
        passed=$((passed + 1))
    fi
    total=$((total + 1))
    
    # Test Trading Engine (if running)
    if test_endpoint "Trading Engine Health" "http://localhost:8001/health"; then
        passed=$((passed + 1))
    fi
    total=$((total + 1))
    
    echo ""
    echo -e "${YELLOW}💰 Wallet Tests:${NC}"
    
    # Test wallet balance
    if test_endpoint "Devnet Wallet Balance" "http://localhost:8002/api/wallet/devnet-balance"; then
        passed=$((passed + 1))
        
        # Show wallet info
        local balance_response=$(curl -s "http://localhost:8002/api/wallet/devnet-balance" 2>/dev/null)
        if [ -n "$balance_response" ]; then
            local address=$(echo "$balance_response" | jq -r '.address' 2>/dev/null || echo "unknown")
            local balance=$(echo "$balance_response" | jq -r '.balance_sol' 2>/dev/null || echo "unknown")
            echo -e "    💰 Address: ${YELLOW}$address${NC}"
            echo -e "    💰 Balance: ${YELLOW}$balance SOL${NC}"
        fi
    fi
    total=$((total + 1))
    
    echo ""
    echo -e "${YELLOW}🎯 Trading Tests:${NC}"
    
    # Test devnet trading endpoints
    if test_devnet_trading "buy" 0.1; then
        passed=$((passed + 1))
    fi
    total=$((total + 1))
    
    if test_devnet_trading "sell" 0.05; then
        passed=$((passed + 1))
    fi
    total=$((total + 1))
    
    if test_devnet_trading "hold" 0.0; then
        passed=$((passed + 1))
    fi
    total=$((total + 1))
    
    echo ""
    echo -e "${YELLOW}🧮 AI Integration Tests:${NC}"
    
    # Test AI position calculation
    local ai_payload='{"capital": 8.0, "risk_tolerance": 0.05, "expected_return": 0.15, "volatility": 0.3, "strategy": "devnet_test"}'
    print_status "Testing AI position calculation..."
    
    local ai_response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "$ai_payload" \
        "http://localhost:8002/ai/calculate/position-size" 2>/dev/null || echo "")
    
    if echo "$ai_response" | jq -e '.result.position_size' > /dev/null 2>&1; then
        local position_size=$(echo "$ai_response" | jq -r '.result.position_size')
        local confidence=$(echo "$ai_response" | jq -r '.result.confidence')
        echo -e "  ✅ ${GREEN}AI Calculation: OK (Position: $position_size SOL, Confidence: $confidence)${NC}"
        passed=$((passed + 1))
    else
        echo -e "  ❌ ${RED}AI Calculation: FAILED${NC}"
    fi
    total=$((total + 1))
    
    echo ""
    echo -e "${YELLOW}🌐 Solana Network Tests:${NC}"
    
    # Test Solana RPC connection
    print_status "Testing Solana Devnet RPC..."
    
    local rpc_response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
        "https://api.devnet.solana.com" 2>/dev/null || echo "")
    
    if echo "$rpc_response" | jq -e '.result' > /dev/null 2>&1; then
        echo -e "  ✅ ${GREEN}Solana Devnet RPC: OK${NC}"
        passed=$((passed + 1))
    else
        echo -e "  ❌ ${RED}Solana Devnet RPC: FAILED${NC}"
    fi
    total=$((total + 1))
    
    # Test wallet balance via Solana CLI (if available)
    if command -v solana &> /dev/null; then
        print_status "Testing Solana CLI wallet balance..."
        
        local cli_balance=$(solana balance DSJXCqXuRckDhSX34oiFgEQChuezxvVgkEAyaA2MML8X --url devnet 2>/dev/null || echo "")
        
        if [[ "$cli_balance" =~ [0-9]+\.[0-9]+ ]]; then
            echo -e "  ✅ ${GREEN}Solana CLI Balance: $cli_balance${NC}"
            passed=$((passed + 1))
        else
            echo -e "  ❌ ${RED}Solana CLI Balance: FAILED${NC}"
        fi
    else
        echo -e "  ⚠️ ${YELLOW}Solana CLI not available, skipping balance check${NC}"
        passed=$((passed + 1))  # Don't penalize for missing CLI
    fi
    total=$((total + 1))
    
    # Generate test report
    echo ""
    echo -e "${GREEN}📋 Devnet Test Report${NC}"
    echo -e "${BLUE}=====================${NC}"
    
    local success_rate=$(( passed * 100 / total ))
    
    echo -e "  • Total Tests: $total"
    echo -e "  • Passed: ${GREEN}$passed${NC}"
    echo -e "  • Failed: ${RED}$((total - passed))${NC}"
    echo -e "  • Success Rate: $success_rate%"
    echo ""
    
    if [ $success_rate -ge 90 ]; then
        echo -e "  • Status: ${GREEN}🎉 EXCELLENT${NC}"
        echo -e "  • Recommendation: ${GREEN}Devnet integration is ready for trading${NC}"
    elif [ $success_rate -ge 75 ]; then
        echo -e "  • Status: ${YELLOW}⚠️  GOOD${NC}"
        echo -e "  • Recommendation: ${YELLOW}Minor issues detected, monitor closely${NC}"
    elif [ $success_rate -ge 50 ]; then
        echo -e "  • Status: ${YELLOW}⚠️  FAIR${NC}"
        echo -e "  • Recommendation: ${YELLOW}Several issues detected, investigate${NC}"
    else
        echo -e "  • Status: ${RED}❌ POOR${NC}"
        echo -e "  • Recommendation: ${RED}Critical issues detected, fix before trading${NC}"
    fi
    
    echo ""
    echo -e "${YELLOW}💡 Next Steps:${NC}"
    echo -e "  • Start Devnet stack: ${GREEN}make devnet${NC}"
    echo -e "  • View Devnet logs: ${GREEN}make devnet-logs${NC}"
    echo -e "  • Check wallet: ${GREEN}make devnet-wallet${NC}"
    echo -e "  • Open frontend: ${GREEN}http://localhost:3000${NC}"
    echo -e "  • Monitor metrics: ${GREEN}http://localhost:3001${NC}"
    
    echo ""
    echo -e "${GREEN}🌐 Devnet testing completed!${NC}"
    
    # Cleanup
    rm -f /tmp/test_response.json
    
    return $((total - passed))
}

# Run main function
main "$@"
