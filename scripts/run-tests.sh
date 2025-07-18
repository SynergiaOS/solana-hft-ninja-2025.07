#!/bin/bash

# 🧪 HFT Ninja Test Suite
# Comprehensive testing for all components

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TEST_TYPE=${1:-all}
VERBOSE=${2:-false}

echo -e "${BLUE}🧪 Solana HFT Ninja - Test Suite${NC}"
echo -e "${GREEN}========================================${NC}"
echo -e "${YELLOW}Test Type: $TEST_TYPE${NC}"

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

# Function to run test and capture result
run_test() {
    local test_name=$1
    local test_command=$2
    local expected_result=${3:-0}
    
    print_status "Running: $test_name"
    
    if [ "$VERBOSE" = true ]; then
        echo -e "${BLUE}Command: $test_command${NC}"
    fi
    
    local start_time=$(date +%s%N)
    
    if eval "$test_command" > /tmp/test_output.log 2>&1; then
        local result=0
    else
        local result=$?
    fi
    
    local end_time=$(date +%s%N)
    local duration=$(( (end_time - start_time) / 1000000 ))
    
    if [ $result -eq $expected_result ]; then
        echo -e "  ✅ ${GREEN}PASSED${NC} (${duration}ms)"
        if [ "$VERBOSE" = true ]; then
            cat /tmp/test_output.log | head -5
        fi
        return 0
    else
        echo -e "  ❌ ${RED}FAILED${NC} (${duration}ms, exit code: $result)"
        if [ "$VERBOSE" = true ]; then
            cat /tmp/test_output.log
        fi
        return 1
    fi
}

# Function to test API endpoints
test_api_endpoints() {
    echo -e "${YELLOW}🔗 API Endpoint Tests:${NC}"
    
    local passed=0
    local total=8
    
    # Health endpoints
    if run_test "AI Health Check" "curl -s -f http://localhost:8003/health"; then
        passed=$((passed + 1))
    fi
    
    if run_test "BFF Health Check" "curl -s -f http://localhost:8002/health"; then
        passed=$((passed + 1))
    fi
    
    # Trading endpoints
    if run_test "Trading Signals" "curl -s -f http://localhost:8002/api/trading/signals"; then
        passed=$((passed + 1))
    fi
    
    if run_test "Trading Status" "curl -s -f http://localhost:8002/api/trading/status"; then
        passed=$((passed + 1))
    fi
    
    if run_test "Wallet Balance" "curl -s -f http://localhost:8002/api/wallet/balance"; then
        passed=$((passed + 1))
    fi
    
    if run_test "Strategies List" "curl -s -f http://localhost:8002/api/strategies"; then
        passed=$((passed + 1))
    fi
    
    # AI proxy endpoints
    if run_test "AI Proxy Health" "curl -s -f http://localhost:8002/ai/health"; then
        passed=$((passed + 1))
    fi
    
    # Frontend
    if run_test "Frontend Access" "curl -s -f http://localhost:3000"; then
        passed=$((passed + 1))
    fi
    
    echo -e "  ${BLUE}API Tests: $passed/$total passed${NC}"
    return $((total - passed))
}

# Function to test trading functionality
test_trading_functionality() {
    echo -e "${YELLOW}💰 Trading Functionality Tests:${NC}"
    
    local passed=0
    local total=5
    
    # Test buy order
    local buy_payload='{"action": "buy", "token": "SOL", "amount": 0.1, "strategy": "test"}'
    if run_test "Buy Order Execution" "curl -s -X POST -H 'Content-Type: application/json' -d '$buy_payload' http://localhost:8002/api/trading/execute"; then
        passed=$((passed + 1))
    fi
    
    # Test sell order
    local sell_payload='{"action": "sell", "token": "SOL", "amount": 0.1, "strategy": "test"}'
    if run_test "Sell Order Execution" "curl -s -X POST -H 'Content-Type: application/json' -d '$sell_payload' http://localhost:8002/api/trading/execute"; then
        passed=$((passed + 1))
    fi
    
    # Test hold signal
    local hold_payload='{"action": "hold", "token": "SOL", "amount": 0.0, "strategy": "test"}'
    if run_test "Hold Signal" "curl -s -X POST -H 'Content-Type: application/json' -d '$hold_payload' http://localhost:8002/api/trading/execute"; then
        passed=$((passed + 1))
    fi
    
    # Test trading history
    if run_test "Trading History" "curl -s -f http://localhost:8002/api/trading/history"; then
        passed=$((passed + 1))
    fi
    
    # Test portfolio status
    if run_test "Portfolio Status" "curl -s -f http://localhost:8002/api/portfolio/status"; then
        passed=$((passed + 1))
    fi
    
    echo -e "  ${BLUE}Trading Tests: $passed/$total passed${NC}"
    return $((total - passed))
}

