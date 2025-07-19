#!/bin/bash

# 🔧 Cerberus Refactor-Kit v1.0
# Inteligentny system refaktoryzacji dla ekosystemu Solana HFT Ninja + Cerebro
# Autor: Solana HFT Ninja Team
# Data: 2025-07-18

set -euo pipefail

# Kolory dla lepszej czytelności
readonly GREEN="\033[0;32m"
readonly YELLOW="\033[0;33m"
readonly CYAN="\033[0;36m"
readonly RED="\033[0;31m"
readonly BLUE="\033[0;34m"
readonly RESET="\033[0m"

# Konfiguracja ścieżek
readonly PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
readonly RUST_SRC="${PROJECT_ROOT}/src"
readonly PYTHON_SRC="${PROJECT_ROOT}/cerebro"
readonly CONFIG_DIRS=("${PROJECT_ROOT}/config" "${PROJECT_ROOT}/monitoring" "${PROJECT_ROOT}/infrastructure")

# Sprawdzenie wymaganych narzędzi
check_dependencies() {
    local missing_tools=()
    
    if ! command -v ast-grep &> /dev/null; then
        missing_tools+=("ast-grep")
    fi
    
    if ! command -v sd &> /dev/null; then
        missing_tools+=("sd")
    fi
    
    if ! command -v rg &> /dev/null; then
        missing_tools+=("ripgrep")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        echo -e "${RED}❌ Brakujące narzędzia: ${missing_tools[*]}${RESET}"
        echo -e "${YELLOW}Zainstaluj je używając:${RESET}"
        echo "cargo install ast-grep sd ripgrep"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Wszystkie wymagane narzędzia są dostępne${RESET}"
}

# Funkcja do bezpiecznej refaktoryzacji w Rust za pomocą ast-grep
refactor_rust() {
    local pattern="$1"
    local rewrite="$2"
    local description="$3"
    
    echo -e "${CYAN}🔍 ${description}${RESET}"
    echo -e "${BLUE}Wzorzec: ${pattern}${RESET}"
    echo -e "${BLUE}Zamiana: ${rewrite}${RESET}"
    echo ""
    
    # Najpierw uruchom w trybie podglądu
    echo -e "${YELLOW}📋 Podgląd zmian:${RESET}"
    if ast-grep -p "${pattern}" -r "${rewrite}" --lang rust "${RUST_SRC}" 2>/dev/null; then
        echo ""
        read -p "Czy chcesz zastosować te zmiany? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            # Zastosuj zmiany
            ast-grep -p "${pattern}" -r "${rewrite}" --lang rust "${RUST_SRC}" --update-all
            echo -e "${GREEN}✅ Zmiany w kodzie Rust zostały zastosowane.${RESET}"
            
            # Uruchom cargo check aby sprawdzić poprawność
            echo -e "${YELLOW}🔧 Sprawdzanie poprawności kodu...${RESET}"
            if (cd "${PROJECT_ROOT}" && cargo check --quiet); then
                echo -e "${GREEN}✅ Kod kompiluje się poprawnie${RESET}"
            else
                echo -e "${RED}❌ Błędy kompilacji! Sprawdź kod ręcznie.${RESET}"
            fi
        else
            echo -e "${YELLOW}❌ Anulowano.${RESET}"
        fi
    else
        echo -e "${YELLOW}ℹ️ Nie znaleziono pasujących wzorców.${RESET}"
    fi
}

# Funkcja do refaktoryzacji w Pythonie za pomocą ast-grep
refactor_python() {
    local pattern="$1"
    local rewrite="$2"
    local description="$3"
    
    echo -e "${CYAN}🔍 ${description}${RESET}"
    echo -e "${BLUE}Wzorzec: ${pattern}${RESET}"
    echo -e "${BLUE}Zamiana: ${rewrite}${RESET}"
    echo ""
    
    echo -e "${YELLOW}📋 Podgląd zmian:${RESET}"
    if ast-grep -p "${pattern}" -r "${rewrite}" --lang python "${PYTHON_SRC}" 2>/dev/null; then
        echo ""
        read -p "Czy chcesz zastosować te zmiany? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            ast-grep -p "${pattern}" -r "${rewrite}" --lang python "${PYTHON_SRC}" --update-all
            echo -e "${GREEN}✅ Zmiany w kodzie Pythona zostały zastosowane.${RESET}"
            
            # Sprawdź składnię Pythona
            echo -e "${YELLOW}🔧 Sprawdzanie składni Pythona...${RESET}"
            if find "${PYTHON_SRC}" -name "*.py" -exec python3 -m py_compile {} \; 2>/dev/null; then
                echo -e "${GREEN}✅ Składnia Pythona jest poprawna${RESET}"
            else
                echo -e "${RED}❌ Błędy składni! Sprawdź kod ręcznie.${RESET}"
            fi
        else
            echo -e "${YELLOW}❌ Anulowano.${RESET}"
        fi
    else
        echo -e "${YELLOW}ℹ️ Nie znaleziono pasujących wzorców.${RESET}"
    fi
}

