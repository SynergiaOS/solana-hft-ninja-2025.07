# =========================================================================
#  🥷 Solana HFT Ninja 2025.07 - Development Kit
#  Complete development toolkit for HFT trading system
# =========================================================================

# Configuration
COMPOSE_FILE := docker-compose.traefik.yml
COMPOSE_DEV := docker-compose.dev.yml
PROJECT_NAME := solana-hft-ninja

# Colors for better readability
GREEN := \033[0;32m
YELLOW := \033[0;33m
BLUE := \033[0;34m
RED := \033[0;31m
RESET := \033[0m

.PHONY: help dev prod down logs status clean test build deploy

# =============================================================================
# 📋 HELP & DOCUMENTATION
# =============================================================================

help:
	@echo -e "$(GREEN)🥷 Solana HFT Ninja 2025.07 - Development Kit$(RESET)"
	@echo -e "$(BLUE)========================================$(RESET)"
	@echo ""
	@echo -e "$(YELLOW)🚀 DEVELOPMENT COMMANDS:$(RESET)"
	@echo -e "  $(GREEN)make dev$(RESET)              - Start development environment (local services)"
	@echo -e "  $(GREEN)make dev-verbose$(RESET)      - Start development with live logs"
	@echo -e "  $(GREEN)make prod$(RESET)             - Start production environment (Traefik + full stack)"
	@echo -e "  $(GREEN)make down$(RESET)             - Stop all services"
	@echo -e "  $(GREEN)make restart$(RESET)          - Restart all services"
	@echo ""
	@echo -e "$(YELLOW)📊 MONITORING & DEBUGGING:$(RESET)"
	@echo -e "  $(GREEN)make logs$(RESET)             - Follow logs from all services"
	@echo -e "  $(GREEN)make logs-ai$(RESET)          - Follow AI service logs"
	@echo -e "  $(GREEN)make logs-bff$(RESET)         - Follow BFF service logs"
	@echo -e "  $(GREEN)make logs-trading$(RESET)     - Follow trading engine logs"
	@echo -e "  $(GREEN)make status$(RESET)           - Show status of all services"
	@echo -e "  $(GREEN)make health$(RESET)           - Check health of all endpoints"
	@echo ""
	@echo -e "$(YELLOW)🧪 TESTING & VALIDATION:$(RESET)"
	@echo -e "  $(GREEN)make test$(RESET)             - Run complete test suite"
	@echo -e "  $(GREEN)make test-api$(RESET)         - Test all API endpoints"
	@echo -e "  $(GREEN)make test-trading$(RESET)     - Test trading functionality"
	@echo -e "  $(GREEN)make test-ai$(RESET)          - Test AI calculations"
	@echo ""
	@echo -e "$(YELLOW)🔧 BUILD & DEPLOYMENT:$(RESET)"
	@echo -e "  $(GREEN)make build$(RESET)            - Build all Docker images"
	@echo -e "  $(GREEN)make build-frontend$(RESET)   - Build React frontend"
	@echo -e "  $(GREEN)make deploy$(RESET)           - Deploy to production (Oracle/Traefik)"
	@echo -e "  $(GREEN)make clean$(RESET)            - Clean all data and volumes"
	@echo ""
	@echo -e "$(YELLOW)📈 STRATEGY MANAGEMENT:$(RESET)"
	@echo -e "  $(GREEN)make strategy-list$(RESET)    - List all trading strategies"
	@echo -e "  $(GREEN)make strategy-deploy s=<name>$(RESET) - Deploy strategy"
	@echo -e "  $(GREEN)make strategy-scale s=<name> n=<count>$(RESET) - Scale strategy"
	@echo ""
	@echo -e "$(YELLOW)🛡️ SECURITY & INFRASTRUCTURE:$(RESET)"
	@echo -e "  $(GREEN)make security-scan$(RESET)    - Run security vulnerability scan"
	@echo -e "  $(GREEN)make backup$(RESET)           - Backup all data"
	@echo -e "  $(GREEN)make restore$(RESET)          - Restore from backup"

# =============================================================================
# 🚀 DEVELOPMENT ENVIRONMENT
# =============================================================================

dev:
	@echo -e "$(GREEN)🚀 Starting HFT Ninja development environment...$(RESET)"
	@echo -e "$(BLUE)Services: AI API (8003), BFF (8002), Frontend (3000)$(RESET)"
	@./scripts/start-dev-stack.sh

dev-verbose:
	@echo -e "$(GREEN)🚀 Starting HFT Ninja development with live logs...$(RESET)"
	@./scripts/start-dev-stack.sh --verbose

prod:
	@echo -e "$(GREEN)🚀 Starting HFT Ninja production environment...$(RESET)"
	@echo -e "$(BLUE)Services: Traefik, AI, BFF, Frontend, Monitoring$(RESET)"
	docker-compose -f $(COMPOSE_FILE) up -d
	@echo -e "$(GREEN)✅ Production stack started!$(RESET)"
	@echo -e "$(YELLOW)🌐 Access points:$(RESET)"
	@echo -e "  • Frontend: http://localhost:3000"
	@echo -e "  • API: http://localhost:8002"
	@echo -e "  • Traefik Dashboard: http://localhost:8080"

down:
	@echo -e "$(YELLOW)🛑 Stopping all HFT Ninja services...$(RESET)"
	@docker-compose -f $(COMPOSE_FILE) down 2>/dev/null || true
	@./scripts/stop-dev-stack.sh 2>/dev/null || true
	@echo -e "$(GREEN)✅ All services stopped$(RESET)"

restart: down
	@sleep 2
	@make dev

# =============================================================================
# 📊 MONITORING & DEBUGGING
# =============================================================================

logs:
	@echo -e "$(GREEN)📜 Following logs from all services... (Ctrl+C to exit)$(RESET)"
	@./scripts/show-logs.sh all

logs-ai:
	@echo -e "$(GREEN)📜 Following AI service logs... (Ctrl+C to exit)$(RESET)"
	@./scripts/show-logs.sh ai

logs-bff:
	@echo -e "$(GREEN)📜 Following BFF service logs... (Ctrl+C to exit)$(RESET)"
	@./scripts/show-logs.sh bff

logs-trading:
	@echo -e "$(GREEN)📜 Following trading engine logs... (Ctrl+C to exit)$(RESET)"
	@./scripts/show-logs.sh trading

status:
	@echo -e "$(GREEN)📊 HFT Ninja Services Status:$(RESET)"
	@./scripts/check-status.sh

health:
	@echo -e "$(GREEN)🏥 Checking health of all endpoints...$(RESET)"
	@./scripts/health-check.sh

# =============================================================================
# 🧪 TESTING & VALIDATION
# =============================================================================

test:
	@echo -e "$(GREEN)🧪 Running complete test suite...$(RESET)"
	@./scripts/run-tests.sh all

test-api:
	@echo -e "$(GREEN)🧪 Testing API endpoints...$(RESET)"
	@./scripts/run-tests.sh api

test-trading:
	@echo -e "$(GREEN)🧪 Testing trading functionality...$(RESET)"
	@./scripts/run-tests.sh trading

test-ai:
	@echo -e "$(GREEN)🧪 Testing AI calculations...$(RESET)"
	@./scripts/run-tests.sh ai

# =============================================================================
# 🔧 BUILD & DEPLOYMENT
# =============================================================================

build:
	@echo -e "$(GREEN)🔧 Building all Docker images...$(RESET)"
	@docker-compose -f $(COMPOSE_FILE) build --parallel
	@echo -e "$(GREEN)✅ All images built successfully$(RESET)"

build-frontend:
	@echo -e "$(GREEN)🔧 Building React frontend...$(RESET)"
	@cd hft-ninja-frontend && npm run build
	@echo -e "$(GREEN)✅ Frontend built successfully$(RESET)"

deploy:
	@echo -e "$(GREEN)🚀 Deploying to production...$(RESET)"
	@./scripts/deploy-production-gateway.sh
	@echo -e "$(GREEN)✅ Production deployment completed$(RESET)"

clean:
	@echo -e "$(YELLOW)🧹 Cleaning all data and volumes...$(RESET)"
	@echo -e "$(RED)⚠️  This will remove ALL data! Press Ctrl+C to cancel...$(RESET)"
	@sleep 5
	@docker-compose -f $(COMPOSE_FILE) down -v
	@docker system prune -f
	@echo -e "$(GREEN)✅ Cleanup completed$(RESET)"

# =============================================================================
# 📈 STRATEGY MANAGEMENT
# =============================================================================

strategy-list:
	@echo -e "$(GREEN)📈 Available trading strategies:$(RESET)"
	@./scripts/strategy-manager.sh list

strategy-deploy:
	@echo -e "$(GREEN)📈 Deploying strategy: $(s)$(RESET)"
	@./scripts/strategy-manager.sh deploy $(s)

strategy-scale:
	@echo -e "$(GREEN)📈 Scaling strategy $(s) to $(n) instances$(RESET)"
	@./scripts/strategy-manager.sh scale $(s) $(n)

# =============================================================================
# 🛡️ SECURITY & MAINTENANCE
# =============================================================================

security-scan:
	@echo -e "$(GREEN)🛡️ Running security vulnerability scan...$(RESET)"
	@./scripts/security-scan.sh

backup:
	@echo -e "$(GREEN)💾 Creating backup...$(RESET)"
	@./scripts/backup-data.sh

restore:
	@echo -e "$(GREEN)🔄 Restoring from backup...$(RESET)"
	@./scripts/restore-data.sh

# =============================================================================
# 🎯 QUICK ACTIONS
# =============================================================================

quick-start: dev health
	@echo -e "$(GREEN)🎉 HFT Ninja is ready for development!$(RESET)"

quick-test: test-api test-ai
	@echo -e "$(GREEN)🎉 All tests passed!$(RESET)"

quick-deploy: build deploy
	@echo -e "$(GREEN)🎉 Production deployment completed!$(RESET)"
