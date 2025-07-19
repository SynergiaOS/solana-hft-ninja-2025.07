#!/bin/bash

# 🛠️ Instalator narzędzi dla Cerberus Refactor-Kit
# Automatycznie instaluje wszystkie wymagane narzędzia

set -euo pipefail

readonly GREEN="\033[0;32m"
readonly YELLOW="\033[0;33m"
readonly CYAN="\033[0;36m"
readonly RED="\033[0;31m"
readonly RESET="\033[0m"

echo -e "${CYAN}🛠️ Instalator Cerberus Refactor-Kit${RESET}"
echo ""

# Sprawdź czy Rust jest zainstalowany
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Rust/Cargo nie jest zainstalowany${RESET}"
    echo -e "${YELLOW}Zainstaluj Rust z: https://rustup.rs/${RESET}"
    exit 1
fi

echo -e "${GREEN}✅ Rust/Cargo jest dostępny${RESET}"

# Lista narzędzi do zainstalowania
declare -A tools=(
    ["ast-grep"]="Inteligentne wyszukiwanie i refaktoryzacja kodu"
    ["sd"]="Nowoczesna alternatywa dla sed"
    ["ripgrep"]="Szybkie wyszukiwanie w plikach"
    ["fd-find"]="Szybkie znajdowanie plików"
    ["bat"]="Kolorowe wyświetlanie plików"
)

echo ""
echo -e "${CYAN}📦 Instalowanie narzędzi...${RESET}"

for tool in "${!tools[@]}"; do
    echo ""
    echo -e "${YELLOW}🔧 Sprawdzanie: ${tool}${RESET}"
    
    if command -v "$tool" &> /dev/null; then
        echo -e "${GREEN}✅ ${tool} już jest zainstalowany${RESET}"
        continue
    fi
    
    echo -e "${CYAN}📥 Instalowanie ${tool}: ${tools[$tool]}${RESET}"
    
    case "$tool" in
        "ast-grep")
            cargo install ast-grep --locked
            ;;
        "sd")
            cargo install sd --locked
            ;;
        "ripgrep")
            cargo install ripgrep --locked
            ;;
        "fd-find")
            cargo install fd-find --locked
            ;;
        "bat")
            cargo install bat --locked
            ;;
    esac
    
    if command -v "$tool" &> /dev/null; then
        echo -e "${GREEN}✅ ${tool} zainstalowany pomyślnie${RESET}"
    else
        echo -e "${RED}❌ Błąd instalacji ${tool}${RESET}"
    fi
done

echo ""
echo -e "${GREEN}🎉 Instalacja zakończona!${RESET}"
echo ""
echo -e "${CYAN}📋 Sprawdzenie wersji:${RESET}"

for tool in "${!tools[@]}"; do
    if command -v "$tool" &> /dev/null; then
        case "$tool" in
            "ast-grep")
                echo "  🦀 ast-grep: $(ast-grep --version 2>/dev/null || echo 'zainstalowany')"
                ;;
            "sd")
                echo "  🔄 sd: $(sd --version 2>/dev/null || echo 'zainstalowany')"
                ;;
            "ripgrep")
                echo "  🔍 ripgrep: $(rg --version | head -1 2>/dev/null || echo 'zainstalowany')"
                ;;
            "fd-find")
                echo "  📁 fd: $(fd --version 2>/dev/null || echo 'zainstalowany')"
                ;;
            "bat")
                echo "  🦇 bat: $(bat --version 2>/dev/null || echo 'zainstalowany')"
                ;;
        esac
    fi
done

echo ""
echo -e "${GREEN}🚀 Gotowe! Możesz teraz uruchomić:${RESET}"
echo -e "${CYAN}  ./scripts/refactor-kit/refactor.sh${RESET}"