# Funkcja do prostej zamiany w plikach konfiguracyjnych
refactor_config() {
    local old_string="$1"
    local new_string="$2"
    local description="$3"
    
    echo -e "${CYAN}🔍 ${description}${RESET}"
    echo -e "${BLUE}Szukanie: ${old_string}${RESET}"
    echo -e "${BLUE}Zamiana: ${new_string}${RESET}"
    echo ""
    
    # Znajdź pliki konfiguracyjne
    local config_files=()
    for dir in "${CONFIG_DIRS[@]}"; do
        if [ -d "$dir" ]; then
            while IFS= read -r -d '' file; do
                config_files+=("$file")
            done < <(find "$dir" -type f \( -name "*.toml" -o -name "*.yml" -o -name "*.yaml" -o -name "*.tf" -o -name "*.json" \) -print0)
        fi
    done
    
    if [ ${#config_files[@]} -eq 0 ]; then
        echo -e "${YELLOW}ℹ️ Nie znaleziono plików konfiguracyjnych.${RESET}"
        return
    fi
    
    # Pokaż podgląd zmian
    echo -e "${YELLOW}📋 Pliki do zmiany:${RESET}"
    local found_matches=false
    for file in "${config_files[@]}"; do
        if grep -l "$old_string" "$file" 2>/dev/null; then
            echo "  📄 $file"
            grep -n --color=always "$old_string" "$file" | head -3
            found_matches=true
        fi
    done
    
    if [ "$found_matches" = false ]; then
        echo -e "${YELLOW}ℹ️ Nie znaleziono pasujących wzorców.${RESET}"
        return
    fi
    
    echo ""
    read -p "Czy chcesz zastosować te zmiany? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        for file in "${config_files[@]}"; do
            sd "$old_string" "$new_string" "$file" 2>/dev/null || true
        done
        echo -e "${GREEN}✅ Zmiany w plikach konfiguracyjnych zostały zastosowane.${RESET}"
    else
        echo -e "${YELLOW}❌ Anulowano.${RESET}"
    fi
}

# Funkcja do wyszukiwania wzorców w kodzie
search_patterns() {
    local search_term="$1"
    local file_types="$2"
    
    echo -e "${CYAN}🔍 Wyszukiwanie wzorca: ${search_term}${RESET}"
    echo -e "${BLUE}Typy plików: ${file_types}${RESET}"
    echo ""
    
    case "$file_types" in
        "rust")
            rg "$search_term" --type rust "${RUST_SRC}" -n --color=always
            ;;
        "python")
            rg "$search_term" --type python "${PYTHON_SRC}" -n --color=always
            ;;
        "config")
            for dir in "${CONFIG_DIRS[@]}"; do
                if [ -d "$dir" ]; then
                    rg "$search_term" --type-add 'config:*.{toml,yml,yaml,json,tf}' --type config "$dir" -n --color=always
                fi
            done
            ;;
        "all")
            rg "$search_term" "${PROJECT_ROOT}" -n --color=always \
                --type rust --type python \
                --type-add 'config:*.{toml,yml,yaml,json,tf}' --type config
            ;;
    esac
}

