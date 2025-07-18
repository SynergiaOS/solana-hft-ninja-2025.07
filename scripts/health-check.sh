#!/bin/bash

# 🏥 HFT Ninja Health Check
# Comprehensive health check for all services and endpoints

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🏥 Solana HFT Ninja - Health Check${NC}"
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
    local timeout=${4:-10}
    
    local start_time=$(date +%s%N)
    local response=$(curl -s -w "%{http_code}:%{time_total}" -m $timeout "$url" 2>/dev/null || echo "000:0")
    local end_time=$(date +%s%N)
    
    local http_code=$(echo "$response" | cut -d: -f1)
    local time_total=$(echo "$response" | cut -d: -f2)
    local total_time=$(( (end_time - start_time) / 1000000 ))
    
    if [ "$http_code" = "$expected_status" ]; then
        echo -e "  • $name: ${GREEN}✅ HEALTHY${NC} (${time_total}s, HTTP $http_code)"
        return 0
    elif [ "$http_code" = "000" ]; then
        echo -e "  • $name: ${RED}❌ TIMEOUT/ERROR${NC} (${timeout}s timeout)"
        return 1
    else
        echo -e "  • $name: ${YELLOW}⚠️  UNEXPECTED${NC} (${time_total}s, HTTP $http_code)"
        return 1
    fi
}

# Function to test API functionality
test_api_functionality() {
    local name=$1
    local url=$2
    local method=${3:-GET}
    local data=${4:-""}
    local expected_field=${5:-""}
    
    local start_time=$(date +%s%N)
    
    if [ "$method" = "POST" ] && [ -n "$data" ]; then
        local response=$(curl -s -X POST -H "Content-Type: application/json" -d "$data" "$url" 2>/dev/null || echo "")
    else
        local response=$(curl -s "$url" 2>/dev/null || echo "")
    fi
    
    local end_time=$(date +%s%N)
    local total_time=$(( (end_time - start_time) / 1000000 ))
    
    if [ -n "$response" ]; then
        if [ -n "$expected_field" ]; then
            local field_value=$(echo "$response" | jq -r ".$expected_field" 2>/dev/null || echo "null")
            if [ "$field_value" != "null" ] && [ "$field_value" != "" ]; then
                echo -e "  • $name: ${GREEN}✅ FUNCTIONAL${NC} (${total_time}ms, $expected_field: $field_value)"
                return 0
            else
                echo -e "  • $name: ${YELLOW}⚠️  PARTIAL${NC} (${total_time}ms, missing $expected_field)"
                return 1
            fi
        else
            echo -e "  • $name: ${GREEN}✅ RESPONDING${NC} (${total_time}ms)"
            return 0
        fi
    else
        echo -e "  • $name: ${RED}❌ NO RESPONSE${NC}"
        return 1
    fi
}

# Function to check basic health endpoints
check_basic_health() {
    echo -e "${YELLOW}🔍 Basic Health Checks:${NC}"
    
    local healthy=0
    local total=3
    
    # AI API Health
    if test_endpoint "AI API Health" "http://localhost:8003/health" 200 5; then
        healthy=$((healthy + 1))
    fi
    
    # BFF Health
    if test_endpoint "BFF Health" "http://localhost:8002/health" 200 5; then
        healthy=$((healthy + 1))
    fi
    
    # Frontend
    if test_endpoint "Frontend" "http://localhost:3000" 200 5; then
        healthy=$((healthy + 1))
    fi
    
    echo -e "  ${BLUE}Summary: $healthy/$total services healthy${NC}"
    return $((total - healthy))
}

