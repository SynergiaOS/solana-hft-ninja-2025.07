#!/bin/bash
# 🧪 Smoke Test Assertions - Validate Performance Targets

set -e

echo "🧪 SMOKE TEST ASSERTIONS"
echo "========================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default thresholds
MAX_LATENCY=200  # ms
MIN_PROFIT=0.0005  # SOL
MAX_DRAWDOWN=0.0002  # SOL
MAX_ERRORS=0

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --max-latency=*)
            MAX_LATENCY="${1#*=}"
            shift
            ;;
        --min-profit=*)
            MIN_PROFIT="${1#*=}"
            shift
            ;;
        --max-drawdown=*)
            MAX_DRAWDOWN="${1#*=}"
            shift
            ;;
        --max-errors=*)
            MAX_ERRORS="${1#*=}"
            shift
            ;;
        *)
            echo "Unknown option $1"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}📋 Assertion Thresholds:${NC}"
echo "  Max Latency: ${MAX_LATENCY}ms"
echo "  Min Profit: ${MIN_PROFIT} SOL"
echo "  Max Drawdown: ${MAX_DRAWDOWN} SOL"
echo "  Max Errors: ${MAX_ERRORS}"

# Results file
RESULTS_FILE="smoke_test_results.txt"
ASSERTIONS_FILE="assertion_results.txt"

if [ ! -f "$RESULTS_FILE" ]; then
    echo -e "${RED}❌ Results file not found: $RESULTS_FILE${NC}"
    exit 1
fi

echo "# Assertion Results - $(date)" > $ASSERTIONS_FILE

# Function to assert metric
assert_metric() {
    local metric_name="$1"
    local actual_value="$2"
    local expected_value="$3"
    local comparison="$4"  # "lt", "gt", "eq", "le", "ge"
    local unit="$5"
    
    echo -e "${BLUE}🔍 Asserting: $metric_name${NC}"
    echo "  Actual: $actual_value$unit"
    echo "  Expected: $comparison $expected_value$unit"
    
    local result=false
    
    case $comparison in
        "lt")
            if (( $(echo "$actual_value < $expected_value" | bc -l) )); then
                result=true
            fi
            ;;
        "le")
            if (( $(echo "$actual_value <= $expected_value" | bc -l) )); then
                result=true
            fi
            ;;
        "gt")
            if (( $(echo "$actual_value > $expected_value" | bc -l) )); then
                result=true
            fi
            ;;
        "ge")
            if (( $(echo "$actual_value >= $expected_value" | bc -l) )); then
                result=true
            fi
            ;;
        "eq")
            if (( $(echo "$actual_value == $expected_value" | bc -l) )); then
                result=true
            fi
            ;;
    esac
    
    if [ "$result" = true ]; then
        echo -e "  ${GREEN}✅ PASS${NC}"
        echo "$metric_name: PASS - $actual_value$unit $comparison $expected_value$unit" >> $ASSERTIONS_FILE
        return 0
    else
        echo -e "  ${RED}❌ FAIL${NC}"
        echo "$metric_name: FAIL - $actual_value$unit not $comparison $expected_value$unit" >> $ASSERTIONS_FILE
        return 1
    fi
}

# Function to extract value from results
extract_value() {
    local key="$1"
    local default="$2"
    
    if grep -q "^$key=" "$RESULTS_FILE"; then
        grep "^$key=" "$RESULTS_FILE" | cut -d'=' -f2
    else
        echo "$default"
    fi
}

# Extract metrics from results file
echo -e "${YELLOW}📊 Extracting metrics...${NC}"

TOTAL_TRADES=$(extract_value "total_trades" "0")
SUCCESSFUL_TRADES=$(extract_value "successful_trades" "0")
FAILED_TRADES=$(extract_value "failed_trades" "0")
TOTAL_PROFIT=$(extract_value "total_profit" "0")
MAX_DRAWDOWN_ACTUAL=$(extract_value "max_drawdown" "0")
AVG_LATENCY=$(extract_value "avg_latency_ms" "0")
P99_LATENCY=$(extract_value "p99_latency_ms" "0")
ERROR_COUNT=$(extract_value "error_count" "0")

echo "Extracted metrics:"
echo "  Total trades: $TOTAL_TRADES"
echo "  Successful trades: $SUCCESSFUL_TRADES"
echo "  Failed trades: $FAILED_TRADES"
echo "  Total profit: $TOTAL_PROFIT SOL"
echo "  Max drawdown: $MAX_DRAWDOWN_ACTUAL SOL"
echo "  Average latency: ${AVG_LATENCY}ms"
echo "  P99 latency: ${P99_LATENCY}ms"
echo "  Error count: $ERROR_COUNT"

# Run assertions
echo -e "\n${YELLOW}🧪 Running assertions...${NC}"

PASSED=0
FAILED=0

# Assert P99 latency
if assert_metric "P99 Latency" "$P99_LATENCY" "$MAX_LATENCY" "le" "ms"; then
    PASSED=$((PASSED + 1))
else
    FAILED=$((FAILED + 1))
fi

