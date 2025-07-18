#!/bin/bash
# Project Cerebro - Startup Script

set -e

echo "🧠 Starting Project Cerebro..."
echo "================================"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# Check if docker-compose is available
if ! command -v docker-compose &> /dev/null; then
    echo "❌ docker-compose not found. Please install docker-compose."
    exit 1
fi

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    echo "📝 Creating .env file from template..."
    cp .env.example .env
    echo "✅ Please edit .env file with your configuration"
fi

# Pull latest images
echo "📦 Pulling latest Docker images..."
docker-compose pull

# Start services
echo "🚀 Starting Cerebro services..."
docker-compose up -d

# Wait for services to be ready
echo "⏳ Waiting for services to start..."
sleep 10

# Check service health
echo "🔍 Checking service health..."

# Check DragonflyDB
if docker-compose exec -T dragonflydb redis-cli -a cerebro_secure_2025 ping > /dev/null 2>&1; then
    echo "✅ DragonflyDB is healthy"
else
    echo "⚠️  DragonflyDB is not responding"
fi

# Check BFF
if curl -s http://localhost:8000/health > /dev/null 2>&1; then
    echo "✅ BFF API is healthy"
else
    echo "⚠️  BFF API is not responding"
fi

# Check Kestra
if curl -s http://localhost:8081 > /dev/null 2>&1; then
    echo "✅ Kestra is healthy"
else
    echo "⚠️  Kestra is not responding"
fi

echo ""
echo "🎉 Project Cerebro is starting up!"
echo "================================"
echo "📊 Services:"
echo "  - BFF API:      http://localhost:8000"
echo "  - API Docs:     http://localhost:8000/docs"
echo "  - Kestra UI:    http://localhost:8081"
echo "  - Redis Insight: http://localhost:8001"
echo "  - Jina Gateway: http://localhost:8002"
echo ""
echo "📋 Useful commands:"
echo "  - View logs:    docker-compose logs -f"
echo "  - Stop:         docker-compose down"
echo "  - Restart:      docker-compose restart"
echo ""
echo "🧪 Test the system:"
echo "  curl http://localhost:8000/health"
echo ""