# Function to check API functionality
check_api_functionality() {
    echo -e "${YELLOW}🧪 API Functionality Tests:${NC}"
    
    local functional=0
    local total=6
    
    # Trading Signals
    if test_api_functionality "Trading Signals" "http://localhost:8002/api/trading/signals" "GET" "" "signals"; then
        functional=$((functional + 1))
    fi
    
    # Trading Status
    if test_api_functionality "Trading Status" "http://localhost:8002/api/trading/status" "GET" "" "status"; then
        functional=$((functional + 1))
    fi
    
    # Wallet Balance
    if test_api_functionality "Wallet Balance" "http://localhost:8002/api/wallet/balance" "GET" "" "balance"; then
        functional=$((functional + 1))
    fi
    
    # Strategies List
    if test_api_functionality "Strategies List" "http://localhost:8002/api/strategies" "GET" "" "strategies"; then
        functional=$((functional + 1))
    fi
    
    # AI Position Calculation
    local ai_payload='{"capital": 8.0, "risk_tolerance": 0.05, "expected_return": 0.15, "volatility": 0.3, "strategy": "test"}'
    if test_api_functionality "AI Position Calc" "http://localhost:8002/ai/calculate/position-size" "POST" "$ai_payload" "result"; then
        functional=$((functional + 1))
    fi
    
    # Trading Execution (dry run)
    local trade_payload='{"action": "buy", "token": "SOL", "amount": 0.1, "strategy": "test"}'
    if test_api_functionality "Trading Execute" "http://localhost:8002/api/trading/execute" "POST" "$trade_payload" "trade_id"; then
        functional=$((functional + 1))
    fi
    
    echo -e "  ${BLUE}Summary: $functional/$total endpoints functional${NC}"
    return $((total - functional))
}

# Function to check performance metrics
check_performance() {
    echo -e "${YELLOW}⚡ Performance Tests:${NC}"
    
    local performance_ok=0
    local total=3
    
    # AI API Performance
    local ai_start=$(date +%s%N)
    local ai_response=$(curl -s "http://localhost:8003/health" 2>/dev/null || echo "")
    local ai_end=$(date +%s%N)
    local ai_time=$(( (ai_end - ai_start) / 1000000 ))
    
    if [ $ai_time -lt 100 ]; then
        echo -e "  • AI API Latency: ${GREEN}✅ EXCELLENT${NC} (${ai_time}ms < 100ms)"
        performance_ok=$((performance_ok + 1))
    elif [ $ai_time -lt 500 ]; then
        echo -e "  • AI API Latency: ${YELLOW}⚠️  ACCEPTABLE${NC} (${ai_time}ms < 500ms)"
    else
        echo -e "  • AI API Latency: ${RED}❌ SLOW${NC} (${ai_time}ms > 500ms)"
    fi
    
    # BFF Performance
    local bff_start=$(date +%s%N)
    local bff_response=$(curl -s "http://localhost:8002/health" 2>/dev/null || echo "")
    local bff_end=$(date +%s%N)
    local bff_time=$(( (bff_end - bff_start) / 1000000 ))
    
    if [ $bff_time -lt 200 ]; then
        echo -e "  • BFF Latency: ${GREEN}✅ EXCELLENT${NC} (${bff_time}ms < 200ms)"
        performance_ok=$((performance_ok + 1))
    elif [ $bff_time -lt 1000 ]; then
        echo -e "  • BFF Latency: ${YELLOW}⚠️  ACCEPTABLE${NC} (${bff_time}ms < 1000ms)"
    else
        echo -e "  • BFF Latency: ${RED}❌ SLOW${NC} (${bff_time}ms > 1000ms)"
    fi
    
    # Memory Usage
    local memory_usage=$(free | awk 'NR==2{printf "%.0f", $3*100/$2}')
    if [ $memory_usage -lt 80 ]; then
        echo -e "  • Memory Usage: ${GREEN}✅ GOOD${NC} (${memory_usage}% < 80%)"
        performance_ok=$((performance_ok + 1))
    elif [ $memory_usage -lt 90 ]; then
        echo -e "  • Memory Usage: ${YELLOW}⚠️  HIGH${NC} (${memory_usage}% < 90%)"
    else
        echo -e "  • Memory Usage: ${RED}❌ CRITICAL${NC} (${memory_usage}% > 90%)"
    fi
    
    echo -e "  ${BLUE}Summary: $performance_ok/$total metrics optimal${NC}"
    return $((total - performance_ok))
}

