#!/bin/bash

set -e

echo "🚀 Starting Solana HFT Ninja 2025.07..."

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

# Make setup script executable
chmod +x scripts/setup.sh

# Run setup if config doesn't exist
if [ ! -f config/config.toml ]; then
    echo "🔧 Running setup..."
    ./scripts/setup.sh
fi

# Build and start services
echo "🏗️  Building and starting services..."
docker-compose up -d --build

echo "✅ Services started successfully!"
echo ""
echo "📊 Access the monitoring dashboard:"
echo "   - Grafana: http://localhost:3000 (admin/admin)"
echo "   - Prometheus: http://localhost:9090"
echo "   - HFT Metrics: http://localhost:8080/metrics"
echo ""
echo "📋 View logs:"
echo "   docker-compose logs -f hft-ninja"
echo ""
echo "🛑 Stop services:"
echo "   docker-compose down"