#!/bin/bash

# 🎬 Demo Cerberus Refactor-Kit
# Demonstracja możliwości systemu refaktoryzacji

set -euo pipefail

readonly GREEN="\033[0;32m"
readonly YELLOW="\033[0;33m"
readonly CYAN="\033[0;36m"
readonly RED="\033[0;31m"
readonly BLUE="\033[0;34m"
readonly RESET="\033[0m"

readonly PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo -e "${CYAN}🎬 Demo Cerberus Refactor-Kit${RESET}"
echo -e "${BLUE}Demonstracja możliwości inteligentnej refaktoryzacji${RESET}"
echo ""

# Funkcja do wyświetlania kroków
show_step() {
    local step_num="$1"
    local description="$2"
    echo -e "${GREEN}📋 Krok ${step_num}: ${description}${RESET}"
    echo ""
}

# Funkcja do wyświetlania przykładów kodu
show_code_example() {
    local title="$1"
    local code="$2"
    echo -e "${YELLOW}💻 ${title}:${RESET}"
    echo -e "${CYAN}${code}${RESET}"
    echo ""
}

# Demo 1: Analiza zdrowia kodu
demo_code_health() {
    show_step "1" "Analiza zdrowia kodu"
    
    echo -e "${BLUE}Sprawdzamy obecny stan kodu w projekcie...${RESET}"
    echo ""
    
    # Statystyki podstawowe
    echo -e "${YELLOW}📊 Statystyki projektu:${RESET}"
    
    if [ -d "${PROJECT_ROOT}/src" ]; then
        local rust_files=$(find "${PROJECT_ROOT}/src" -name "*.rs" 2>/dev/null | wc -l)
        local rust_lines=$(find "${PROJECT_ROOT}/src" -name "*.rs" -exec cat {} \; 2>/dev/null | wc -l)
        echo "  🦀 Pliki Rust: ${rust_files}"
        echo "  🦀 Linie Rust: ${rust_lines}"
    fi
    
    if [ -d "${PROJECT_ROOT}/cerebro" ]; then
        local python_files=$(find "${PROJECT_ROOT}/cerebro" -name "*.py" 2>/dev/null | wc -l)
        local python_lines=$(find "${PROJECT_ROOT}/cerebro" -name "*.py" -exec cat {} \; 2>/dev/null | wc -l)
        echo "  🐍 Pliki Python: ${python_files}"
        echo "  🐍 Linie Python: ${python_lines}"
    fi
    
    local config_files=$(find "${PROJECT_ROOT}" -name "*.toml" -o -name "*.yml" -o -name "*.yaml" 2>/dev/null | wc -l)
    echo "  ⚙️  Pliki konfiguracyjne: ${config_files}"
    
    echo ""
    
    # Przykład wyszukiwania problemów
    echo -e "${YELLOW}🔍 Przykład wyszukiwania TODO/FIXME:${RESET}"
    if command -v rg &> /dev/null; then
        local todos=$(rg "TODO|FIXME|XXX" "${PROJECT_ROOT}" -n --color=never 2>/dev/null | head -3 || true)
        if [ -n "$todos" ]; then
            echo "$todos" | while read -r line; do
                echo -e "${CYAN}  📌 $line${RESET}"
            done
        else
            echo -e "${GREEN}  ✅ Brak nierozwiązanych TODO/FIXME${RESET}"
        fi
    else
        echo -e "${YELLOW}  ⚠️  ripgrep nie jest zainstalowany - uruchom install-tools.sh${RESET}"
    fi
    
    echo ""
    read -p "Naciśnij Enter aby przejść do następnego demo..."
    echo ""
}

# Demo 2: Przykłady wzorców refaktoryzacji
demo_patterns() {
    show_step "2" "Wzorce refaktoryzacji"
    
    echo -e "${BLUE}Przykłady wzorców, które można bezpiecznie refaktoryzować:${RESET}"
    echo ""
    
    # Rust patterns
    echo -e "${YELLOW}🦀 Wzorce Rust:${RESET}"
    
    show_code_example "Zmiana nazwy funkcji" \
"// Przed:
fn evaluate_opportunity(data: &MarketData) -> Result<Decision> {
    // logika...
}

// Po refaktoryzacji:
fn assess_market_chance(data: &MarketData) -> Result<Decision> {
    // logika...
}"

    show_code_example "Zmiana nazwy struktury" \
"// Przed:
struct TradingBot {
    strategies: Vec<Strategy>,
}

// Po refaktoryzacji:
struct HftEngine {
    strategies: Vec<Strategy>,
}"

    # Python patterns
    echo -e "${YELLOW}🐍 Wzorce Python:${RESET}"
    
    show_code_example "Zmiana nazwy klasy" \
"# Przed:
class DataProcessor:
    def process(self, data):
        pass

# Po refaktoryzacji:
class MarketAnalyzer:
    def process(self, data):
        pass"

    # Config patterns
    echo -e "${YELLOW}⚙️ Wzorce konfiguracji:${RESET}"
    
    show_code_example "Zmiana klucza TOML" \
"# Przed:
[trading]
max-position-size = 1000

# Po refaktoryzacji:
[trading]
max_position_size = 1000"

    echo ""
    read -p "Naciśnij Enter aby przejść do następnego demo..."
    echo ""
}

