#!/bin/bash

# 🚀 HFT Ninja Development Stack Starter
# Starts all development services in the correct order

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
VERBOSE=false
if [ "$1" = "--verbose" ]; then
    VERBOSE=true
fi

echo -e "${BLUE}🥷 Starting Solana HFT Ninja Development Stack${NC}"
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

# Function to check if port is available
check_port() {
    local port=$1
    local service=$2
    
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        print_warning "Port $port is already in use (may be $service already running)"
        return 1
    fi
    return 0
}

# Function to wait for service to be ready
wait_for_service() {
    local url=$1
    local service_name=$2
    local max_attempts=30
    local attempt=1
    
    print_status "Waiting for $service_name to be ready..."
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s -f "$url" > /dev/null 2>&1; then
            print_status "✅ $service_name is ready!"
            return 0
        fi
        
        echo -n "."
        sleep 2
        attempt=$((attempt + 1))
    done
    
    print_error "❌ $service_name failed to start within $((max_attempts * 2)) seconds"
    return 1
}

# Function to start service in background
start_service() {
    local service_name=$1
    local command=$2
    local port=$3
    local health_url=$4
    
    print_status "Starting $service_name..."
    
    # Check if port is available
    if ! check_port $port "$service_name"; then
        print_status "$service_name may already be running on port $port"
        if curl -s -f "$health_url" > /dev/null 2>&1; then
            print_status "✅ $service_name is already running and healthy"
            return 0
        fi
    fi
    
    # Start service
    if [ "$VERBOSE" = true ]; then
        echo -e "${BLUE}Command: $command${NC}"
        eval "$command" &
    else
        eval "$command" > /dev/null 2>&1 &
    fi
    
    local pid=$!
    echo $pid > "/tmp/hft-ninja-$service_name.pid"
    
    # Wait for service to be ready
    if wait_for_service "$health_url" "$service_name"; then
        print_status "✅ $service_name started successfully (PID: $pid)"
        return 0
    else
        print_error "❌ Failed to start $service_name"
        kill $pid 2>/dev/null || true
        rm -f "/tmp/hft-ninja-$service_name.pid"
        return 1
    fi
}

# Function to create development environment file
create_dev_env() {
    print_status "Creating development environment configuration..."
    
    cat > .env.development << EOF
# Development Environment Configuration
NODE_ENV=development
ENVIRONMENT=development

# API URLs
AI_API_URL=http://localhost:8003
BFF_API_URL=http://localhost:8002
FRONTEND_URL=http://localhost:3000

# Solana Configuration (Devnet)
SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_WS_URL=wss://api.devnet.solana.com

# DragonflyDB Configuration
DRAGONFLY_URL=redis://localhost:6379

# Development Settings
LOG_LEVEL=DEBUG
ENABLE_CORS=true
ENABLE_DEBUG=true

# AI Configuration
MODEL_NAME=deepseek-ai/deepseek-math-7b-instruct
USE_QUANTIZATION=true
MAX_DAILY_AI_COST=1.0
PREFER_CACHE=true

# Trading Configuration
MAX_POSITION_SIZE=2.0
ENABLE_PAPER_TRADING=true
RISK_LEVEL=low
EOF

    print_status "✅ Development environment configured"
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    # Check if Python virtual environment exists
    if [ ! -d "cerebro/venv" ]; then
        print_status "Creating Python virtual environment..."
        cd cerebro
        python3 -m venv venv
        source venv/bin/activate
        pip install -r requirements.txt
        cd ..
    fi
    
    # Check if Node.js dependencies are installed
    if [ ! -d "hft-ninja-frontend/node_modules" ]; then
        print_status "Installing Node.js dependencies..."
        cd hft-ninja-frontend
        npm install
        cd ..
    fi
    
    # Check if frontend is built
    if [ ! -d "hft-ninja-frontend/build" ]; then
        print_status "Building React frontend..."
        cd hft-ninja-frontend
        npm run build
        cd ..
    fi
    
    print_status "✅ Prerequisites checked"
}

# Function to start all services
start_all_services() {
    print_status "Starting all development services..."
    
    # Start AI API (DeepSeek-Math Mock)
    start_service "AI-API" \
        "cd cerebro && source venv/bin/activate && python ai/deepseek_api_mock.py" \
        8003 \
        "http://localhost:8003/health"
    
    # Start BFF (Backend for Frontend)
    start_service "BFF" \
        "cd cerebro && source venv/bin/activate && cd bff && python main_simple.py" \
        8002 \
        "http://localhost:8002/health"
    
    # Start Frontend (React App)
    start_service "Frontend" \
        "cd hft-ninja-frontend && python3 -m http.server 3000 --directory build" \
        3000 \
        "http://localhost:3000"
    
    print_status "✅ All services started successfully!"
}

# Function to display service information
display_service_info() {
    echo ""
    echo -e "${GREEN}🎉 HFT Ninja Development Stack is Ready!${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
    echo -e "${YELLOW}📊 Service Endpoints:${NC}"
    echo -e "  • Frontend (React):     ${GREEN}http://localhost:3000${NC}"
    echo -e "  • BFF API:              ${GREEN}http://localhost:8002${NC}"
    echo -e "  • AI API:               ${GREEN}http://localhost:8003${NC}"
    echo ""
    echo -e "${YELLOW}🔧 Development Commands:${NC}"
    echo -e "  • View all logs:        ${GREEN}make logs${NC}"
    echo -e "  • Check status:         ${GREEN}make status${NC}"
    echo -e "  • Run tests:            ${GREEN}make test${NC}"
    echo -e "  • Stop services:        ${GREEN}make down${NC}"
    echo ""
    echo -e "${YELLOW}🧪 Quick Tests:${NC}"
    echo -e "  • AI Health:            ${GREEN}curl http://localhost:8003/health${NC}"
    echo -e "  • BFF Health:           ${GREEN}curl http://localhost:8002/health${NC}"
    echo -e "  • Trading Signals:      ${GREEN}curl http://localhost:8002/api/trading/signals${NC}"
    echo ""
    echo -e "${YELLOW}📈 Trading Panel:${NC}"
    echo -e "  • Open browser:         ${GREEN}http://localhost:3000${NC}"
    echo -e "  • Navigate to:          ${GREEN}Trading tab${NC}"
    echo -e "  • Test buy/sell/hold:   ${GREEN}Use the trading panel${NC}"
    echo ""
    
    if [ "$VERBOSE" = true ]; then
        echo -e "${YELLOW}📜 Live Logs (Ctrl+C to exit):${NC}"
        echo ""
        # Show live logs if verbose mode
        tail -f /tmp/hft-ninja-*.log 2>/dev/null || true
    else
        echo -e "${GREEN}💡 Tip: Use 'make dev-verbose' to see live logs${NC}"
        echo -e "${GREEN}💡 Tip: Use 'make logs' to follow logs after startup${NC}"
    fi
}

# Main execution
main() {
    # Create development environment
    create_dev_env
    
    # Check prerequisites
    check_prerequisites
    
    # Start all services
    start_all_services
    
    # Display information
    display_service_info
    
    # If verbose mode, keep script running to show logs
    if [ "$VERBOSE" = true ]; then
        echo ""
        echo -e "${BLUE}Press Ctrl+C to stop all services and exit${NC}"
        
        # Trap Ctrl+C to cleanup
        trap 'echo -e "\n${YELLOW}Stopping all services...${NC}"; ./scripts/stop-dev-stack.sh; exit 0' INT
        
        # Wait indefinitely
        while true; do
            sleep 1
        done
    fi
}

# Run main function
main "$@"
