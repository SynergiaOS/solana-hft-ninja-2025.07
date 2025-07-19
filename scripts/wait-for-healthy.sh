#!/bin/bash
# 🔍 Wait for service to become healthy

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

if [ $# -lt 2 ]; then
    echo "Usage: $0 <health_url> <timeout_seconds>"
    echo "Example: $0 localhost:8080/health 30"
    exit 1
fi

HEALTH_URL="$1"
TIMEOUT="$2"
INTERVAL=2

echo -e "${BLUE}🔍 Waiting for service to become healthy...${NC}"
echo "  URL: $HEALTH_URL"
echo "  Timeout: ${TIMEOUT}s"
echo "  Check interval: ${INTERVAL}s"

START_TIME=$(date +%s)
ATTEMPT=1

while true; do
    CURRENT_TIME=$(date +%s)
    ELAPSED=$((CURRENT_TIME - START_TIME))
    
    if [ $ELAPSED -ge $TIMEOUT ]; then
        echo -e "${RED}❌ Timeout reached (${TIMEOUT}s)${NC}"
        exit 1
    fi
    
    echo -n "  Attempt $ATTEMPT (${ELAPSED}s elapsed)... "
    
    if curl -s -f "http://$HEALTH_URL" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Service is healthy!${NC}"
        exit 0
    else
        echo -e "${YELLOW}⏳ Not ready yet${NC}"
    fi
    
    sleep $INTERVAL
    ATTEMPT=$((ATTEMPT + 1))
done
