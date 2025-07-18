#!/bin/bash

# 🛑 HFT Ninja Development Stack Stopper
# Stops all development services gracefully

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🛑 Stopping Solana HFT Ninja Development Stack${NC}"
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

# Function to stop service by PID file
stop_service() {
    local service_name=$1
    local pid_file="/tmp/hft-ninja-$service_name.pid"
    
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        
        if kill -0 "$pid" 2>/dev/null; then
            print_status "Stopping $service_name (PID: $pid)..."
            
            # Try graceful shutdown first
            kill -TERM "$pid" 2>/dev/null || true
            
            # Wait up to 10 seconds for graceful shutdown
            local count=0
            while kill -0 "$pid" 2>/dev/null && [ $count -lt 10 ]; do
                sleep 1
                count=$((count + 1))
            done
            
            # Force kill if still running
            if kill -0 "$pid" 2>/dev/null; then
                print_warning "Force killing $service_name..."
                kill -KILL "$pid" 2>/dev/null || true
            fi
            
            print_status "✅ $service_name stopped"
        else
            print_status "$service_name was not running"
        fi
        
        rm -f "$pid_file"
    else
        print_status "$service_name PID file not found (may not be running)"
    fi
}

# Function to stop services by port
stop_by_port() {
    local port=$1
    local service_name=$2
    
    local pid=$(lsof -ti:$port 2>/dev/null || true)
    
    if [ -n "$pid" ]; then
        print_status "Stopping $service_name on port $port (PID: $pid)..."
        kill -TERM "$pid" 2>/dev/null || true
        
        # Wait for graceful shutdown
        sleep 2
        
        # Force kill if still running
        if kill -0 "$pid" 2>/dev/null; then
            kill -KILL "$pid" 2>/dev/null || true
        fi
        
        print_status "✅ $service_name stopped"
    else
        print_status "$service_name not running on port $port"
    fi
}

# Function to cleanup temporary files
cleanup_temp_files() {
    print_status "Cleaning up temporary files..."
    
    # Remove PID files
    rm -f /tmp/hft-ninja-*.pid
    
    # Remove log files
    rm -f /tmp/hft-ninja-*.log
    
    # Remove development environment file
    rm -f .env.development
    
    print_status "✅ Temporary files cleaned"
}

# Function to stop all services
stop_all_services() {
    print_status "Stopping all development services..."
    
    # Stop services by PID files first
    stop_service "AI-API"
    stop_service "BFF"
    stop_service "Frontend"
    
    # Stop any remaining services by port (fallback)
    stop_by_port 8003 "AI-API"
    stop_by_port 8002 "BFF"
    stop_by_port 3000 "Frontend"
    
    # Stop any Python processes that might be related
    print_status "Stopping any remaining Python processes..."
    pkill -f "deepseek_api_mock.py" 2>/dev/null || true
    pkill -f "main_simple.py" 2>/dev/null || true
    pkill -f "http.server 3000" 2>/dev/null || true
    
    print_status "✅ All services stopped"
}

# Function to verify all services are stopped
verify_stopped() {
    print_status "Verifying all services are stopped..."
    
    local ports=(8003 8002 3000)
    local all_stopped=true
    
    for port in "${ports[@]}"; do
        if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
            print_warning "Port $port is still in use"
            all_stopped=false
        fi
    done
    
    if [ "$all_stopped" = true ]; then
        print_status "✅ All ports are free"
    else
        print_warning "Some ports are still in use"
        print_status "You may need to manually kill remaining processes"
    fi
}

# Function to display final status
display_final_status() {
    echo ""
    echo -e "${GREEN}🎉 HFT Ninja Development Stack Stopped!${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
    echo -e "${YELLOW}📊 Port Status:${NC}"
    
    local ports=(8003 8002 3000)
    local services=("AI-API" "BFF" "Frontend")
    
    for i in "${!ports[@]}"; do
        local port=${ports[$i]}
        local service=${services[$i]}
        
        if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
            echo -e "  • Port $port ($service): ${RED}OCCUPIED${NC}"
        else
            echo -e "  • Port $port ($service): ${GREEN}FREE${NC}"
        fi
    done
    
    echo ""
    echo -e "${YELLOW}🔧 Next Steps:${NC}"
    echo -e "  • Start development:    ${GREEN}make dev${NC}"
    echo -e "  • Start with logs:      ${GREEN}make dev-verbose${NC}"
    echo -e "  • Start production:     ${GREEN}make prod${NC}"
    echo -e "  • Run tests:            ${GREEN}make test${NC}"
    echo ""
}

# Main execution
main() {
    # Stop all services
    stop_all_services
    
    # Cleanup temporary files
    cleanup_temp_files
    
    # Verify all services are stopped
    verify_stopped
    
    # Display final status
    display_final_status
}

# Run main function
main "$@"
