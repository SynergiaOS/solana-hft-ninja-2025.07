#!/bin/bash

# 🏥 Code Health Checker dla Cerberus
# Automatyczne wykrywanie problemów w kodzie i sugestie refaktoryzacji

set -euo pipefail

readonly GREEN="\033[0;32m"
readonly YELLOW="\033[0;33m"
readonly CYAN="\033[0;36m"
readonly RED="\033[0;31m"
readonly BLUE="\033[0;34m"
readonly RESET="\033[0m"

readonly PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
readonly RUST_SRC="${PROJECT_ROOT}/src"
readonly PYTHON_SRC="${PROJECT_ROOT}/cerebro"

# Sprawdź długie funkcje w Rust
check_long_rust_functions() {
    echo -e "${CYAN}🦀 Sprawdzanie długich funkcji Rust (>50 linii)...${RESET}"
    
    local long_functions=0
    
    while IFS= read -r -d '' file; do
        if [ -f "$file" ]; then
            # Użyj ast-grep do znajdowania funkcji
            ast-grep -p 'fn $NAME($$$) { $$$ }' --lang rust "$file" 2>/dev/null | \
            while read -r line; do
                if [[ "$line" =~ ^fn\ ([a-zA-Z_][a-zA-Z0-9_]*) ]]; then
                    local func_name="${BASH_REMATCH[1]}"
                    local start_line=$(grep -n "fn $func_name" "$file" | head -1 | cut -d: -f1)
                    
                    # Znajdź koniec funkcji (uproszczone)
                    local brace_count=0
                    local end_line=$start_line
                    local in_function=false
                    
                    while IFS= read -r code_line; do
                        end_line=$((end_line + 1))
                        
                        if [[ "$code_line" =~ \{ ]]; then
                            brace_count=$((brace_count + 1))
                            in_function=true
                        fi
                        
                        if [[ "$code_line" =~ \} ]] && [ "$in_function" = true ]; then
                            brace_count=$((brace_count - 1))
                            if [ $brace_count -eq 0 ]; then
                                break
                            fi
                        fi
                    done < <(tail -n +$start_line "$file")
                    
                    local func_length=$((end_line - start_line))
                    
                    if [ $func_length -gt 50 ]; then
                        echo -e "${YELLOW}⚠️  ${file}:${start_line} - funkcja '${func_name}' ma ${func_length} linii${RESET}"
                        long_functions=$((long_functions + 1))
                    fi
                fi
            done
        fi
    done < <(find "$RUST_SRC" -name "*.rs" -print0 2>/dev/null)
    
    if [ $long_functions -eq 0 ]; then
        echo -e "${GREEN}✅ Wszystkie funkcje Rust mają odpowiednią długość${RESET}"
    else
        echo -e "${RED}❌ Znaleziono ${long_functions} długich funkcji${RESET}"
    fi
}

# Sprawdź długie funkcje w Pythonie
check_long_python_functions() {
    echo -e "${CYAN}🐍 Sprawdzanie długich funkcji Python (>50 linii)...${RESET}"
    
    local long_functions=0
    
    find "$PYTHON_SRC" -name "*.py" -type f 2>/dev/null | while read -r file; do
        if [ -f "$file" ]; then
            # Znajdź definicje funkcji i klas
            grep -n "^def \|^class " "$file" | while IFS=: read -r line_num definition; do
                local func_name=$(echo "$definition" | sed 's/^def \|^class \|\(.*\):.*/\1/' | cut -d'(' -f1)
                local start_line=$line_num
                
                # Znajdź następną definicję lub koniec pliku
                local next_def=$(grep -n "^def \|^class " "$file" | awk -F: -v start="$start_line" '$1 > start {print $1; exit}')
                local end_line=${next_def:-$(wc -l < "$file")}
                
                local func_length=$((end_line - start_line))
                
                if [ $func_length -gt 50 ]; then
                    echo -e "${YELLOW}⚠️  ${file}:${start_line} - ${definition%:} ma ${func_length} linii${RESET}"
                    long_functions=$((long_functions + 1))
                fi
            done
        fi
    done
    
    if [ $long_functions -eq 0 ]; then
        echo -e "${GREEN}✅ Wszystkie funkcje Python mają odpowiednią długość${RESET}"
    fi
}

