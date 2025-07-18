#!/bin/bash

# 📊 HFT Ninja Status Checker
# Comprehensive status check for all services

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}📊 Solana HFT Ninja - System Status${NC}"
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

# Function to check service status
check_service() {
    local service_name=$1
    local port=$2
    local health_url=$3
    local expected_response=$4
    
    # Check if port is listening
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        local pid=$(lsof -ti:$port)
        
        # Check health endpoint
        if curl -s -f "$health_url" > /dev/null 2>&1; then
            local response=$(curl -s "$health_url" | jq -r '.status' 2>/dev/null || echo "unknown")
            
            if [ "$response" = "$expected_response" ] || [ "$expected_response" = "any" ]; then
                echo -e "  • $service_name: ${GREEN}✅ HEALTHY${NC} (PID: $pid, Port: $port)"
                return 0
            else
                echo -e "  • $service_name: ${YELLOW}⚠️  UNHEALTHY${NC} (PID: $pid, Port: $port, Response: $response)"
                return 1
            fi
        else
            echo -e "  • $service_name: ${RED}❌ NOT RESPONDING${NC} (PID: $pid, Port: $port)"
            return 1
        fi
    else
        echo -e "  • $service_name: ${RED}❌ NOT RUNNING${NC} (Port: $port)"
        return 1
    fi
}

# Function to check Docker services
check_docker_services() {
    echo -e "${YELLOW}🐳 Docker Services:${NC}"
    
    if command -v docker-compose &> /dev/null; then
        if [ -f "docker-compose.traefik.yml" ]; then
            local running_services=$(docker-compose -f docker-compose.traefik.yml ps --services --filter status=running 2>/dev/null || echo "")
            
            if [ -n "$running_services" ]; then
                echo "$running_services" | while read -r service; do
                    local status=$(docker-compose -f docker-compose.traefik.yml ps "$service" --format "table {{.State}}" | tail -n +2)
                    if [ "$status" = "Up" ]; then
                        echo -e "  • $service: ${GREEN}✅ RUNNING${NC}"
                    else
                        echo -e "  • $service: ${RED}❌ $status${NC}"
                    fi
                done
            else
                echo -e "  • ${YELLOW}No Docker services running${NC}"
            fi
        else
            echo -e "  • ${YELLOW}Docker Compose file not found${NC}"
        fi
    else
        echo -e "  • ${YELLOW}Docker Compose not available${NC}"
    fi
}

# Function to check development services
check_dev_services() {
    echo -e "${YELLOW}🚀 Development Services:${NC}"
    
    local services_healthy=0
    local total_services=3
    
    # Check AI API
    if check_service "AI API" 8003 "http://localhost:8003/health" "healthy"; then
        services_healthy=$((services_healthy + 1))
    fi
    
    # Check BFF
    if check_service "BFF API" 8002 "http://localhost:8002/health" "healthy"; then
        services_healthy=$((services_healthy + 1))
    fi
    
    # Check Frontend
    if check_service "Frontend" 3000 "http://localhost:3000" "any"; then
        services_healthy=$((services_healthy + 1))
    fi
    
    echo ""
    echo -e "${YELLOW}📊 Development Services Summary:${NC}"
    echo -e "  • Healthy: ${GREEN}$services_healthy${NC}/$total_services"
    
    if [ $services_healthy -eq $total_services ]; then
        echo -e "  • Status: ${GREEN}✅ ALL SERVICES HEALTHY${NC}"
    elif [ $services_healthy -gt 0 ]; then
        echo -e "  • Status: ${YELLOW}⚠️  PARTIAL SERVICES RUNNING${NC}"
    else
        echo -e "  • Status: ${RED}❌ NO SERVICES RUNNING${NC}"
    fi
}

# Function to check system resources
check_system_resources() {
    echo -e "${YELLOW}💻 System Resources:${NC}"
    
    # Memory usage
    local memory_info=$(free -h | awk 'NR==2{printf "Used: %s/%s (%.0f%%)", $3,$2,$3*100/$2}')
    echo -e "  • Memory: $memory_info"
    
    # CPU load
    local cpu_load=$(uptime | awk -F'load average:' '{print $2}' | xargs)
    echo -e "  • CPU Load: $cpu_load"
    
    # Disk usage
    local disk_usage=$(df -h / | awk 'NR==2{printf "%s/%s (%s)", $3,$2,$5}')
    echo -e "  • Disk Usage: $disk_usage"
    
    # Network connections
    local connections=$(netstat -tn 2>/dev/null | grep ESTABLISHED | wc -l)
    echo -e "  • Active Connections: $connections"
}