# Function to test AI calculations
test_ai_calculations() {
    echo -e "${YELLOW}🧮 AI Calculation Tests:${NC}"
    
    local passed=0
    local total=4
    
    # Test position size calculation
    local position_payload='{"capital": 10.0, "risk_tolerance": 0.05, "expected_return": 0.15, "volatility": 0.3, "strategy": "test"}'
    if run_test "Position Size Calculation" "curl -s -X POST -H 'Content-Type: application/json' -d '$position_payload' http://localhost:8002/ai/calculate/position-size"; then
        passed=$((passed + 1))
    fi
    
    # Test arbitrage analysis
    local arbitrage_payload='{"token_a": "SOL", "token_b": "USDC", "amount": 1.0, "dex_a": "raydium", "dex_b": "orca"}'
    if run_test "Arbitrage Analysis" "curl -s -X POST -H 'Content-Type: application/json' -d '$arbitrage_payload' http://localhost:8002/ai/calculate/arbitrage"; then
        passed=$((passed + 1))
    fi
    
    # Test risk assessment
    local risk_payload='{"portfolio": {"SOL": 2.5, "USDC": 450.0}, "market_conditions": "volatile"}'
    if run_test "Risk Assessment" "curl -s -X POST -H 'Content-Type: application/json' -d '$risk_payload' http://localhost:8002/ai/calculate/risk"; then
        passed=$((passed + 1))
    fi
    
    # Test market prediction
    local prediction_payload='{"token": "SOL", "timeframe": "1h", "indicators": ["rsi", "macd", "volume"]}'
    if run_test "Market Prediction" "curl -s -X POST -H 'Content-Type: application/json' -d '$prediction_payload' http://localhost:8002/ai/calculate/prediction"; then
        passed=$((passed + 1))
    fi
    
    echo -e "  ${BLUE}AI Tests: $passed/$total passed${NC}"
    return $((total - passed))
}

# Function to test performance
test_performance() {
    echo -e "${YELLOW}⚡ Performance Tests:${NC}"
    
    local passed=0
    local total=3
    
    # Test API response time
    local start_time=$(date +%s%N)
    curl -s http://localhost:8002/health > /dev/null
    local end_time=$(date +%s%N)
    local response_time=$(( (end_time - start_time) / 1000000 ))
    
    if [ $response_time -lt 200 ]; then
        echo -e "  ✅ ${GREEN}API Response Time: ${response_time}ms < 200ms${NC}"
        passed=$((passed + 1))
    else
        echo -e "  ❌ ${RED}API Response Time: ${response_time}ms > 200ms${NC}"
    fi
    
    # Test concurrent requests
    print_status "Testing concurrent requests..."
    local concurrent_start=$(date +%s%N)
    
    for i in {1..10}; do
        curl -s http://localhost:8002/health > /dev/null &
    done
    wait
    
    local concurrent_end=$(date +%s%N)
    local concurrent_time=$(( (concurrent_end - concurrent_start) / 1000000 ))
    
    if [ $concurrent_time -lt 1000 ]; then
        echo -e "  ✅ ${GREEN}Concurrent Requests: ${concurrent_time}ms < 1000ms${NC}"
        passed=$((passed + 1))
    else
        echo -e "  ❌ ${RED}Concurrent Requests: ${concurrent_time}ms > 1000ms${NC}"
    fi
    
    # Test memory usage
    local memory_usage=$(free | awk 'NR==2{printf "%.0f", $3*100/$2}')
    if [ $memory_usage -lt 80 ]; then
        echo -e "  ✅ ${GREEN}Memory Usage: ${memory_usage}% < 80%${NC}"
        passed=$((passed + 1))
    else
        echo -e "  ❌ ${RED}Memory Usage: ${memory_usage}% > 80%${NC}"
    fi
    
    echo -e "  ${BLUE}Performance Tests: $passed/$total passed${NC}"
    return $((total - passed))
}