# Sprawdź TODO i FIXME
check_todos() {
    echo -e "${CYAN}📝 Sprawdzanie TODO, FIXME, XXX...${RESET}"
    
    local todo_count=0
    
    if command -v rg &> /dev/null; then
        local todos=$(rg "TODO|FIXME|XXX|HACK" "$PROJECT_ROOT" -n --color=never 2>/dev/null || true)
        
        if [ -n "$todos" ]; then
            echo "$todos" | while read -r line; do
                echo -e "${YELLOW}📌 $line${RESET}"
                todo_count=$((todo_count + 1))
            done
            echo -e "${YELLOW}Znaleziono TODO/FIXME do rozwiązania${RESET}"
        else
            echo -e "${GREEN}✅ Brak nierozwiązanych TODO/FIXME${RESET}"
        fi
    fi
}

# Sprawdź duplikaty kodu
check_duplicates() {
    echo -e "${CYAN}🔍 Sprawdzanie potencjalnych duplikatów...${RESET}"
    
    # Znajdź podobne nazwy funkcji
    echo -e "${BLUE}Podobne nazwy funkcji w Rust:${RESET}"
    if [ -d "$RUST_SRC" ]; then
        rg "^fn " --type rust "$RUST_SRC" -o --no-filename 2>/dev/null | \
        sed 's/fn \([a-zA-Z_][a-zA-Z0-9_]*\).*/\1/' | \
        sort | uniq -c | sort -nr | \
        awk '$1 > 1 {print "  🔄 " $2 " (" $1 " wystąpień)"}'
    fi
    
    echo -e "${BLUE}Podobne nazwy funkcji w Python:${RESET}"
    if [ -d "$PYTHON_SRC" ]; then
        rg "^def " --type python "$PYTHON_SRC" -o --no-filename 2>/dev/null | \
        sed 's/def \([a-zA-Z_][a-zA-Z0-9_]*\).*/\1/' | \
        sort | uniq -c | sort -nr | \
        awk '$1 > 1 {print "  🔄 " $2 " (" $1 " wystąpień)"}'
    fi
}

# Sprawdź złożoność cyklomatyczną (uproszczone)
check_complexity() {
    echo -e "${CYAN}🧮 Sprawdzanie złożoności kodu...${RESET}"
    
    echo -e "${BLUE}Funkcje z wysoką złożonością (>10 if/match/loop):${RESET}"
    
    # Rust
    if [ -d "$RUST_SRC" ]; then
        find "$RUST_SRC" -name "*.rs" -type f | while read -r file; do
            # Policz if, match, for, while w każdej funkcji
            local in_function=false
            local function_name=""
            local complexity=0
            local line_num=0
            
            while IFS= read -r line; do
                line_num=$((line_num + 1))
                
                if [[ "$line" =~ ^[[:space:]]*fn[[:space:]]+([a-zA-Z_][a-zA-Z0-9_]*) ]]; then
                    if [ "$in_function" = true ] && [ $complexity -gt 10 ]; then
                        echo -e "${YELLOW}⚠️  ${file} - funkcja '${function_name}' ma złożoność ${complexity}${RESET}"
                    fi
                    
                    function_name="${BASH_REMATCH[1]}"
                    in_function=true
                    complexity=1
                elif [ "$in_function" = true ]; then
                    # Policz konstrukcje zwiększające złożoność
                    if [[ "$line" =~ (if|match|for|while|loop) ]]; then
                        complexity=$((complexity + 1))
                    fi
                    
                    # Sprawdź czy kończy się funkcja (uproszczone)
                    if [[ "$line" =~ ^[[:space:]]*\}[[:space:]]*$ ]]; then
                        if [ $complexity -gt 10 ]; then
                            echo -e "${YELLOW}⚠️  ${file}:${line_num} - funkcja '${function_name}' ma złożoność ${complexity}${RESET}"
                        fi
                        in_function=false
                    fi
                fi
            done < "$file"
        done
    fi
}

# Sprawdź konwencje nazewnictwa
check_naming_conventions() {
    echo -e "${CYAN}📏 Sprawdzanie konwencji nazewnictwa...${RESET}"
    
    # Rust - snake_case dla funkcji i zmiennych
    echo -e "${BLUE}Rust - sprawdzanie snake_case:${RESET}"
    if [ -d "$RUST_SRC" ]; then
        rg "fn [a-zA-Z]*[A-Z]" --type rust "$RUST_SRC" -n 2>/dev/null | head -5 | while read -r line; do
            echo -e "${YELLOW}⚠️  $line - funkcja powinna używać snake_case${RESET}"
        done
    fi
    
    # Python - snake_case dla funkcji, PascalCase dla klas
    echo -e "${BLUE}Python - sprawdzanie konwencji:${RESET}"
    if [ -d "$PYTHON_SRC" ]; then
        # Funkcje powinny być snake_case
        rg "def [a-zA-Z]*[A-Z]" --type python "$PYTHON_SRC" -n 2>/dev/null | head -5 | while read -r line; do
            echo -e "${YELLOW}⚠️  $line - funkcja powinna używać snake_case${RESET}"
        done
        
        # Klasy powinny być PascalCase
        rg "class [a-z]" --type python "$PYTHON_SRC" -n 2>/dev/null | head -5 | while read -r line; do
            echo -e "${YELLOW}⚠️  $line - klasa powinna używać PascalCase${RESET}"
        done
    fi
}

