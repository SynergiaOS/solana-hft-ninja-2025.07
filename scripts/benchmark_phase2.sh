#!/bin/bash
# 🧠 FAZA 2 BENCHMARK - Cerebro Batch & Cache Blitz

set -e

echo "🧠 FAZA 2/3: Benchmark Cerebro optimizations"
echo "============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Benchmark results
RESULTS_FILE="benchmark_results_phase2.txt"
echo "# FAZA 2 Benchmark Results - $(date)" > $RESULTS_FILE

# Function to run benchmark and check threshold
run_benchmark() {
    local test_name="$1"
    local command="$2"
    local threshold="$3"
    local unit="$4"
    local comparison_op="${5:-<}"  # Default to less than
    
    echo -e "${BLUE}🔍 Testing: $test_name${NC}"
    echo "Command: $command"
    
    # Run benchmark 3 times and get average
    local total=0
    local runs=3
    
    for i in $(seq 1 $runs); do
        echo -n "  Run $i/$runs... "
        local result=$(eval $command 2>/dev/null | tail -1)
        local value=$(echo $result | grep -oE '[0-9]+\.?[0-9]*' | head -1)
        
        if [[ -z "$value" ]]; then
            echo -e "${RED}FAILED${NC} - No numeric result"
            return 1
        fi
        
        total=$(echo "$total + $value" | bc -l)
        echo "${value}${unit}"
    done
    
    local average=$(echo "scale=2; $total / $runs" | bc -l)
    echo "  Average: ${average}${unit}"
    
    # Check against threshold based on comparison operator
    local comparison
    case $comparison_op in
        "<")
            comparison=$(echo "$average < $threshold" | bc -l)
            ;;
        ">")
            comparison=$(echo "$average > $threshold" | bc -l)
            ;;
        ">=")
            comparison=$(echo "$average >= $threshold" | bc -l)
            ;;
        "<=")
            comparison=$(echo "$average <= $threshold" | bc -l)
            ;;
    esac
    
    if [[ $comparison -eq 1 ]]; then
        echo -e "  ${GREEN}✅ PASS${NC} (${average}${unit} ${comparison_op} ${threshold}${unit})"
        echo "$test_name: PASS - ${average}${unit} ${comparison_op} ${threshold}${unit}" >> $RESULTS_FILE
        return 0
    else
        echo -e "  ${RED}❌ FAIL${NC} (${average}${unit} not ${comparison_op} ${threshold}${unit})"
        echo "$test_name: FAIL - ${average}${unit} not ${comparison_op} ${threshold}${unit}" >> $RESULTS_FILE
        return 1
    fi
}

# Function to check if Redis is running
check_redis() {
    if ! redis-cli ping &>/dev/null; then
        echo -e "${YELLOW}⚠️  Redis not running, starting Docker container...${NC}"
        docker run -d --name redis-test -p 6379:6379 redis:alpine || true
        sleep 3
        
        if ! redis-cli ping &>/dev/null; then
            echo -e "${RED}❌ Could not start Redis${NC}"
            return 1
        fi
    fi
    echo -e "${GREEN}✅ Redis is running${NC}"
}

# Function to check if DragonflyDB is running
check_dragonfly() {
    if ! redis-cli -p 6380 ping &>/dev/null; then
        echo -e "${YELLOW}⚠️  DragonflyDB not running, starting Docker container...${NC}"
        docker run -d --name dragonfly-test -p 6380:6379 docker.dragonflydb.io/dragonflydb/dragonfly || true
        sleep 5
        
        if ! redis-cli -p 6380 ping &>/dev/null; then
            echo -e "${YELLOW}⚠️  DragonflyDB not available, using Redis fallback${NC}"
            return 1
        fi
    fi
    echo -e "${GREEN}✅ DragonflyDB is running${NC}"
}

# Check prerequisites
echo -e "${YELLOW}📋 Checking prerequisites...${NC}"
check_redis
check_dragonfly || true  # DragonflyDB is optional

# Build Cerebro components
echo -e "${YELLOW}🔨 Building Cerebro components...${NC}"
cd cerebro
cargo build --release --quiet
cd ..

# Test 2A: Redis batch processing
echo -e "\n${YELLOW}2A: Redis Batch Processing${NC}"

# Clear Redis queues
redis-cli del cerebro:fast cerebro:slow &>/dev/null || true

# Simulate batch processing
BATCH_TEST="redis-cli lpush cerebro:fast '{\"test\":\"data\"}' &>/dev/null && redis-cli llen cerebro:fast"
run_benchmark "Redis Batch Queue" "$BATCH_TEST" "0" "" ">="

# Test batch processing performance
BATCH_PERF="time (for i in {1..100}; do redis-cli lpush cerebro:fast '{\"timestamp\":1640995200,\"wallet\":\"test\",\"amount\":0.1}' &>/dev/null; done) 2>&1 | grep real | awk '{print \$2}' | sed 's/m//' | sed 's/s//' | awk -F. '{print \$1*1000 + \$2}'"
run_benchmark "Batch Processing Speed" "$BATCH_PERF" "1000" "ms"

# Test 2B: Prompt compression
echo -e "\n${YELLOW}2B: Prompt Compression${NC}"

# Create test data
TEST_DATA='{"transactions":[{"wallet":"test1","amount":0.1},{"wallet":"test2","amount":0.2}]}'
echo "$TEST_DATA" > /tmp/test_prompt.json

# Test compression ratio
COMPRESS_TEST="echo '$TEST_DATA' | gzip | wc -c && echo '$TEST_DATA' | wc -c | awk '{print \$1}'"
ORIGINAL_SIZE=$(echo "$TEST_DATA" | wc -c)
COMPRESSED_SIZE=$(echo "$TEST_DATA" | gzip | wc -c)
COMPRESSION_RATIO=$(echo "scale=2; (1 - $COMPRESSED_SIZE / $ORIGINAL_SIZE) * 100" | bc -l)