# Function to test data validation
test_data_validation() {
    echo -e "${YELLOW}🔍 Data Validation Tests:${NC}"
    
    local passed=0
    local total=4
    
    # Test invalid trading payload
    local invalid_payload='{"action": "invalid", "token": "", "amount": -1}'
    if run_test "Invalid Trading Payload" "curl -s -X POST -H 'Content-Type: application/json' -d '$invalid_payload' http://localhost:8002/api/trading/execute" 1; then
        passed=$((passed + 1))
    fi
    
    # Test missing required fields
    local missing_payload='{"action": "buy"}'
    if run_test "Missing Required Fields" "curl -s -X POST -H 'Content-Type: application/json' -d '$missing_payload' http://localhost:8002/api/trading/execute" 1; then
        passed=$((passed + 1))
    fi
    
    # Test invalid AI calculation
    local invalid_ai_payload='{"capital": -10, "risk_tolerance": 2.0}'
    if run_test "Invalid AI Calculation" "curl -s -X POST -H 'Content-Type: application/json' -d '$invalid_ai_payload' http://localhost:8002/ai/calculate/position-size" 1; then
        passed=$((passed + 1))
    fi
    
    # Test malformed JSON
    local malformed_payload='{"action": "buy", "token": "SOL"'
    if run_test "Malformed JSON" "curl -s -X POST -H 'Content-Type: application/json' -d '$malformed_payload' http://localhost:8002/api/trading/execute" 1; then
        passed=$((passed + 1))
    fi
    
    echo -e "  ${BLUE}Validation Tests: $passed/$total passed${NC}"
    return $((total - passed))
}

# Function to run all tests
run_all_tests() {
    local total_failed=0
    
    test_api_endpoints
    total_failed=$((total_failed + $?))
    echo ""
    
    test_trading_functionality
    total_failed=$((total_failed + $?))
    echo ""
    
    test_ai_calculations
    total_failed=$((total_failed + $?))
    echo ""
    
    test_performance
    total_failed=$((total_failed + $?))
    echo ""
    
    test_data_validation
    total_failed=$((total_failed + $?))
    
    return $total_failed
}

# Function to generate test report
generate_test_report() {
    local failed_tests=$1
    local total_tests=24  # Sum of all test categories
    local passed_tests=$((total_tests - failed_tests))
    local success_rate=$(( passed_tests * 100 / total_tests ))
    
    echo ""
    echo -e "${GREEN}📋 Test Report Summary${NC}"
    echo -e "${BLUE}=======================${NC}"
    echo -e "  • Total Tests: $total_tests"
    echo -e "  • Passed: ${GREEN}$passed_tests${NC}"
    echo -e "  • Failed: ${RED}$failed_tests${NC}"
    echo -e "  • Success Rate: $success_rate%"
    echo ""
    
    if [ $success_rate -ge 95 ]; then
        echo -e "  • Status: ${GREEN}🎉 EXCELLENT${NC}"
        echo -e "  • Recommendation: ${GREEN}System is ready for production${NC}"
    elif [ $success_rate -ge 85 ]; then
        echo -e "  • Status: ${YELLOW}⚠️  GOOD${NC}"
        echo -e "  • Recommendation: ${YELLOW}Minor issues, acceptable for testing${NC}"
    elif [ $success_rate -ge 70 ]; then
        echo -e "  • Status: ${YELLOW}⚠️  FAIR${NC}"
        echo -e "  • Recommendation: ${YELLOW}Several issues, investigate before production${NC}"
    else
        echo -e "  • Status: ${RED}❌ POOR${NC}"
        echo -e "  • Recommendation: ${RED}Critical issues, fix before proceeding${NC}"
    fi
}

# Main execution
main() {
    # Check if services are running
    if ! curl -s -f http://localhost:8002/health > /dev/null 2>&1; then
        print_error "Services are not running. Please start with 'make dev' first."
        exit 1
    fi
    
    local failed_tests=0
    
    case $TEST_TYPE in
        "api")
            test_api_endpoints
            failed_tests=$?
            ;;
        "trading")
            test_trading_functionality
            failed_tests=$?
            ;;
        "ai")
            test_ai_calculations
            failed_tests=$?
            ;;
        "performance")
            test_performance
            failed_tests=$?
            ;;
        "validation")
            test_data_validation
            failed_tests=$?
            ;;
        "all")
            run_all_tests
            failed_tests=$?
            ;;
        *)
            print_error "Unknown test type: $TEST_TYPE"
            print_status "Available types: all, api, trading, ai, performance, validation"
            exit 1
            ;;
    esac
    
    # Generate report for comprehensive tests
    if [ "$TEST_TYPE" = "all" ]; then
        generate_test_report $failed_tests
    fi
    
    echo ""
    echo -e "${GREEN}🧪 Test suite completed!${NC}"
    
    # Cleanup
    rm -f /tmp/test_output.log
    
    return $failed_tests
}

# Run main function
main "$@"
