#!/bin/bash

set -e

echo "💰 Checking Solana HFT Ninja Balance..."

# Check if Docker is running
if ! docker-compose ps | grep -q "hft-ninja"; then
    echo "❌ HFT Ninja is not running. Please start it first with ./scripts/start.sh"
    exit 1
fi

# Execute balance check
docker-compose exec hft-ninja ./hft-ninja --balance

echo ""
echo "📊 Additional balance info:"
echo "   - Check logs: docker-compose logs hft-ninja | grep balance"
echo "   - Check metrics: curl http://localhost:8080/metrics"