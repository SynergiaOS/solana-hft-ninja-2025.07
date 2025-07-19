#!/bin/bash

# 🥷 Solana HFT Ninja 2025.07 - Enhanced Grafana Startup Script
# Uruchamia kompletny stack monitoringu z nowymi dashboardami HFT

set -euo pipefail

# Kolory dla lepszej czytelności
readonly GREEN="\033[0;32m"
readonly YELLOW="\033[0;33m"
readonly CYAN="\033[0;36m"
readonly RED="\033[0;31m"
readonly BLUE="\033[0;34m"
readonly RESET="\033[0m"

# Konfiguracja
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo -e "${CYAN}🥷 Solana HFT Ninja 2025.07 - Enhanced Monitoring Stack${RESET}"
echo -e "${BLUE}Starting Prometheus + Grafana + Node Exporter with HFT Dashboards...${RESET}"
echo ""

# Sprawdź czy Docker jest dostępny
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker nie jest zainstalowany lub niedostępny${RESET}"
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}❌ Docker Compose nie jest zainstalowany lub niedostępny${RESET}"
    exit 1
fi

echo -e "${GREEN}✅ Docker i Docker Compose są dostępne${RESET}"

# Sprawdź czy pliki konfiguracyjne istnieją
echo -e "${YELLOW}🔍 Sprawdzanie plików konfiguracyjnych...${RESET}"

required_files=(
    "docker-compose.grafana.yml"
    "grafana/grafana.ini"
    "grafana/dashboards/hft-ninja-comprehensive.json"
    "grafana/dashboards/mev-strategies-comprehensive.json"
    "grafana/provisioning/dashboards/dashboards.yml"
)

for file in "${required_files[@]}"; do
    if [ -f "${SCRIPT_DIR}/${file}" ]; then
        echo -e "${GREEN}✅ ${file}${RESET}"
    else
        echo -e "${RED}❌ Brakuje pliku: ${file}${RESET}"
        exit 1
    fi
done

echo ""

# Stwórz katalogi jeśli nie istnieją
echo -e "${YELLOW}📁 Tworzenie katalogów...${RESET}"
mkdir -p "${SCRIPT_DIR}/grafana/data"
mkdir -p "${SCRIPT_DIR}/prometheus/data"

# Ustaw uprawnienia
echo -e "${YELLOW}🔐 Ustawianie uprawnień...${RESET}"
sudo chown -R 472:472 "${SCRIPT_DIR}/grafana/data" 2>/dev/null || true
sudo chown -R 65534:65534 "${SCRIPT_DIR}/prometheus/data" 2>/dev/null || true

# Zatrzymaj istniejące kontenery jeśli działają
echo -e "${YELLOW}🛑 Zatrzymywanie istniejących kontenerów...${RESET}"
cd "${SCRIPT_DIR}"
docker-compose -f docker-compose.grafana.yml down 2>/dev/null || true

# Uruchom stack monitoringu
echo -e "${CYAN}🚀 Uruchamianie enhanced monitoring stack...${RESET}"
docker-compose -f docker-compose.grafana.yml up -d

# Sprawdź status kontenerów
echo ""
echo -e "${YELLOW}📊 Status kontenerów:${RESET}"
sleep 5

containers=("hft-prometheus" "hft-grafana" "hft-node-exporter")

for container in "${containers[@]}"; do
    if docker ps --format "table {{.Names}}\t{{.Status}}" | grep -q "$container"; then
        status=$(docker ps --format "table {{.Names}}\t{{.Status}}" | grep "$container" | awk '{print $2, $3, $4}')
        echo -e "${GREEN}✅ $container: $status${RESET}"
    else
        echo -e "${RED}❌ $container: Not running${RESET}"
    fi
done

echo ""

# Sprawdź health checks
echo -e "${YELLOW}🏥 Sprawdzanie health checks...${RESET}"
sleep 10

# Prometheus health check
if curl -s http://localhost:9090/-/healthy > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Prometheus: Healthy${RESET}"
else
    echo -e "${RED}❌ Prometheus: Unhealthy${RESET}"
fi

# Grafana health check
if curl -s http://localhost:3000/api/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Grafana: Healthy${RESET}"
else
    echo -e "${RED}❌ Grafana: Unhealthy (może jeszcze się uruchamiać...)${RESET}"
fi

# Node Exporter health check
if curl -s http://localhost:9100/metrics > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Node Exporter: Healthy${RESET}"
else
    echo -e "${RED}❌ Node Exporter: Unhealthy${RESET}"
fi

echo ""

# Wyświetl informacje o dostępie
echo -e "${GREEN}🎉 Enhanced Monitoring Stack uruchomiony pomyślnie!${RESET}"
echo ""
echo -e "${CYAN}📊 Dostęp do usług:${RESET}"
echo -e "${BLUE}Grafana:${RESET}       http://localhost:3000"
echo -e "${BLUE}  Login:${RESET}       admin"
echo -e "${BLUE}  Password:${RESET}    hft-ninja-2025"
echo ""
echo -e "${BLUE}Prometheus:${RESET}    http://localhost:9090"
echo -e "${BLUE}Node Exporter:${RESET} http://localhost:9100"
echo ""

echo -e "${CYAN}📋 Nowe dashboardy HFT:${RESET}"
echo -e "${BLUE}• 🥷 HFT Ninja - Comprehensive Dashboard${RESET}"
echo -e "${BLUE}  - 💰 Real-time P&L tracking${RESET}"
echo -e "${BLUE}  - ⚡ Execution latency metrics${RESET}"
echo -e "${BLUE}  - 🎯 Trade success rates${RESET}"
echo -e "${BLUE}  - 🛡️ Security & circuit breaker status${RESET}"
echo ""
echo -e "${BLUE}• 🎯 MEV Strategies - Comprehensive Dashboard${RESET}"
echo -e "${BLUE}  - 🥪 Sandwich opportunities${RESET}"
echo -e "${BLUE}  - ⚖️ Arbitrage detection${RESET}"
echo -e "${BLUE}  - 🔥 Liquidation opportunities${RESET}"
echo -e "${BLUE}  - 📦 Jito bundle metrics${RESET}"
echo ""

echo -e "${YELLOW}💡 Przydatne komendy:${RESET}"
echo -e "${BLUE}Logi Grafana:${RESET}     docker logs -f hft-grafana"
echo -e "${BLUE}Logi Prometheus:${RESET}  docker logs -f hft-prometheus"
echo -e "${BLUE}Zatrzymaj stack:${RESET}  docker-compose -f docker-compose.grafana.yml down"
echo -e "${BLUE}Restart stack:${RESET}    docker-compose -f docker-compose.grafana.yml restart"
echo ""

echo -e "${CYAN}🎯 Kluczowe metryki do monitorowania:${RESET}"
echo -e "${BLUE}• hft_total_profit_sol - hft_total_loss_sol${RESET} (Net P&L)"
echo -e "${BLUE}• hft_execution_latency_seconds${RESET} (Execution latency)"
echo -e "${BLUE}• hft_trades_successful_total / hft_trades_executed_total${RESET} (Success rate)"
echo -e "${BLUE}• hft_mev_profit_sol${RESET} (MEV profit)"
echo -e "${BLUE}• hft_circuit_breaker_state${RESET} (Security status)"
echo ""

echo -e "${GREEN}🥷 Enhanced Monitoring gotowy do użycia!${RESET}"
echo -e "${YELLOW}💡 Dashboardy automatycznie załadują się w Grafana${RESET}"