# Demo 3: Bezpieczeństwo refaktoryzacji
demo_safety() {
    show_step "3" "Bezpieczeństwo refaktoryzacji"
    
    echo -e "${BLUE}Cerberus Refactor-Kit chroni przed przypadkowymi błędami:${RESET}"
    echo ""
    
    echo -e "${GREEN}✅ Zabezpieczenia:${RESET}"
    echo "  🔍 Podgląd wszystkich zmian przed zastosowaniem"
    echo "  🧪 Automatyczne sprawdzanie kompilacji po zmianach"
    echo "  🚫 Ignorowanie wzorców bezpieczeństwa (password, secret, key)"
    echo "  📝 Ignorowanie komentarzy i stringów literalnych"
    echo "  🎯 Precyzyjne dopasowanie wzorców AST"
    echo ""
    
    echo -e "${RED}🚨 Wzorce automatycznie ignorowane:${RESET}"
    echo "  🔐 password, secret, private_key"
    echo "  💰 wallet, seed, signature"
    echo "  🔑 token, hash, api_key"
    echo ""
    
    echo -e "${YELLOW}💡 Przykład bezpiecznej refaktoryzacji:${RESET}"
    
    show_code_example "Bezpieczna zmiana" \
"// Ten kod zostanie zmieniony:
fn calculate_profit(amount: f64) -> f64 {
    amount * 0.1
}

// Ten kod zostanie ZIGNOROWANY:
const API_SECRET: &str = \"calculate_profit_key_123\";
// TODO: Improve calculate_profit function"

    echo ""
    read -p "Naciśnij Enter aby przejść do następnego demo..."
    echo ""
}

# Demo 4: Integracja z AI
demo_ai_integration() {
    show_step "4" "Integracja z AI"
    
    echo -e "${BLUE}Dla złożonych refaktoryzacji używamy AI:${RESET}"
    echo ""
    
    echo -e "${YELLOW}🤖 Przykład promptu dla AI:${RESET}"
    
    show_code_example "Prompt dla asystenta AI" \
"Projekt: Solana HFT Ninja + Cerebro
Zadanie: Rozbij funkcję evaluate w src/engine/mod.rs na mniejsze, bardziej modularne funkcje

Struktura projektu:
- src/ (Rust - HFT engine)
- cerebro/ (Python - AI brain)
- config/ (konfiguracja TOML/YAML)

Proszę o:
1. Analizę obecnego kodu
2. Konkretny plan refaktoryzacji
3. Kod po zmianach
4. Instrukcje testowania"

    echo ""
    echo -e "${GREEN}✨ Korzyści integracji z AI:${RESET}"
    echo "  🧠 Zrozumienie kontekstu biznesowego"
    echo "  🔄 Refaktoryzacja logiczna, nie tylko syntaktyczna"
    echo "  📚 Automatyczne generowanie dokumentacji"
    echo "  🧪 Sugestie testów po refaktoryzacji"
    echo ""
    
    read -p "Naciśnij Enter aby przejść do podsumowania..."
    echo ""
}

# Demo 5: Podsumowanie
demo_summary() {
    show_step "5" "Podsumowanie możliwości"
    
    echo -e "${GREEN}🎯 Cerberus Refactor-Kit oferuje:${RESET}"
    echo ""
    
    echo -e "${CYAN}🔧 Narzędzia:${RESET}"
    echo "  • ast-grep - inteligentna refaktoryzacja oparta na AST"
    echo "  • sd - nowoczesna alternatywa dla sed"
    echo "  • ripgrep - szybkie wyszukiwanie wzorców"
    echo "  • fd - szybkie znajdowanie plików"
    echo ""
    
    echo -e "${CYAN}🎯 Funkcje:${RESET}"
    echo "  • Refaktoryzacja Rust (funkcje, struktury, moduły)"
    echo "  • Refaktoryzacja Python (klasy, funkcje, zmienne)"
    echo "  • Refaktoryzacja konfiguracji (TOML, YAML, JSON)"
    echo "  • Analiza zdrowia kodu"
    echo "  • Integracja z AI dla złożonych zadań"
    echo ""
    
    echo -e "${CYAN}🛡️ Bezpieczeństwo:${RESET}"
    echo "  • Podgląd przed zastosowaniem"
    echo "  • Automatyczne sprawdzanie kompilacji"
    echo "  • Ochrona wrażliwych danych"
    echo "  • Precyzyjne dopasowanie wzorców"
    echo ""
    
    echo -e "${YELLOW}🚀 Następne kroki:${RESET}"
    echo "  1. Uruchom: ./scripts/refactor-kit/install-tools.sh"
    echo "  2. Sprawdź kod: ./scripts/refactor-kit/code-health.sh"
    echo "  3. Refaktoryzuj: ./scripts/refactor-kit/refactor.sh"
    echo ""
    
    echo -e "${GREEN}🥷 Utrzymuj swój kod w doskonałej kondycji z Cerberus Refactor-Kit!${RESET}"
}

# Główna funkcja demo
main() {
    cd "$PROJECT_ROOT"
    
    echo -e "${BLUE}🏠 Katalog projektu: ${PROJECT_ROOT}${RESET}"
    echo ""
    
    demo_code_health
    demo_patterns
    demo_safety
    demo_ai_integration
    demo_summary
    
    echo ""
    echo -e "${GREEN}🎉 Demo zakończone!${RESET}"
    echo -e "${YELLOW}💡 Przeczytaj README.md w scripts/refactor-kit/ aby dowiedzieć się więcej${RESET}"
}

# Uruchom demo
main "$@"