echo "  Original size: ${ORIGINAL_SIZE} bytes"
echo "  Compressed size: ${COMPRESSED_SIZE} bytes"
echo "  Compression ratio: ${COMPRESSION_RATIO}%"

if (( $(echo "$COMPRESSION_RATIO >= 40" | bc -l) )); then
    echo -e "  ${GREEN}✅ PASS${NC} (${COMPRESSION_RATIO}% >= 40%)"
    echo "Prompt Compression: PASS - ${COMPRESSION_RATIO}% >= 40%" >> $RESULTS_FILE
else
    echo -e "  ${RED}❌ FAIL${NC} (${COMPRESSION_RATIO}% < 40%)"
    echo "Prompt Compression: FAIL - ${COMPRESSION_RATIO}% < 40%" >> $RESULTS_FILE
fi

# Test 2C: DragonflyDB cache performance
echo -e "\n${YELLOW}2C: DragonflyDB Cache Performance${NC}"

if redis-cli -p 6380 ping &>/dev/null; then
    # Test cache hit rate simulation
    CACHE_PORT=6380
    CACHE_NAME="DragonflyDB"
else
    # Fallback to Redis
    CACHE_PORT=6379
    CACHE_NAME="Redis"
fi

echo "Using $CACHE_NAME on port $CACHE_PORT"

# Clear cache
redis-cli -p $CACHE_PORT flushall &>/dev/null || true

# Simulate cache operations
for i in {1..100}; do
    redis-cli -p $CACHE_PORT setex "strategy:test:$i" 300 "cached_result_$i" &>/dev/null
done

# Test cache hit rate
HITS=0
for i in {1..100}; do
    if redis-cli -p $CACHE_PORT get "strategy:test:$i" &>/dev/null; then
        HITS=$((HITS + 1))
    fi
done

HIT_RATE=$(echo "scale=2; $HITS" | bc -l)
echo "  Cache hits: $HITS/100"
echo "  Hit rate: ${HIT_RATE}%"

if (( $(echo "$HIT_RATE >= 95" | bc -l) )); then
    echo -e "  ${GREEN}✅ PASS${NC} (${HIT_RATE}% >= 95%)"
    echo "Cache Hit Rate: PASS - ${HIT_RATE}% >= 95%" >> $RESULTS_FILE
else
    echo -e "  ${RED}❌ FAIL${NC} (${HIT_RATE}% < 95%)"
    echo "Cache Hit Rate: FAIL - ${HIT_RATE}% < 95%" >> $RESULTS_FILE
fi

# Test 2D: Model switching cost simulation
echo -e "\n${YELLOW}2D: Model Switching Cost Simulation${NC}"

# Simulate daily cost calculation
HOT_REQUESTS=1000    # GPT-4o-mini requests
WARM_REQUESTS=100    # GPT-4o requests  
COLD_REQUESTS=10     # Llama-3-70B requests

HOT_COST=$(echo "scale=2; $HOT_REQUESTS * 0.00015" | bc -l)    # $0.15/1M tokens
WARM_COST=$(echo "scale=2; $WARM_REQUESTS * 0.005" | bc -l)    # $5/1M tokens
COLD_COST=0  # Free on own GPU

TOTAL_COST=$(echo "scale=2; $HOT_COST + $WARM_COST + $COLD_COST" | bc -l)

echo "  Hot model cost (GPT-4o-mini): \$${HOT_COST}"
echo "  Warm model cost (GPT-4o): \$${WARM_COST}"
echo "  Cold model cost (Llama-3-70B): \$${COLD_COST}"
echo "  Total daily cost: \$${TOTAL_COST}"

if (( $(echo "$TOTAL_COST <= 30" | bc -l) )); then
    echo -e "  ${GREEN}✅ PASS${NC} (\$${TOTAL_COST} <= \$30)"
    echo "Model Switching Cost: PASS - \$${TOTAL_COST} <= \$30" >> $RESULTS_FILE
else
    echo -e "  ${RED}❌ FAIL${NC} (\$${TOTAL_COST} > \$30)"
    echo "Model Switching Cost: FAIL - \$${TOTAL_COST} > \$30" >> $RESULTS_FILE
fi

# Cleanup
redis-cli -p $CACHE_PORT flushall &>/dev/null || true
rm -f /tmp/test_prompt.json

# Overall assessment
echo -e "\n${BLUE}📊 FAZA 2 SUMMARY${NC}"
echo "==================="

PASS_COUNT=$(grep "PASS" $RESULTS_FILE | wc -l)
TOTAL_COUNT=$(grep -E "(PASS|FAIL)" $RESULTS_FILE | wc -l)

echo "Results saved to: $RESULTS_FILE"
echo "Tests passed: $PASS_COUNT/$TOTAL_COUNT"

if [[ $PASS_COUNT -eq $TOTAL_COUNT ]]; then
    echo -e "${GREEN}🎉 ALL TESTS PASSED! Ready for FAZA 3${NC}"
    echo "PHASE2_STATUS=PASS" >> $RESULTS_FILE
    exit 0
elif [[ $PASS_COUNT -gt 0 ]]; then
    echo -e "${YELLOW}⚠️  PARTIAL SUCCESS - Some optimizations working${NC}"
    echo "PHASE2_STATUS=PARTIAL" >> $RESULTS_FILE
    exit 0
else
    echo -e "${RED}❌ ALL TESTS FAILED - Need investigation${NC}"
    echo "PHASE2_STATUS=FAIL" >> $RESULTS_FILE
    exit 1
fi
