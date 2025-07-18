#!/bin/bash

# 📜 HFT Ninja Log Viewer
# Intelligent log viewing for all services

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SERVICE=${1:-all}
LINES=${2:-50}

echo -e "${BLUE}📜 Solana HFT Ninja - Log Viewer${NC}"
echo -e "${GREEN}========================================${NC}"
echo -e "${YELLOW}Service: $SERVICE | Lines: $LINES${NC}"

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

# Function to get process logs
get_process_logs() {
    local service_name=$1
    local pid_file="/tmp/hft-ninja-$service_name.pid"
    local log_file="/tmp/hft-ninja-$service_name.log"
    
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            if [ -f "$log_file" ]; then
                echo -e "${GREEN}📋 $service_name Logs (PID: $pid):${NC}"
                tail -n $LINES "$log_file" | while IFS= read -r line; do
                    # Color code log levels
                    if echo "$line" | grep -q "ERROR"; then
                        echo -e "${RED}$line${NC}"
                    elif echo "$line" | grep -q "WARNING\|WARN"; then
                        echo -e "${YELLOW}$line${NC}"
                    elif echo "$line" | grep -q "INFO"; then
                        echo -e "${GREEN}$line${NC}"
                    elif echo "$line" | grep -q "DEBUG"; then
                        echo -e "${BLUE}$line${NC}"
                    else
                        echo "$line"
                    fi
                done
            else
                print_warning "$service_name log file not found"
            fi
        else
            print_warning "$service_name is not running"
        fi
    else
        print_warning "$service_name PID file not found"
    fi
}

# Function to follow process logs
follow_process_logs() {
    local service_name=$1
    local pid_file="/tmp/hft-ninja-$service_name.pid"
    local log_file="/tmp/hft-ninja-$service_name.log"
    
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            if [ -f "$log_file" ]; then
                echo -e "${GREEN}📋 Following $service_name Logs (PID: $pid) - Press Ctrl+C to exit:${NC}"
                tail -f "$log_file" | while IFS= read -r line; do
                    # Color code log levels with timestamp
                    local timestamp=$(date '+%H:%M:%S')
                    if echo "$line" | grep -q "ERROR"; then
                        echo -e "${RED}[$timestamp] $line${NC}"
                    elif echo "$line" | grep -q "WARNING\|WARN"; then
                        echo -e "${YELLOW}[$timestamp] $line${NC}"
                    elif echo "$line" | grep -q "INFO"; then
                        echo -e "${GREEN}[$timestamp] $line${NC}"
                    elif echo "$line" | grep -q "DEBUG"; then
                        echo -e "${BLUE}[$timestamp] $line${NC}"
                    else
                        echo "[$timestamp] $line"
                    fi
                done
            else
                print_error "$service_name log file not found"
                return 1
            fi
        else
            print_error "$service_name is not running"
            return 1
        fi
    else
        print_error "$service_name PID file not found"
        return 1
    fi
}

# Function to get live system logs
get_live_system_logs() {
    local service_name=$1
    local port=$2
    
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        local pid=$(lsof -ti:$port)
        echo -e "${GREEN}📋 $service_name Live Logs (PID: $pid, Port: $port):${NC}"
        
        # Try to get logs from journalctl if it's a systemd service
        if systemctl is-active --quiet "$service_name" 2>/dev/null; then
            journalctl -u "$service_name" -n $LINES --no-pager -f
        else
            # Fallback to process output
            print_status "Monitoring process output..."
            while kill -0 "$pid" 2>/dev/null; do
                # Monitor network activity as a proxy for service activity
                local connections=$(netstat -tn 2>/dev/null | grep ":$port " | wc -l)
                echo "$(date '+%H:%M:%S') - Active connections: $connections"
                sleep 2
            done
        fi
    else
        print_error "$service_name is not running on port $port"
        return 1
    fi
}

# Function to show aggregated logs
show_aggregated_logs() {
    echo -e "${GREEN}📋 Aggregated Logs from All Services:${NC}"
    echo -e "${BLUE}========================================${NC}"
    
    local services=("AI-API" "BFF" "Frontend")
    local temp_file="/tmp/aggregated_logs.tmp"
    
    # Collect logs from all services
    > "$temp_file"
    
    for service in "${services[@]}"; do
        local log_file="/tmp/hft-ninja-$service.log"
        if [ -f "$log_file" ]; then
            # Add service prefix and timestamp to each line
            tail -n $LINES "$log_file" | while IFS= read -r line; do
                echo "$(date '+%H:%M:%S') [$service] $line" >> "$temp_file"
            done
        fi
    done
    
    # Sort by timestamp and display
    if [ -s "$temp_file" ]; then
        sort "$temp_file" | tail -n $((LINES * 3)) | while IFS= read -r line; do
            # Color code by service
            if echo "$line" | grep -q "\[AI-API\]"; then
                echo -e "${BLUE}$line${NC}"
            elif echo "$line" | grep -q "\[BFF\]"; then
                echo -e "${GREEN}$line${NC}"
            elif echo "$line" | grep -q "\[Frontend\]"; then
                echo -e "${YELLOW}$line${NC}"
            else
                echo "$line"
            fi
        done
    else
        print_warning "No logs found from any service"
    fi
    
    rm -f "$temp_file"
}

