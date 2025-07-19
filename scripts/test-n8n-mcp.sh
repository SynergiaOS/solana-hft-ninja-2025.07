#!/bin/bash

# 🧪 Test n8n + MCP Integration
# Comprehensive testing of workflow automation and MCP protocol

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${BLUE}🧪 Testing n8n + MCP Integration${NC}"
echo "=================================="

# Function to check if service is running
check_service() {
    local service_name=$1
    local url=$2
    
    echo -e "${YELLOW}🔍 Checking $service_name...${NC}"
    
    if curl -s -f "$url" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ $service_name is running${NC}"
        return 0
    else
        echo -e "${RED}❌ $service_name is not running${NC}"
        return 1
    fi
}

# Function to wait for service
wait_for_service() {
    local service_name=$1
    local url=$2
    local max_attempts=30
    local attempt=1

    echo -e "${YELLOW}⏳ Waiting for $service_name to be ready...${NC}"
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s -f "$url" > /dev/null 2>&1; then
            echo -e "${GREEN}✅ $service_name is ready!${NC}"
            return 0
        fi
        
        echo -e "${YELLOW}   Attempt $attempt/$max_attempts...${NC}"
        sleep 2
        ((attempt++))
    done
    
    echo -e "${RED}❌ $service_name failed to start within expected time${NC}"
    return 1
}

# Check prerequisites
echo -e "${BLUE}📋 Checking Prerequisites${NC}"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}❌ Docker is not running. Please start Docker first.${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Docker is running${NC}"

# Check if Python is available
if ! command -v python3 > /dev/null 2>&1; then
    echo -e "${RED}❌ Python 3 is not installed${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Python 3 is available${NC}"

# Check if required Python packages are installed
echo -e "${YELLOW}📦 Checking Python dependencies...${NC}"
cd "$PROJECT_ROOT"

# Install test dependencies if needed
if ! python3 -c "import httpx, pytest" 2>/dev/null; then
    echo -e "${YELLOW}📦 Installing test dependencies...${NC}"
    pip3 install httpx pytest asyncio-pytest || {
        echo -e "${RED}❌ Failed to install test dependencies${NC}"
        exit 1
    }
fi
echo -e "${GREEN}✅ Python dependencies are available${NC}"

# Start services if not running
echo -e "${BLUE}🚀 Starting Required Services${NC}"

# Check if services are already running
services_running=true

if ! check_service "Cerebro BFF" "http://localhost:8000/health"; then
    services_running=false
fi

if ! check_service "n8n" "http://localhost:5678/healthz"; then
    services_running=false
fi

if [ "$services_running" = false ]; then
    echo -e "${YELLOW}🚀 Starting services...${NC}"
    
    # Start the full stack
    docker-compose up -d
    
    # Wait for services to be ready
    wait_for_service "Cerebro BFF" "http://localhost:8000/health"
    wait_for_service "n8n" "http://localhost:5678/healthz"
    
    # Give services extra time to fully initialize
    echo -e "${YELLOW}⏳ Waiting for services to fully initialize...${NC}"
    sleep 15
else
    echo -e "${GREEN}✅ All required services are already running${NC}"
fi

# Run the integration tests
echo -e "${BLUE}🧪 Running Integration Tests${NC}"
echo "=============================="

cd "$PROJECT_ROOT"

# Run the Python integration tests
echo -e "${YELLOW}🐍 Running Python integration tests...${NC}"
python3 tests/test_n8n_mcp_integration.py

test_exit_code=$?

# Additional manual tests
echo -e "${BLUE}🔧 Running Manual API Tests${NC}"
echo "============================="

# Test 1: MCP Servers endpoint
echo -e "${YELLOW}🔍 Test 1: MCP Servers Discovery${NC}"
if curl -s "http://localhost:8000/api/mcp/servers" | jq . > /dev/null 2>&1; then
    echo -e "${GREEN}✅ MCP servers endpoint working${NC}"
    curl -s "http://localhost:8000/api/mcp/servers" | jq -r '.servers[]' | while read server; do
        echo -e "${BLUE}   📡 Server: $server${NC}"
    done
else
    echo -e "${YELLOW}⚠️  MCP servers endpoint not ready (install jq for better output)${NC}"
fi

# Test 2: n8n API access
echo -e "${YELLOW}🔍 Test 2: n8n API Access${NC}"
if curl -s -u "admin:cerebro123" "http://localhost:5678/api/v1/workflows" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ n8n API accessible${NC}"
else
    echo -e "${YELLOW}⚠️  n8n API not accessible (may require authentication setup)${NC}"
fi

# Test 3: Workflow files validation
echo -e "${YELLOW}🔍 Test 3: Workflow Files Validation${NC}"
workflow_files=(
    "n8n/workflows/cerebro_status_monitor.json"
    "n8n/workflows/external_data_ingestion.json"
    "n8n/mcp/cerebro_mcp_server.json"
)

all_workflows_valid=true
for workflow_file in "${workflow_files[@]}"; do
    if [ -f "$workflow_file" ]; then
        if jq . "$workflow_file" > /dev/null 2>&1; then
            echo -e "${GREEN}✅ Valid: $workflow_file${NC}"
        else
            echo -e "${RED}❌ Invalid JSON: $workflow_file${NC}"
            all_workflows_valid=false
        fi
    else
        echo -e "${RED}❌ Missing: $workflow_file${NC}"
        all_workflows_valid=false
    fi
done

if [ "$all_workflows_valid" = true ]; then
    echo -e "${GREEN}✅ All workflow files are valid${NC}"
else
    echo -e "${RED}❌ Some workflow files are invalid or missing${NC}"
fi

# Test 4: Docker containers health
echo -e "${YELLOW}🔍 Test 4: Docker Containers Health${NC}"
containers=("n8n-automation" "solana-hft-ninja")

all_containers_healthy=true
for container in "${containers[@]}"; do
    if docker ps --filter "name=$container" --filter "status=running" | grep -q "$container"; then
        echo -e "${GREEN}✅ Container running: $container${NC}"
    else
        echo -e "${RED}❌ Container not running: $container${NC}"
        all_containers_healthy=false
    fi
done

# Display final results
echo ""
echo -e "${BLUE}📊 Test Summary${NC}"
echo "==============="

if [ $test_exit_code -eq 0 ] && [ "$all_workflows_valid" = true ] && [ "$all_containers_healthy" = true ]; then
    echo -e "${GREEN}🎉 All tests passed successfully!${NC}"
    echo ""
    echo -e "${BLUE}🔗 Access Information:${NC}"
    echo -e "   🌐 n8n Web UI:     ${GREEN}http://localhost:5678${NC} (admin/cerebro123)"
    echo -e "   📊 Cerebro BFF:    ${GREEN}http://localhost:8000${NC}"
    echo -e "   🔗 MCP Endpoints:  ${GREEN}http://localhost:8000/api/mcp/*${NC}"
    echo ""
    echo -e "${BLUE}🚀 Next Steps:${NC}"
    echo "   1. Open n8n Web UI and activate workflows"
    echo "   2. Configure external API credentials"
    echo "   3. Test MCP integration with Claude/Cursor"
    echo ""
    exit 0
else
    echo -e "${RED}❌ Some tests failed${NC}"
    echo ""
    echo -e "${YELLOW}🔧 Troubleshooting:${NC}"
    echo "   1. Check Docker containers: docker-compose ps"
    echo "   2. Check logs: docker-compose logs"
    echo "   3. Restart services: docker-compose restart"
    echo ""
    exit 1
fi