# Funkcja do generowania raportu refaktoryzacji
generate_report() {
    local report_file="${PROJECT_ROOT}/refactor_report_$(date +%Y%m%d_%H%M%S).md"
    
    cat > "$report_file" << EOF
# Raport Refaktoryzacji Cerberus
**Data:** $(date)
**Projekt:** Solana HFT Ninja + Cerebro

## Statystyki Kodu

### Rust (HFT Ninja)
- Pliki .rs: $(find "${RUST_SRC}" -name "*.rs" | wc -l)
- Linie kodu: $(find "${RUST_SRC}" -name "*.rs" -exec cat {} \; | wc -l)
- Funkcje: $(rg "^fn " --type rust "${RUST_SRC}" | wc -l)
- Struktury: $(rg "^struct " --type rust "${RUST_SRC}" | wc -l)

### Python (Cerebro)
- Pliki .py: $(find "${PYTHON_SRC}" -name "*.py" | wc -l)
- Linie kodu: $(find "${PYTHON_SRC}" -name "*.py" -exec cat {} \; | wc -l)
- Klasy: $(rg "^class " --type python "${PYTHON_SRC}" | wc -l)
- Funkcje: $(rg "^def " --type python "${PYTHON_SRC}" | wc -l)

### Konfiguracja
- Pliki TOML: $(find "${PROJECT_ROOT}" -name "*.toml" | wc -l)
- Pliki YAML: $(find "${PROJECT_ROOT}" -name "*.yml" -o -name "*.yaml" | wc -l)
- Pliki JSON: $(find "${PROJECT_ROOT}" -name "*.json" | wc -l)

## Rekomendacje

### Potencjalne Problemy
EOF

    # Znajdź potencjalne problemy
    echo "### TODO i FIXME" >> "$report_file"
    rg "TODO|FIXME|XXX|HACK" "${PROJECT_ROOT}" -n >> "$report_file" 2>/dev/null || echo "Brak znalezionych TODO/FIXME" >> "$report_file"
    
    echo "" >> "$report_file"
    echo "### Długie funkcje (>50 linii)" >> "$report_file"
    # Znajdź długie funkcje w Rust
    ast-grep -p 'fn $NAME($$$) { $$$ }' --lang rust "${RUST_SRC}" 2>/dev/null | \
        awk '/^fn / {start=NR} /^}$/ {if(NR-start>50) print FILENAME":"start":"$0}' >> "$report_file" 2>/dev/null || true
    
    echo -e "${GREEN}✅ Raport wygenerowany: ${report_file}${RESET}"
}

# Główne menu
show_menu() {
    echo -e "${GREEN}=====================================${RESET}"
    echo -e "${GREEN}==  🔧 Cerberus Refactor-Kit v1.0 ==${RESET}"
    echo -e "${GREEN}=====================================${RESET}"
    echo ""
    echo "Wybierz opcję:"
    echo "1) 🦀 Refaktoryzacja Rust (funkcje, struktury, zmienne)"
    echo "2) 🐍 Refaktoryzacja Python (klasy, funkcje, zmienne)"
    echo "3) ⚙️  Refaktoryzacja konfiguracji (klucze, wartości)"
    echo "4) 🔍 Wyszukiwanie wzorców w kodzie"
    echo "5) 📊 Generuj raport refaktoryzacji"
    echo "6) 🤖 Zaawansowana refaktoryzacja z AI"
    echo "7) 🚪 Wyjście"
    echo ""
}