# Function to check API endpoints
check_api_endpoints() {
    echo -e "${YELLOW}🔗 API Endpoints:${NC}"
    
    local endpoints=(
        "AI Health:http://localhost:8003/health"
        "BFF Health:http://localhost:8002/health"
        "Trading Signals:http://localhost:8002/api/trading/signals"
        "Trading Status:http://localhost:8002/api/trading/status"
        "Wallet Balance:http://localhost:8002/api/wallet/balance"
        "Strategies:http://localhost:8002/api/strategies"
    )
    
    for endpoint in "${endpoints[@]}"; do
        IFS=':' read -r name url <<< "$endpoint"
        
        if curl -s -f "$url" > /dev/null 2>&1; then
            local response_time=$(curl -s -w "%{time_total}" -o /dev/null "$url")
            echo -e "  • $name: ${GREEN}✅ OK${NC} (${response_time}s)"
        else
            echo -e "  • $name: ${RED}❌ FAILED${NC}"
        fi
    done
}

# Function to check processes
check_processes() {
    echo -e "${YELLOW}⚙️  Related Processes:${NC}"
    
    # Python processes
    local python_procs=$(ps aux | grep -E "(deepseek_api_mock|main_simple)" | grep -v grep | wc -l)
    echo -e "  • Python Services: $python_procs running"
    
    # HTTP server processes
    local http_procs=$(ps aux | grep "http.server" | grep -v grep | wc -l)
    echo -e "  • HTTP Servers: $http_procs running"
    
    # Docker processes
    local docker_procs=$(docker ps --format "table {{.Names}}" 2>/dev/null | tail -n +2 | wc -l)
    echo -e "  • Docker Containers: $docker_procs running"
}

# Function to check configuration
check_configuration() {
    echo -e "${YELLOW}⚙️  Configuration:${NC}"
    
    # Check environment files
    if [ -f ".env.development" ]; then
        echo -e "  • Development Config: ${GREEN}✅ FOUND${NC}"
    else
        echo -e "  • Development Config: ${YELLOW}⚠️  NOT FOUND${NC}"
    fi
    
    if [ -f ".env.production" ]; then
        echo -e "  • Production Config: ${GREEN}✅ FOUND${NC}"
    else
        echo -e "  • Production Config: ${YELLOW}⚠️  NOT FOUND${NC}"
    fi
    
    # Check Python virtual environment
    if [ -d "cerebro/venv" ]; then
        echo -e "  • Python Virtual Env: ${GREEN}✅ FOUND${NC}"
    else
        echo -e "  • Python Virtual Env: ${RED}❌ NOT FOUND${NC}"
    fi
    
    # Check Node.js dependencies
    if [ -d "hft-ninja-frontend/node_modules" ]; then
        echo -e "  • Node.js Dependencies: ${GREEN}✅ FOUND${NC}"
    else
        echo -e "  • Node.js Dependencies: ${RED}❌ NOT FOUND${NC}"
    fi
    
    # Check frontend build
    if [ -d "hft-ninja-frontend/build" ]; then
        echo -e "  • Frontend Build: ${GREEN}✅ FOUND${NC}"
    else
        echo -e "  • Frontend Build: ${YELLOW}⚠️  NOT FOUND${NC}"
    fi
}

# Function to display recommendations
display_recommendations() {
    echo ""
    echo -e "${YELLOW}💡 Recommendations:${NC}"
    
    # Check if any development services are running
    local dev_running=false
    if lsof -Pi :8003 -sTCP:LISTEN -t >/dev/null 2>&1 || \
       lsof -Pi :8002 -sTCP:LISTEN -t >/dev/null 2>&1 || \
       lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null 2>&1; then
        dev_running=true
    fi
    
    if [ "$dev_running" = false ]; then
        echo -e "  • Start development: ${GREEN}make dev${NC}"
        echo -e "  • Start with logs: ${GREEN}make dev-verbose${NC}"
    else
        echo -e "  • View logs: ${GREEN}make logs${NC}"
        echo -e "  • Run tests: ${GREEN}make test${NC}"
        echo -e "  • Open frontend: ${GREEN}http://localhost:3000${NC}"
    fi
    
    # Check if Docker services are available
    if command -v docker-compose &> /dev/null && [ -f "docker-compose.traefik.yml" ]; then
        echo -e "  • Start production: ${GREEN}make prod${NC}"
    fi
    
    echo -e "  • Check health: ${GREEN}make health${NC}"
    echo -e "  • Stop services: ${GREEN}make down${NC}"
}

# Main execution
main() {
    # Check development services
    check_dev_services
    echo ""
    
    # Check Docker services
    check_docker_services
    echo ""
    
    # Check API endpoints
    check_api_endpoints
    echo ""
    
    # Check system resources
    check_system_resources
    echo ""
    
    # Check processes
    check_processes
    echo ""
    
    # Check configuration
    check_configuration
    
    # Display recommendations
    display_recommendations
    
    echo ""
    echo -e "${GREEN}📊 Status check completed!${NC}"
}

# Run main function
main "$@"
