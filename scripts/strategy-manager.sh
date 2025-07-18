#!/bin/bash

# 🎯 Dynamic Strategy Manager for Solana HFT Ninja with Traefik
# Manages trading strategies dynamically without downtime

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.traefik.yml}"
DOMAIN="${DOMAIN:-api.hft-ninja.com}"
STRATEGIES_DIR="./strategies"

echo -e "${BLUE}🎯 HFT Ninja Strategy Manager${NC}"
echo -e "${GREEN}Managing strategies dynamically with Traefik${NC}"
echo ""

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

# Function to list available strategies
list_strategies() {
    print_status "Available strategies:"
    
    # Get running strategies
    running_strategies=$(docker-compose -f "$COMPOSE_FILE" ps --services | grep "strategy-" || true)
    
    if [ -n "$running_strategies" ]; then
        echo -e "${GREEN}🟢 Running Strategies:${NC}"
        for strategy in $running_strategies; do
            strategy_name=$(echo "$strategy" | sed 's/strategy-//')
            status=$(docker-compose -f "$COMPOSE_FILE" ps "$strategy" --format "table {{.State}}" | tail -n +2)
            echo "  • $strategy_name: $status"
        done
    else
        echo -e "${YELLOW}⚠️  No strategies currently running${NC}"
    fi
    
    echo ""
    
    # Get available strategy definitions
    if [ -d "$STRATEGIES_DIR" ]; then
        echo -e "${BLUE}📁 Available Strategy Definitions:${NC}"
        for strategy_file in "$STRATEGIES_DIR"/*.yml; do
            if [ -f "$strategy_file" ]; then
                strategy_name=$(basename "$strategy_file" .yml)
                echo "  • $strategy_name"
            fi
        done
    fi
}

# Function to deploy a new strategy
deploy_strategy() {
    local strategy_name="$1"
    local strategy_config="$2"
    
    if [ -z "$strategy_name" ]; then
        print_error "Strategy name is required"
        return 1
    fi
    
    print_status "Deploying strategy: $strategy_name"
    
    # Create strategy configuration if provided
    if [ -n "$strategy_config" ]; then
        mkdir -p "$STRATEGIES_DIR"
        echo "$strategy_config" > "$STRATEGIES_DIR/$strategy_name.yml"
    fi
    
    # Generate dynamic Docker Compose override
    cat > "docker-compose.strategy-$strategy_name.yml" << EOF
version: '3.8'

services:
  strategy-$strategy_name:
    build:
      context: .
      dockerfile: Dockerfile.strategy
      args:
        STRATEGY_TYPE: $strategy_name
    container_name: strategy-$strategy_name
    restart: unless-stopped
    
    environment:
      - RUST_LOG=info
      - STRATEGY_NAME=$strategy_name
      - CORE_ENGINE_URL=http://hft-ninja-core:8080
      - STRATEGY_CONFIG_PATH=/app/config/$strategy_name.yml
    
    volumes:
      - ./config:/app/config:ro
      - ./logs:/app/logs
      - ./strategies/$strategy_name.yml:/app/config/$strategy_name.yml:ro
    
    networks:
      - hft-network
      - traefik
    
    labels:
      # Enable Traefik
      - "traefik.enable=true"
      - "traefik.docker.network=traefik"
      
      # Strategy API Routes
      - "traefik.http.routers.strategy-$strategy_name.rule=Host(\`$DOMAIN\`) && PathPrefix(\`/strategies/$strategy_name\`)"
      - "traefik.http.routers.strategy-$strategy_name.tls.certresolver=letsencrypt"
      - "traefik.http.routers.strategy-$strategy_name.middlewares=strategy-rate-limit,security-headers"
      - "traefik.http.services.strategy-$strategy_name.loadbalancer.server.port=808\$(echo \$RANDOM | cut -c1-1)"
    
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 30s
    
    depends_on:
      - hft-ninja-core

networks:
  hft-network:
    external: true
  traefik:
    external: true
EOF
    
    # Deploy the strategy
    docker-compose -f "$COMPOSE_FILE" -f "docker-compose.strategy-$strategy_name.yml" up -d "strategy-$strategy_name"
    
    # Wait for deployment
    print_status "Waiting for strategy to become healthy..."
    sleep 10
    
    # Check if strategy is running
    if docker-compose -f "$COMPOSE_FILE" -f "docker-compose.strategy-$strategy_name.yml" ps "strategy-$strategy_name" | grep -q "Up"; then
        print_status "✅ Strategy $strategy_name deployed successfully"
        print_status "🌐 Available at: https://$DOMAIN/strategies/$strategy_name"
        
        # Test endpoint
        if curl -s -f "https://$DOMAIN/strategies/$strategy_name/health" > /dev/null; then
            print_status "✅ Strategy endpoint is responding"
        else
            print_warning "⚠️  Strategy endpoint not yet responding (may need a few more seconds)"
        fi
    else
        print_error "❌ Strategy deployment failed"
        return 1
    fi
}

# Function to remove a strategy
remove_strategy() {
    local strategy_name="$1"
    
    if [ -z "$strategy_name" ]; then
        print_error "Strategy name is required"
        return 1
    fi
    
    print_status "Removing strategy: $strategy_name"
    
    # Stop and remove the strategy container
    if [ -f "docker-compose.strategy-$strategy_name.yml" ]; then
        docker-compose -f "$COMPOSE_FILE" -f "docker-compose.strategy-$strategy_name.yml" down "strategy-$strategy_name"
        rm -f "docker-compose.strategy-$strategy_name.yml"
        print_status "✅ Strategy $strategy_name removed successfully"
    else
        print_warning "⚠️  Strategy configuration not found"
    fi
}

# Function to scale a strategy
scale_strategy() {
    local strategy_name="$1"
    local replicas="$2"
    
    if [ -z "$strategy_name" ] || [ -z "$replicas" ]; then
        print_error "Strategy name and replica count are required"
        return 1
    fi
    
    print_status "Scaling strategy $strategy_name to $replicas replicas"
    
    if [ -f "docker-compose.strategy-$strategy_name.yml" ]; then
        docker-compose -f "$COMPOSE_FILE" -f "docker-compose.strategy-$strategy_name.yml" up -d --scale "strategy-$strategy_name=$replicas"
        print_status "✅ Strategy $strategy_name scaled to $replicas replicas"
    else
        print_error "❌ Strategy configuration not found"
        return 1
    fi
}

# Function to show strategy logs
show_logs() {
    local strategy_name="$1"
    local lines="${2:-50}"
    
    if [ -z "$strategy_name" ]; then
        print_error "Strategy name is required"
        return 1
    fi
    
    print_status "Showing logs for strategy: $strategy_name (last $lines lines)"
    
    if docker-compose -f "$COMPOSE_FILE" ps "strategy-$strategy_name" | grep -q "strategy-$strategy_name"; then
        docker-compose -f "$COMPOSE_FILE" logs --tail="$lines" "strategy-$strategy_name"
    else
        print_error "❌ Strategy $strategy_name is not running"
        return 1
    fi
}

# Function to show strategy status
show_status() {
    local strategy_name="$1"
    
    if [ -z "$strategy_name" ]; then
        # Show all strategies
        list_strategies
        return 0
    fi
    
    print_status "Status for strategy: $strategy_name"
    
    if docker-compose -f "$COMPOSE_FILE" ps "strategy-$strategy_name" | grep -q "strategy-$strategy_name"; then
        # Container status
        echo -e "${BLUE}📦 Container Status:${NC}"
        docker-compose -f "$COMPOSE_FILE" ps "strategy-$strategy_name"
        echo ""
        
        # Health check
        echo -e "${BLUE}🏥 Health Status:${NC}"
        if curl -s -f "https://$DOMAIN/strategies/$strategy_name/health" > /dev/null; then
            echo "✅ Healthy"
        else
            echo "❌ Unhealthy"
        fi
        echo ""
        
        # Resource usage
        echo -e "${BLUE}💻 Resource Usage:${NC}"
        docker stats "strategy-$strategy_name" --no-stream --format "table {{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}"
    else
        print_error "❌ Strategy $strategy_name is not running"
        return 1
    fi
}

# Function to create a new strategy template
create_strategy() {
    local strategy_name="$1"
    local strategy_type="${2:-basic}"
    
    if [ -z "$strategy_name" ]; then
        print_error "Strategy name is required"
        return 1
    fi
    
    print_status "Creating new strategy: $strategy_name (type: $strategy_type)"
    
    mkdir -p "$STRATEGIES_DIR"
    
    # Create strategy configuration
    cat > "$STRATEGIES_DIR/$strategy_name.yml" << EOF
# Strategy Configuration: $strategy_name
name: $strategy_name
type: $strategy_type
created: $(date -u +%Y-%m-%dT%H:%M:%SZ)

# Trading Parameters
parameters:
  max_position_size: 1.0
  min_profit_threshold: 0.001
  max_slippage: 0.005
  risk_level: medium
  
# Market Conditions
conditions:
  min_liquidity: 1000.0
  max_volatility: 0.1
  allowed_tokens:
    - SOL
    - USDC
    - USDT
    
# Risk Management
risk_management:
  stop_loss: 0.02
  take_profit: 0.05
  max_daily_loss: 0.1
  position_timeout: 300  # seconds

# AI Integration
ai:
  enabled: true
  model: deepseek-math
  confidence_threshold: 0.8
  use_sentiment: false
EOF
    
    print_status "✅ Strategy configuration created: $STRATEGIES_DIR/$strategy_name.yml"
    print_status "📝 Edit the configuration file and then deploy with:"
    print_status "   $0 deploy $strategy_name"
}

# Main function
main() {
    local command="$1"
    shift
    
    case "$command" in
        "list"|"ls")
            list_strategies
            ;;
        "deploy")
            deploy_strategy "$@"
            ;;
        "remove"|"rm")
            remove_strategy "$@"
            ;;
        "scale")
            scale_strategy "$@"
            ;;
        "logs")
            show_logs "$@"
            ;;
        "status")
            show_status "$@"
            ;;
        "create")
            create_strategy "$@"
            ;;
        "help"|"--help"|"-h")
            echo "Usage: $0 <command> [options]"
            echo ""
            echo "Commands:"
            echo "  list                    List all strategies"
            echo "  deploy <name>           Deploy a strategy"
            echo "  remove <name>           Remove a strategy"
            echo "  scale <name> <count>    Scale strategy replicas"
            echo "  logs <name> [lines]     Show strategy logs"
            echo "  status [name]           Show strategy status"
            echo "  create <name> [type]    Create new strategy template"
            echo "  help                    Show this help"
            echo ""
            echo "Examples:"
            echo "  $0 create my-strategy arbitrage"
            echo "  $0 deploy my-strategy"
            echo "  $0 scale my-strategy 3"
            echo "  $0 status my-strategy"
            echo "  $0 logs my-strategy 100"
            echo "  $0 remove my-strategy"
            ;;
        *)
            print_error "Unknown command: $command"
            echo "Use '$0 help' for usage information"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
