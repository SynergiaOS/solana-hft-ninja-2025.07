#!/bin/bash
# Setup script for DragonflyDB Cloud integration

echo "🐉 Setting up DragonflyDB Cloud for Project Cerebro..."

# Check if API key is set
if [ -z "$DRAGONFLY_API_KEY" ]; then
    echo "❌ DRAGONFLY_API_KEY not set. Please set it in .env file"
    exit 1
fi

echo "✅ DragonflyDB API Key found"

# Test connection to DragonflyDB Cloud (you'll need to update the URL)
echo "🔍 Testing DragonflyDB Cloud connection..."

# You'll need to replace this with your actual DragonflyDB Cloud instance URL
# Example: redis://default:password@your-instance.dragonflydb.cloud:6379

echo "📝 Next steps:"
echo "1. Create a DragonflyDB Cloud instance at https://dragonflydb.cloud"
echo "2. Update DRAGONFLY_URL in .env with your instance connection string"
echo "3. Update DRAGONFLY_PASSWORD in .env with your instance password"
echo "4. Run: docker-compose -f docker-compose-cerebro.yml up -d"

echo ""
echo "🔧 Example .env configuration:"
echo "DRAGONFLY_URL=redis://default:your-password@your-instance.dragonflydb.cloud:6379"
echo "DRAGONFLY_PASSWORD=your-password"

echo ""
echo "🧪 To test the connection:"
echo "python tests/test_dragonflydb.py"