# Sprawdź bezpieczeństwo
check_security() {
    echo -e "${CYAN}🔒 Sprawdzanie potencjalnych problemów bezpieczeństwa...${RESET}"
    
    # Szukaj potencjalnie niebezpiecznych wzorców
    local security_patterns=(
        "unwrap()"
        "expect("
        "panic!"
        "unsafe"
        "transmute"
        "password.*="
        "secret.*="
        "private_key.*="
    )
    
    for pattern in "${security_patterns[@]}"; do
        local matches=$(rg "$pattern" "$PROJECT_ROOT" -n --color=never 2>/dev/null || true)
        if [ -n "$matches" ]; then
            echo -e "${RED}🚨 Potencjalny problem: ${pattern}${RESET}"
            echo "$matches" | head -3 | while read -r line; do
                echo -e "${YELLOW}  $line${RESET}"
            done
        fi
    done
}

# Generuj raport zdrowia kodu
generate_health_report() {
    local report_file="${PROJECT_ROOT}/code_health_$(date +%Y%m%d_%H%M%S).md"
    
    cat > "$report_file" << EOF
# 🏥 Raport Zdrowia Kodu - Cerberus
**Data:** $(date)
**Projekt:** Solana HFT Ninja + Cerebro

## Statystyki

### Rust (HFT Ninja)
- Pliki: $(find "${RUST_SRC}" -name "*.rs" 2>/dev/null | wc -l)
- Linie kodu: $(find "${RUST_SRC}" -name "*.rs" -exec cat {} \; 2>/dev/null | wc -l)
- Funkcje: $(rg "^fn " --type rust "${RUST_SRC}" 2>/dev/null | wc -l)

### Python (Cerebro)
- Pliki: $(find "${PYTHON_SRC}" -name "*.py" 2>/dev/null | wc -l)
- Linie kodu: $(find "${PYTHON_SRC}" -name "*.py" -exec cat {} \; 2>/dev/null | wc -l)
- Funkcje: $(rg "^def " --type python "${PYTHON_SRC}" 2>/dev/null | wc -l)

## Problemy do Rozwiązania

EOF

    # Dodaj wyniki sprawdzeń do raportu
    {
        echo "### TODO i FIXME"
        rg "TODO|FIXME|XXX|HACK" "$PROJECT_ROOT" -n 2>/dev/null || echo "Brak"
        
        echo ""
        echo "### Długie funkcje"
        echo "Sprawdź funkcje >50 linii i rozważ ich podział"
        
        echo ""
        echo "### Rekomendacje"
        echo "- Regularnie uruchamiaj cargo clippy dla Rust"
        echo "- Używaj black/flake8 dla Python"
        echo "- Dodaj więcej testów jednostkowych"
        echo "- Dokumentuj publiczne API"
    } >> "$report_file"
    
    echo -e "${GREEN}✅ Raport wygenerowany: ${report_file}${RESET}"
}

# Główna funkcja
main() {
    cd "$PROJECT_ROOT"
    
    echo -e "${GREEN}=====================================${RESET}"
    echo -e "${GREEN}==  🏥 Code Health Checker v1.0  ==${RESET}"
    echo -e "${GREEN}=====================================${RESET}"
    echo ""
    
    check_long_rust_functions
    echo ""
    
    check_long_python_functions
    echo ""
    
    check_todos
    echo ""
    
    check_duplicates
    echo ""
    
    check_complexity
    echo ""
    
    check_naming_conventions
    echo ""
    
    check_security
    echo ""
    
    echo -e "${CYAN}📊 Generowanie raportu...${RESET}"
    generate_health_report
    
    echo ""
    echo -e "${GREEN}🎉 Sprawdzenie zakończone!${RESET}"
    echo -e "${YELLOW}💡 Uruchom './scripts/refactor-kit/refactor.sh' aby naprawić znalezione problemy${RESET}"
}

# Uruchom sprawdzenie
main "$@"