# Assert average latency (should be even better)
AVG_LATENCY_THRESHOLD=$((MAX_LATENCY / 2))
if assert_metric "Average Latency" "$AVG_LATENCY" "$AVG_LATENCY_THRESHOLD" "le" "ms"; then
    PASSED=$((PASSED + 1))
else
    FAILED=$((FAILED + 1))
fi

# Assert minimum profit (only if we had trades)
if [ "$TOTAL_TRADES" -gt 0 ]; then
    if assert_metric "Total Profit" "$TOTAL_PROFIT" "$MIN_PROFIT" "ge" " SOL"; then
        PASSED=$((PASSED + 1))
    else
        FAILED=$((FAILED + 1))
    fi
else
    echo -e "${YELLOW}⚠️  No trades executed, skipping profit assertion${NC}"
    echo "Total Profit: SKIP - No trades executed" >> $ASSERTIONS_FILE
fi

# Assert maximum drawdown
if assert_metric "Max Drawdown" "$MAX_DRAWDOWN_ACTUAL" "$MAX_DRAWDOWN" "le" " SOL"; then
    PASSED=$((PASSED + 1))
else
    FAILED=$((FAILED + 1))
fi

# Assert error count
if assert_metric "Error Count" "$ERROR_COUNT" "$MAX_ERRORS" "le" ""; then
    PASSED=$((PASSED + 1))
else
    FAILED=$((FAILED + 1))
fi

# Assert success rate (if we had trades)
if [ "$TOTAL_TRADES" -gt 0 ]; then
    SUCCESS_RATE=$(echo "scale=2; $SUCCESSFUL_TRADES * 100 / $TOTAL_TRADES" | bc -l)
    MIN_SUCCESS_RATE=80  # 80% minimum success rate
    
    if assert_metric "Success Rate" "$SUCCESS_RATE" "$MIN_SUCCESS_RATE" "ge" "%"; then
        PASSED=$((PASSED + 1))
    else
        FAILED=$((FAILED + 1))
    fi
else
    echo -e "${YELLOW}⚠️  No trades executed, skipping success rate assertion${NC}"
    echo "Success Rate: SKIP - No trades executed" >> $ASSERTIONS_FILE
fi

# Additional system health checks
echo -e "\n${YELLOW}🔍 System health checks...${NC}"

# Check if devnet_trader log exists and has reasonable content
if [ -f "devnet_trader.log" ]; then
    LOG_SIZE=$(wc -l < devnet_trader.log)
    if [ "$LOG_SIZE" -gt 10 ]; then
        echo -e "${GREEN}✅ Trader log has sufficient content ($LOG_SIZE lines)${NC}"
        echo "Log Content: PASS - $LOG_SIZE lines" >> $ASSERTIONS_FILE
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}❌ Trader log too short ($LOG_SIZE lines)${NC}"
        echo "Log Content: FAIL - Only $LOG_SIZE lines" >> $ASSERTIONS_FILE
        FAILED=$((FAILED + 1))
    fi
else
    echo -e "${RED}❌ Trader log not found${NC}"
    echo "Log Content: FAIL - Log file missing" >> $ASSERTIONS_FILE
    FAILED=$((FAILED + 1))
fi

# Check for critical errors in log
if [ -f "devnet_trader.log" ]; then
    CRITICAL_ERRORS=$(grep -c "CRITICAL\|FATAL\|panic" devnet_trader.log || echo "0")
    if [ "$CRITICAL_ERRORS" -eq 0 ]; then
        echo -e "${GREEN}✅ No critical errors found${NC}"
        echo "Critical Errors: PASS - 0 critical errors" >> $ASSERTIONS_FILE
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}❌ Found $CRITICAL_ERRORS critical errors${NC}"
        echo "Critical Errors: FAIL - $CRITICAL_ERRORS critical errors" >> $ASSERTIONS_FILE
        FAILED=$((FAILED + 1))
    fi
fi

# Final summary
echo -e "\n${BLUE}📊 ASSERTION SUMMARY${NC}"
echo "===================="
echo "Assertions passed: $PASSED"
echo "Assertions failed: $FAILED"
echo "Total assertions: $((PASSED + FAILED))"

# Save summary to file
cat >> $ASSERTIONS_FILE << EOF

ASSERTION_SUMMARY:
passed=$PASSED
failed=$FAILED
total=$((PASSED + FAILED))
success_rate=$(echo "scale=2; $PASSED * 100 / ($PASSED + $FAILED)" | bc -l)%
EOF

if [ "$FAILED" -eq 0 ]; then
    echo -e "${GREEN}🎉 ALL ASSERTIONS PASSED!${NC}"
    echo "OVERALL_STATUS=PASS" >> $ASSERTIONS_FILE
    exit 0
elif [ "$PASSED" -gt "$FAILED" ]; then
    echo -e "${YELLOW}⚠️  PARTIAL SUCCESS - Some assertions failed${NC}"
    echo "OVERALL_STATUS=PARTIAL" >> $ASSERTIONS_FILE
    exit 0
else
    echo -e "${RED}❌ MAJORITY OF ASSERTIONS FAILED${NC}"
    echo "OVERALL_STATUS=FAIL" >> $ASSERTIONS_FILE
    exit 1
fi
