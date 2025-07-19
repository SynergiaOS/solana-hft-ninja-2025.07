# 🔧 Cerberus Refactor-Kit

Inteligentny system refaktoryzacji dla ekosystemu **Solana HFT Ninja + Cerebro**. Bezpieczne, oparte na AST narzędzia do utrzymania czystości i spójności kodu w całym projekcie.

## 🎯 Filozofia

Zamiast prostych operacji "znajdź i zamień", które mogą przypadkowo zmienić nazwy w komentarzach lub stringach, używamy narzędzi rozumiejących strukturę kodu. Połączenie mocy AST z inteligencją AI.

## 🛠️ Narzędzia

- **ast-grep**: Inteligentne wyszukiwanie i refaktoryzacja oparta na AST
- **sd**: Nowoczesna alternatywa dla sed
- **ripgrep**: Szybkie wyszukiwanie w plikach
- **fd**: Szybkie znajdowanie plików
- **bat**: Kolorowe wyświetlanie kodu

## 🚀 Szybki Start

### 1. Instalacja narzędzi

```bash
# Automatyczna instalacja wszystkich wymaganych narzędzi
./scripts/refactor-kit/install-tools.sh
```

### 2. Sprawdzenie zdrowia kodu

```bash
# Automatyczne wykrywanie problemów w kodzie
./scripts/refactor-kit/code-health.sh
```

### 3. Interaktywna refaktoryzacja

```bash
# Główne narzędzie refaktoryzacji
./scripts/refactor-kit/refactor.sh
```

## 📋 Funkcje

### 🦀 Refaktoryzacja Rust
- ✅ Zmiana nazw funkcji (definicje + wywołania)
- ✅ Zmiana nazw struktur (definicje + użycia)
- ✅ Zmiana nazw zmiennych
- ✅ Zmiana nazw modułów (+ aktualizacja importów)
- ✅ Zmiana nazw enum i trait
- ✅ Bezpieczne ignorowanie komentarzy i stringów

### 🐍 Refaktoryzacja Python
- ✅ Zmiana nazw klas (definicje + dziedziczenie)
- ✅ Zmiana nazw funkcji (definicje + wywołania)
- ✅ Zmiana nazw zmiennych
- ✅ Aktualizacja importów
- ✅ Sprawdzanie składni po zmianach

### ⚙️ Refaktoryzacja Konfiguracji
- ✅ Klucze TOML/YAML/JSON
- ✅ Sekcje konfiguracyjne
- ✅ Wartości w plikach .tf (Terraform)
- ✅ Podgląd zmian przed zastosowaniem

### 🔍 Analiza Kodu
- ✅ Wyszukiwanie wzorców w całym projekcie
- ✅ Wykrywanie długich funkcji (>50 linii)
- ✅ Znajdowanie TODO/FIXME
- ✅ Sprawdzanie konwencji nazewnictwa
- ✅ Analiza złożoności cyklomatycznej
- ✅ Wykrywanie potencjalnych problemów bezpieczeństwa

### 🤖 Integracja z AI
- ✅ Generowanie promptów dla złożonych refaktoryzacji
- ✅ Kontekst całego projektu
- ✅ Instrukcje testowania po zmianach

## 📖 Przykłady Użycia

### Zmiana nazwy funkcji w Rust

```bash
./scripts/refactor-kit/refactor.sh
# Wybierz: 1) Refaktoryzacja Rust
# Wybierz: a) Zmiana nazwy funkcji
# Podaj: evaluate_opportunity -> assess_market_chance
```

**Rezultat:**
- Automatycznie znajdzie wszystkie definicje `fn evaluate_opportunity`
- Zaktualizuje wszystkie wywołania `evaluate_opportunity()`
- Zignoruje wystąpienia w komentarzach i stringach
- Sprawdzi czy kod się kompiluje po zmianach

### Zmiana klucza konfiguracji

```bash
./scripts/refactor-kit/refactor.sh
# Wybierz: 3) Refaktoryzacja konfiguracji
# Podaj: jito-tip-multiplier -> jito_tip_multiplier
```

**Rezultat:**
- Znajdzie klucz we wszystkich plikach .toml, .yml, .json
- Pokaże podgląd zmian
- Zastosuje zmiany po potwierdzeniu

### Zaawansowana refaktoryzacja z AI

