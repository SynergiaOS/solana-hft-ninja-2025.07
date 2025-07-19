#!/bin/bash

# 🔄 n8n + MCP Integration Startup Script for Cerebro
# Starts n8n with pre-configured workflows and MCP server

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
N8N_DATA_DIR="$PROJECT_ROOT/n8n"

echo -e "${BLUE}🔄 Starting n8n + MCP Integration for Cerebro${NC}"
echo "=================================================="

# Function to check if service is running
check_service() {
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
        
        echo -e "${YELLOW}   Attempt $attempt/$max_attempts - $service_name not ready yet...${NC}"
        sleep 2
        ((attempt++))
    done
    
    echo -e "${RED}❌ $service_name failed to start within expected time${NC}"
    return 1
}

# Function to import workflow
import_workflow() {
    local workflow_file=$1
    local workflow_name=$2
    
    echo -e "${YELLOW}📥 Importing workflow: $workflow_name${NC}"
    
    if [ -f "$workflow_file" ]; then
        # Use n8n CLI to import workflow
        docker exec n8n-automation n8n import:workflow --input="$workflow_file" || {
            echo -e "${YELLOW}⚠️  Workflow import failed, will be available for manual import${NC}"
        }
    else
        echo -e "${YELLOW}⚠️  Workflow file not found: $workflow_file${NC}"
    fi
}

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}❌ Docker is not running. Please start Docker first.${NC}"
    exit 1
fi

# Create n8n data directories if they don't exist
echo -e "${BLUE}📁 Setting up n8n directories...${NC}"
mkdir -p "$N8N_DATA_DIR"/{workflows,credentials,nodes,mcp}

# Set proper permissions
chmod -R 755 "$N8N_DATA_DIR"

# Start n8n service
echo -e "${BLUE}🚀 Starting n8n service...${NC}"
cd "$PROJECT_ROOT"

# Check if n8n container is already running
if docker ps | grep -q "n8n-automation"; then
    echo -e "${GREEN}✅ n8n container is already running${NC}"
else
    # Start n8n container
    docker-compose up -d n8n
    
    # Wait for n8n to be ready
    check_service "n8n" "http://localhost:5678/healthz"
fi

# Wait a bit more for n8n to fully initialize
echo -e "${YELLOW}⏳ Waiting for n8n to fully initialize...${NC}"
sleep 10

# Import pre-configured workflows
echo -e "${BLUE}📋 Setting up Cerebro workflows...${NC}"

# Copy workflows to n8n data directory
if [ -d "$PROJECT_ROOT/n8n/workflows" ]; then
    echo -e "${YELLOW}📋 Copying workflow files...${NC}"
    
    # Copy workflow files to the mounted volume
    docker exec n8n-automation mkdir -p /home/node/.n8n/workflows
    
    for workflow in "$PROJECT_ROOT/n8n/workflows"/*.json; do
        if [ -f "$workflow" ]; then
            workflow_name=$(basename "$workflow" .json)
            echo -e "${YELLOW}   Copying: $workflow_name${NC}"
            docker cp "$workflow" n8n-automation:/home/node/.n8n/workflows/
        fi
    done
fi

# Copy MCP configuration
if [ -d "$PROJECT_ROOT/n8n/mcp" ]; then
    echo -e "${YELLOW}🔗 Setting up MCP configuration...${NC}"
    docker exec n8n-automation mkdir -p /home/node/.n8n/mcp
    
    for mcp_file in "$PROJECT_ROOT/n8n/mcp"/*.json; do
        if [ -f "$mcp_file" ]; then
            mcp_name=$(basename "$mcp_file")
            echo -e "${YELLOW}   Copying MCP config: $mcp_name${NC}"
            docker cp "$mcp_file" n8n-automation:/home/node/.n8n/mcp/
        fi
    done
fi

# Display connection information
echo ""
echo -e "${GREEN}🎉 n8n + MCP Integration is ready!${NC}"
echo "=================================================="
echo -e "${BLUE}📊 Access Information:${NC}"
echo -e "   🌐 n8n Web UI:     ${GREEN}http://localhost:5678${NC}"
echo -e "   👤 Username:       ${GREEN}admin${NC}"
echo -e "   🔑 Password:       ${GREEN}cerebro123${NC}"
echo -e "   🔗 MCP Server:     ${GREEN}http://localhost:3001${NC}"
echo ""
echo -e "${BLUE}📋 Available Workflows:${NC}"
echo -e "   🔍 Cerebro Status Monitor    - Monitors system health every 5 minutes"
echo -e "   📊 External Data Ingestion   - Fetches market data hourly"
echo ""
echo -e "${BLUE}🔗 MCP Integration:${NC}"
echo -e "   🤖 Cerebro BFF API:  ${GREEN}http://localhost:8000/api/mcp/*${NC}"
echo -e "   🔍 Available servers: n8n_workflows, brave_search, helius_solana"
echo ""
echo -e "${BLUE}🛠️  Next Steps:${NC}"
echo "   1. Open n8n Web UI and activate the imported workflows"
echo "   2. Configure credentials for external services (Twitter, APIs)"
echo "   3. Test MCP endpoints via Cerebro BFF"
echo "   4. Set up Claude/Cursor with MCP server URL"
echo ""

# Test MCP endpoints
echo -e "${BLUE}🧪 Testing MCP Integration...${NC}"

# Wait for Cerebro BFF to be ready
if check_service "Cerebro BFF" "http://localhost:8000/health"; then
    echo -e "${YELLOW}🔍 Testing MCP servers endpoint...${NC}"
    
    # Test MCP servers endpoint
    if curl -s "http://localhost:8000/api/mcp/servers" | jq . > /dev/null 2>&1; then
        echo -e "${GREEN}✅ MCP servers endpoint is working${NC}"
        
        # Display available MCP tools
        echo -e "${BLUE}🛠️  Available MCP Tools:${NC}"
        curl -s "http://localhost:8000/api/mcp/servers" | jq -r '.tools_by_server | to_entries[] | "   \(.key): \(.value | join(", "))"' 2>/dev/null || echo -e "${YELLOW}   (Install jq for formatted output)${NC}"
    else
        echo -e "${YELLOW}⚠️  MCP endpoints not yet ready (this is normal on first startup)${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Cerebro BFF not ready yet${NC}"
fi

echo ""
echo -e "${GREEN}🚀 n8n + MCP Integration startup complete!${NC}"
echo -e "${BLUE}💡 Pro tip: Use 'docker-compose logs n8n' to monitor n8n logs${NC}"
echo ""

# Optional: Open browser
if command -v xdg-open > /dev/null 2>&1; then
    read -p "Open n8n Web UI in browser? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        xdg-open "http://localhost:5678"
    fi
elif command -v open > /dev/null 2>&1; then
    read -p "Open n8n Web UI in browser? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        open "http://localhost:5678"
    fi
fi