# Function to follow aggregated logs
follow_aggregated_logs() {
    echo -e "${GREEN}📋 Following Aggregated Logs - Press Ctrl+C to exit:${NC}"
    echo -e "${BLUE}========================================${NC}"
    
    local services=("AI-API" "BFF" "Frontend")
    local pids=()
    
    # Start following each service log in background
    for service in "${services[@]}"; do
        local log_file="/tmp/hft-ninja-$service.log"
        if [ -f "$log_file" ]; then
            (
                tail -f "$log_file" | while IFS= read -r line; do
                    local timestamp=$(date '+%H:%M:%S')
                    if echo "$line" | grep -q "ERROR"; then
                        echo -e "${RED}[$timestamp] [$service] $line${NC}"
                    elif echo "$line" | grep -q "WARNING\|WARN"; then
                        echo -e "${YELLOW}[$timestamp] [$service] $line${NC}"
                    elif echo "$line" | grep -q "INFO"; then
                        echo -e "${GREEN}[$timestamp] [$service] $line${NC}"
                    elif echo "$line" | grep -q "DEBUG"; then
                        echo -e "${BLUE}[$timestamp] [$service] $line${NC}"
                    else
                        echo "[$timestamp] [$service] $line"
                    fi
                done
            ) &
            pids+=($!)
        fi
    done
    
    # Wait for Ctrl+C
    trap 'echo -e "\n${YELLOW}Stopping log following...${NC}"; for pid in "${pids[@]}"; do kill $pid 2>/dev/null || true; done; exit 0' INT
    
    if [ ${#pids[@]} -gt 0 ]; then
        wait
    else
        print_warning "No log files found to follow"
    fi
}

# Function to show Docker logs
show_docker_logs() {
    local service_name=$1
    
    if command -v docker-compose &> /dev/null && [ -f "docker-compose.traefik.yml" ]; then
        echo -e "${GREEN}📋 Docker Logs for $service_name:${NC}"
        
        if [ "$service_name" = "all" ]; then
            docker-compose -f docker-compose.traefik.yml logs --tail=$LINES
        else
            docker-compose -f docker-compose.traefik.yml logs --tail=$LINES "$service_name"
        fi
    else
        print_warning "Docker Compose not available or configuration not found"
    fi
}

# Function to follow Docker logs
follow_docker_logs() {
    local service_name=$1
    
    if command -v docker-compose &> /dev/null && [ -f "docker-compose.traefik.yml" ]; then
        echo -e "${GREEN}📋 Following Docker Logs for $service_name - Press Ctrl+C to exit:${NC}"
        
        if [ "$service_name" = "all" ]; then
            docker-compose -f docker-compose.traefik.yml logs -f
        else
            docker-compose -f docker-compose.traefik.yml logs -f "$service_name"
        fi
    else
        print_error "Docker Compose not available or configuration not found"
        return 1
    fi
}

# Main execution
main() {
    case $SERVICE in
        "all")
            # Check if Docker services are running
            if command -v docker-compose &> /dev/null && [ -f "docker-compose.traefik.yml" ]; then
                local docker_services=$(docker-compose -f docker-compose.traefik.yml ps --services --filter status=running 2>/dev/null | wc -l)
                if [ $docker_services -gt 0 ]; then
                    print_status "Docker services detected, showing Docker logs"
                    follow_docker_logs "all"
                    return
                fi
            fi
            
            # Fallback to development services
            print_status "Showing development service logs"
            follow_aggregated_logs
            ;;
        "ai")
            follow_process_logs "AI-API"
            ;;
        "bff")
            follow_process_logs "BFF"
            ;;
        "frontend")
            follow_process_logs "Frontend"
            ;;
        "trading")
            # Alias for BFF since trading logic is in BFF
            follow_process_logs "BFF"
            ;;
        "docker")
            follow_docker_logs "all"
            ;;
        *)
            # Try as Docker service name
            if command -v docker-compose &> /dev/null && [ -f "docker-compose.traefik.yml" ]; then
                if docker-compose -f docker-compose.traefik.yml ps "$SERVICE" | grep -q "$SERVICE"; then
                    follow_docker_logs "$SERVICE"
                    return
                fi
            fi
            
            print_error "Unknown service: $SERVICE"
            print_status "Available services: all, ai, bff, frontend, trading, docker"
            print_status "Or use Docker service names: traefik, deepseek-math-primary, cerebro-bff, etc."
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