```bash
./scripts/refactor-kit/refactor.sh
# Wybierz: 6) Zaawansowana refaktoryzacja z AI
# Opisz: "Rozbij funkcję evaluate w src/engine/mod.rs na mniejsze, bardziej modularne funkcje"
```

**Rezultat:**
- Wygeneruje gotowy prompt dla asystenta AI
- Zawiera kontekst projektu i strukturę
- Instrukcje testowania po refaktoryzacji

## 🎯 Wzorce Refaktoryzacji

Gotowe wzorce znajdują się w `patterns.yaml`:

### Rust
```yaml
function_rename:
  pattern: "fn $OLD_NAME($$$) { $$$ }"
  rewrite: "fn $NEW_NAME($$$) { $$$ }"
```

### Python
```yaml
class_rename:
  pattern: "class $OLD_NAME:"
  rewrite: "class $NEW_NAME:"
```

### Specjalne dla projektu
```yaml
strategy_rename:
  pattern: "struct $OLD_NAMEStrategy"
  rewrite: "struct $NEW_NAMEStrategy"
```

## 🛡️ Bezpieczeństwo

### Automatyczne zabezpieczenia:
- ✅ Podgląd wszystkich zmian przed zastosowaniem
- ✅ Sprawdzanie kompilacji po zmianach Rust
- ✅ Sprawdzanie składni po zmianach Python
- ✅ Ignorowanie wzorców bezpieczeństwa (password, secret, key)
- ✅ Backup przed większymi zmianami

### Wzorce niebezpieczne (automatycznie ignorowane):
- `password`, `secret`, `private_key`
- `wallet`, `seed`, `signature`
- `token`, `hash`

## 📊 Raporty

### Raport zdrowia kodu
```bash
./scripts/refactor-kit/code-health.sh
```

Generuje raport zawierający:
- Statystyki projektu (pliki, linie, funkcje)
- Długie funkcje wymagające podziału
- TODO/FIXME do rozwiązania
- Problemy z konwencjami nazewnictwa
- Potencjalne problemy bezpieczeństwa
- Rekomendacje poprawy

### Raport refaktoryzacji
```bash
./scripts/refactor-kit/refactor.sh
# Wybierz: 5) Generuj raport refaktoryzacji
```

## 🔧 Konfiguracja

### Dostosowanie ścieżek
Edytuj zmienne w `refactor.sh`:
```bash
readonly RUST_SRC="${PROJECT_ROOT}/src"
readonly PYTHON_SRC="${PROJECT_ROOT}/cerebro"
readonly CONFIG_DIRS=("${PROJECT_ROOT}/config" "${PROJECT_ROOT}/monitoring")
```

### Dodanie własnych wzorców
Edytuj `patterns.yaml` aby dodać wzorce specyficzne dla projektu.

## 🚨 Rozwiązywanie Problemów

### Narzędzia nie są zainstalowane
```bash
# Sprawdź czy Rust jest zainstalowany
cargo --version

# Zainstaluj narzędzia
./scripts/refactor-kit/install-tools.sh
```

### Błędy kompilacji po refaktoryzacji
```bash
# Sprawdź błędy Rust
cargo check

# Sprawdź składnię Python
find cerebro -name "*.py" -exec python3 -m py_compile {} \;
```

### Cofnięcie zmian
```bash
# Użyj git do cofnięcia
git checkout -- .

# Lub przywróć z backup (jeśli został utworzony)
cp backup_YYYYMMDD_HHMMSS/* .
```

## 🤝 Współpraca

### Dodawanie nowych wzorców
1. Edytuj `patterns.yaml`
2. Dodaj testy w `code-health.sh`
3. Zaktualizuj dokumentację

### Zgłaszanie problemów
Opisz:
- Jakiej refaktoryzacji próbowałeś dokonać
- Jakie błędy wystąpiły
- Przykład kodu przed i po

## 📚 Zaawansowane Użycie

### Batch refaktoryzacja
```bash
# Użyj ast-grep bezpośrednio dla złożonych wzorców
ast-grep -p 'fn $NAME($$$) -> Result<$$$> { $$$ }' \
         -r 'async fn $NAME($$$) -> Result<$$$> { $$$ }' \
         --lang rust src/ --update-all
```

### Integracja z CI/CD
```bash
# Dodaj do pipeline
./scripts/refactor-kit/code-health.sh
if [ $? -ne 0 ]; then
  echo "Code health check failed"
  exit 1
fi
```

---

**🥷 Cerberus Refactor-Kit** - Utrzymuj swój kod w doskonałej kondycji!