# Główna pętla programu
main() {
    cd "$PROJECT_ROOT"
    
    echo -e "${BLUE}🏠 Katalog projektu: ${PROJECT_ROOT}${RESET}"
    echo ""
    
    check_dependencies
    echo ""
    
    while true; do
        show_menu
        read -p "Twój wybór (1-7): " choice
        echo ""
        
        case $choice in
            1)
                echo "Opcje refaktoryzacji Rust:"
                echo "a) Zmiana nazwy funkcji"
                echo "b) Zmiana nazwy struktury"
                echo "c) Zmiana nazwy zmiennej"
                echo "d) Zmiana nazwy modułu"
                read -p "Wybierz (a-d): " rust_choice
                
                case $rust_choice in
                    a)
                        read -p "Stara nazwa funkcji: " old_name
                        read -p "Nowa nazwa funkcji: " new_name
                        refactor_rust "fn ${old_name}(\$\$\$) { \$\$\$ }" "fn ${new_name}(\$\$\$) { \$\$\$ }" "Zmiana nazwy funkcji: ${old_name} → ${new_name}"
                        refactor_rust "${old_name}(\$\$\$)" "${new_name}(\$\$\$)" "Aktualizacja wywołań funkcji: ${old_name} → ${new_name}"
                        ;;
                    b)
                        read -p "Stara nazwa struktury: " old_name
                        read -p "Nowa nazwa struktury: " new_name
                        refactor_rust "struct ${old_name} { \$\$\$ }" "struct ${new_name} { \$\$\$ }" "Zmiana nazwy struktury: ${old_name} → ${new_name}"
                        refactor_rust "${old_name}" "${new_name}" "Aktualizacja użyć struktury: ${old_name} → ${new_name}"
                        ;;
                    c)
                        read -p "Stara nazwa zmiennej: " old_name
                        read -p "Nowa nazwa zmiennej: " new_name
                        refactor_rust "let ${old_name}" "let ${new_name}" "Zmiana nazwy zmiennej: ${old_name} → ${new_name}"
                        ;;
                    d)
                        read -p "Stara nazwa modułu: " old_name
                        read -p "Nowa nazwa modułu: " new_name
                        refactor_rust "mod ${old_name}" "mod ${new_name}" "Zmiana nazwy modułu: ${old_name} → ${new_name}"
                        refactor_rust "use.*${old_name}" "use.*${new_name}" "Aktualizacja importów modułu: ${old_name} → ${new_name}"
                        ;;
                esac
                ;;
            2)
                echo "Opcje refaktoryzacji Python:"
                echo "a) Zmiana nazwy klasy"
                echo "b) Zmiana nazwy funkcji"
                echo "c) Zmiana nazwy zmiennej"
                read -p "Wybierz (a-c): " python_choice
                
                case $python_choice in
                    a)
                        read -p "Stara nazwa klasy: " old_name
                        read -p "Nowa nazwa klasy: " new_name
                        refactor_python "class ${old_name}:" "class ${new_name}:" "Zmiana nazwy klasy: ${old_name} → ${new_name}"
                        refactor_python "${old_name}()" "${new_name}()" "Aktualizacja instancji klasy: ${old_name} → ${new_name}"
                        ;;
                    b)
                        read -p "Stara nazwa funkcji: " old_name
                        read -p "Nowa nazwa funkcji: " new_name
                        refactor_python "def ${old_name}(\$\$\$):" "def ${new_name}(\$\$\$):" "Zmiana nazwy funkcji: ${old_name} → ${new_name}"
                        refactor_python "${old_name}(\$\$\$)" "${new_name}(\$\$\$)" "Aktualizacja wywołań funkcji: ${old_name} → ${new_name}"
                        ;;
                    c)
                        read -p "Stara nazwa zmiennej: " old_name
                        read -p "Nowa nazwa zmiennej: " new_name
                        refactor_python "${old_name} =" "${new_name} =" "Zmiana nazwy zmiennej: ${old_name} → ${new_name}"
                        ;;
                esac
                ;;
            3)
                read -p "Stary klucz/wartość: " old_key
                read -p "Nowy klucz/wartość: " new_key
                refactor_config "$old_key" "$new_key" "Zmiana w konfiguracji: ${old_key} → ${new_key}"
                ;;
            4)
                read -p "Wzorzec do wyszukania: " pattern
                echo "Gdzie szukać?"
                echo "a) Tylko Rust"
                echo "b) Tylko Python"
                echo "c) Tylko konfiguracja"
                echo "d) Wszędzie"
                read -p "Wybierz (a-d): " search_choice
                
                case $search_choice in
                    a) search_patterns "$pattern" "rust" ;;
                    b) search_patterns "$pattern" "python" ;;
                    c) search_patterns "$pattern" "config" ;;
                    d) search_patterns "$pattern" "all" ;;
                esac
                ;;
            5)
                generate_report
                ;;
            6)
                echo -e "${YELLOW}🤖 Zaawansowana refaktoryzacja z AI${RESET}"
                echo ""
                echo "Opisz, co chcesz zrefaktoryzować (np. 'Rozbij funkcję evaluate na mniejsze części'):"
                read -p "> " user_prompt
                echo ""
                echo -e "${CYAN}📋 Skopiuj i wklej poniższy prompt do swojego asystenta AI:${RESET}"
                echo "--------------------------------------------------------"
                echo "Projekt: Solana HFT Ninja + Cerebro"
                echo "Zadanie refaktoryzacji: ${user_prompt}"
                echo ""
                echo "Struktura projektu:"
                echo "- src/ (Rust - HFT engine)"
                echo "- cerebro/ (Python - AI brain)"
                echo "- config/ (konfiguracja TOML/YAML)"
                echo "- monitoring/ (Prometheus/Grafana)"
                echo ""
                echo "Proszę o:"
                echo "1. Analizę obecnego kodu"
                echo "2. Konkretny plan refaktoryzacji"
                echo "3. Kod po zmianach"
                echo "4. Instrukcje testowania"
                echo "--------------------------------------------------------"
                ;;
            7)
                echo -e "${GREEN}👋 Do widzenia!${RESET}"
                exit 0
                ;;
            *)
                echo -e "${RED}❌ Nieprawidłowa opcja. Spróbuj ponownie.${RESET}"
                ;;
        esac
        
        echo ""
        read -p "Naciśnij Enter aby kontynuować..."
        echo ""
    done
}

# Uruchom główną funkcję
main "$@"