# Function to check data integrity
check_data_integrity() {
    echo -e "${YELLOW}🔍 Data Integrity Tests:${NC}"
    
    local integrity_ok=0
    local total=3
    
    # Test AI calculation consistency
    local calc1=$(curl -s -X POST -H "Content-Type: application/json" \
        -d '{"capital": 10.0, "risk_tolerance": 0.05, "expected_return": 0.15, "volatility": 0.3, "strategy": "test"}' \
        "http://localhost:8002/ai/calculate/position-size" 2>/dev/null | jq -r '.result.position_size' 2>/dev/null || echo "null")
    
    local calc2=$(curl -s -X POST -H "Content-Type: application/json" \
        -d '{"capital": 10.0, "risk_tolerance": 0.05, "expected_return": 0.15, "volatility": 0.3, "strategy": "test"}' \
        "http://localhost:8002/ai/calculate/position-size" 2>/dev/null | jq -r '.result.position_size' 2>/dev/null || echo "null")
    
    if [ "$calc1" = "$calc2" ] && [ "$calc1" != "null" ]; then
        echo -e "  • AI Calculation Consistency: ${GREEN}✅ CONSISTENT${NC} ($calc1)"
        integrity_ok=$((integrity_ok + 1))
    else
        echo -e "  • AI Calculation Consistency: ${RED}❌ INCONSISTENT${NC} ($calc1 vs $calc2)"
    fi
    
    # Test trading signals format
    local signals=$(curl -s "http://localhost:8002/api/trading/signals" 2>/dev/null | jq -r '.signals | length' 2>/dev/null || echo "0")
    if [ "$signals" -gt 0 ]; then
        echo -e "  • Trading Signals Format: ${GREEN}✅ VALID${NC} ($signals signals)"
        integrity_ok=$((integrity_ok + 1))
    else
        echo -e "  • Trading Signals Format: ${RED}❌ INVALID${NC} (no signals)"
    fi
    
    # Test wallet balance format
    local balance=$(curl -s "http://localhost:8002/api/wallet/balance" 2>/dev/null | jq -r '.balance' 2>/dev/null || echo "null")
    if [ "$balance" != "null" ] && [ "$balance" != "" ]; then
        echo -e "  • Wallet Balance Format: ${GREEN}✅ VALID${NC} ($balance SOL)"
        integrity_ok=$((integrity_ok + 1))
    else
        echo -e "  • Wallet Balance Format: ${RED}❌ INVALID${NC}"
    fi
    
    echo -e "  ${BLUE}Summary: $integrity_ok/$total data checks passed${NC}"
    return $((total - integrity_ok))
}

# Function to generate health report
generate_health_report() {
    local basic_health=$1
    local api_functionality=$2
    local performance=$3
    local data_integrity=$4
    
    echo ""
    echo -e "${GREEN}📋 Health Report Summary${NC}"
    echo -e "${BLUE}========================${NC}"
    
    local total_score=$((basic_health + api_functionality + performance + data_integrity))
    local max_score=15  # 3+6+3+3
    local health_percentage=$(( (max_score - total_score) * 100 / max_score ))
    
    echo -e "  • Basic Health: $((3 - basic_health))/3"
    echo -e "  • API Functionality: $((6 - api_functionality))/6"
    echo -e "  • Performance: $((3 - performance))/3"
    echo -e "  • Data Integrity: $((3 - data_integrity))/3"
    echo ""
    echo -e "  • Overall Health: ${health_percentage}%"
    
    if [ $health_percentage -ge 90 ]; then
        echo -e "  • Status: ${GREEN}🎉 EXCELLENT${NC}"
        echo -e "  • Recommendation: ${GREEN}System is ready for trading${NC}"
    elif [ $health_percentage -ge 75 ]; then
        echo -e "  • Status: ${YELLOW}⚠️  GOOD${NC}"
        echo -e "  • Recommendation: ${YELLOW}Minor issues detected, monitor closely${NC}"
    elif [ $health_percentage -ge 50 ]; then
        echo -e "  • Status: ${YELLOW}⚠️  FAIR${NC}"
        echo -e "  • Recommendation: ${YELLOW}Several issues detected, investigate${NC}"
    else
        echo -e "  • Status: ${RED}❌ POOR${NC}"
        echo -e "  • Recommendation: ${RED}Critical issues detected, fix before trading${NC}"
    fi
}

# Main execution
main() {
    # Run all health checks
    check_basic_health
    local basic_result=$?
    echo ""
    
    check_api_functionality
    local api_result=$?
    echo ""
    
    check_performance
    local performance_result=$?
    echo ""
    
    check_data_integrity
    local integrity_result=$?
    
    # Generate final report
    generate_health_report $basic_result $api_result $performance_result $integrity_result
    
    echo ""
    echo -e "${GREEN}🏥 Health check completed!${NC}"
    
    # Return overall status
    local total_issues=$((basic_result + api_result + performance_result + integrity_result))
    return $total_issues
}

# Run main function
main "$@"
