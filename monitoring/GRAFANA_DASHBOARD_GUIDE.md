# 📊 Grafana Dashboard Guide - HFT Ninja 2025.07

Kompletny przewodnik po dashboardach Grafana dla systemu Solana HFT Ninja.

## 🎯 Przegląd Dashboardów

### 1. 🥷 HFT Ninja - Comprehensive Dashboard
**UID:** `hft-ninja-comprehensive`  
**Główny dashboard** z kluczowymi metrykami tradingowymi i wydajnościowymi.

#### Sekcje:
- **🎯 Trading Performance Overview**
  - 💰 Net P&L (SOL) - Real-time zyski/straty
  - 📊 Total Volume (SOL) - Całkowity wolumen
  - 🎯 Trade Success Rate - Procent udanych transakcji
  - ⚡ MEV Profit (SOL) - Zyski z MEV

- **📈 Performance Metrics**
  - ⚡ Execution Latency - Latencja wykonania (95th, 50th, 99th percentile)
  - 🔄 Mempool Latency - Latencja przetwarzania mempool

- **🎯 MEV Strategies**
  - 🎯 MEV Opportunities - Podział okazji MEV (pie chart)
  - 📦 Jito Bundles - Metryki bundli (submitted/confirmed/failed)
  - 💰 Jito Tip Amount - Aktualna kwota tip

- **🛡️ Security & System Health**
  - 🔒 Circuit Breaker - Status wyłącznika bezpieczeństwa
  - 📉 Daily Loss Ratio - Dzienny wskaźnik strat
  - 💻 System Resources - CPU, Memory, Error Rate

### 2. 🎯 MEV Strategies - Comprehensive Dashboard
**UID:** `mev-strategies-comprehensive`  
**Specjalistyczny dashboard** dla strategii MEV.

#### Sekcje:
- **🥪 Sandwich Strategy**
  - 🥪 Sandwich Opportunities - Liczba wykrytych okazji
  - 🥪 Sandwich Detection Rate - Tempo wykrywania/min

- **⚖️ Arbitrage Strategy**
  - ⚖️ Arbitrage Opportunities - Liczba wykrytych okazji
  - 💰 Total MEV Profit - Całkowity zysk MEV
  - 🔥 Liquidation Opportunities - Okazje likwidacji
  - 📦 Bundle Confirmation Time - Czas potwierdzenia bundli

## 🔧 Konfiguracja

### Automatyczne Provisioning
Dashboardy są automatycznie ładowane przez Grafana Provisioning:

```yaml
# monitoring/grafana/provisioning/dashboards/dashboards.yml
providers:
  - name: 'hft-ninja-dashboards'
    folder: 'HFT Ninja'
    path: /etc/grafana/provisioning/dashboards/hft-ninja
```

### Datasource
Prometheus skonfigurowany jako domyślne źródło danych:

```yaml
# monitoring/grafana/provisioning/datasources/prometheus.yml
datasources:
  - name: Prometheus
    type: prometheus
    url: http://prometheus:9090
    isDefault: true
```

## 📊 Kluczowe Metryki

### Trading Metrics
| Metryka | Opis | Typ | Jednostka |
|---------|------|-----|-----------|
| `hft_total_profit_sol` | Całkowity zysk w SOL | Gauge | SOL |
| `hft_total_loss_sol` | Całkowite straty w SOL | Gauge | SOL |
| `hft_total_volume_sol` | Całkowity wolumen w SOL | Gauge | SOL |
| `hft_trades_executed_total` | Liczba wykonanych transakcji | Counter | - |
| `hft_trades_successful_total` | Liczba udanych transakcji | Counter | - |
| `hft_trades_failed_total` | Liczba nieudanych transakcji | Counter | - |

### Performance Metrics
| Metryka | Opis | Typ | Jednostka |
|---------|------|-----|-----------|
| `hft_execution_latency_seconds` | Latencja wykonania | Histogram | sekundy |
| `hft_mempool_latency_seconds` | Latencja mempool | Histogram | sekundy |
| `hft_transaction_processing_time` | Czas przetwarzania transakcji | Histogram | sekundy |
| `hft_bridge_queue_size` | Rozmiar kolejki bridge | Gauge | - |

### MEV Metrics
| Metryka | Opis | Typ | Jednostka |
|---------|------|-----|-----------|
| `hft_mev_profit_sol` | Zysk MEV w SOL | Gauge | SOL |
| `hft_sandwich_opportunities_total` | Okazje sandwich | Counter | - |
| `hft_arbitrage_opportunities_total` | Okazje arbitrażu | Counter | - |
| `hft_liquidation_opportunities_total` | Okazje likwidacji | Counter | - |

### Jito Bundle Metrics
| Metryka | Opis | Typ | Jednostka |
|---------|------|-----|-----------|
| `hft_bundles_submitted_total` | Wysłane bundle | Counter | - |
| `hft_bundles_confirmed_total` | Potwierdzone bundle | Counter | - |
| `hft_bundles_failed_total` | Nieudane bundle | Counter | - |
| `hft_bundle_confirmation_seconds` | Czas potwierdzenia bundle | Histogram | sekundy |
| `hft_tip_amount_sol` | Kwota tip w SOL | Gauge | SOL |

### Security Metrics
| Metryka | Opis | Typ | Jednostka |
|---------|------|-----|-----------|
| `hft_circuit_breaker_state` | Stan circuit breaker (0=closed, 1=open, 2=half-open) | Gauge | - |
| `hft_wallet_locked` | Status blokady portfela | Gauge | - |
| `hft_daily_loss_ratio` | Dzienny wskaźnik strat | Gauge | ratio |
| `hft_position_utilization` | Wykorzystanie pozycji | Gauge | ratio |
| `hft_consecutive_failures` | Kolejne niepowodzenia | Gauge | - |

### System Metrics
| Metryka | Opis | Typ | Jednostka |
|---------|------|-----|-----------|
| `hft_memory_usage_bytes` | Użycie pamięci | Gauge | bytes |
| `hft_cpu_usage_percent` | Użycie CPU | Gauge | procent |
| `hft_active_connections` | Aktywne połączenia | Gauge | - |
| `hft_error_rate` | Wskaźnik błędów | Gauge | ratio |

## 🚀 Uruchamianie

### Szybki Start
```bash
# Uruchom enhanced monitoring stack
./monitoring/start-grafana-enhanced.sh
```

### Dostęp
- **Grafana:** http://localhost:3000
- **Login:** admin
- **Password:** hft-ninja-2025
- **Prometheus:** http://localhost:9090

### Docker Compose
```bash
# Uruchom ręcznie
cd monitoring
docker-compose -f docker-compose.grafana.yml up -d

# Sprawdź status
docker-compose -f docker-compose.grafana.yml ps

# Zatrzymaj
docker-compose -f docker-compose.grafana.yml down
```

## 🎨 Customizacja

### Dodawanie Nowych Paneli
1. Edytuj plik JSON dashboardu
2. Dodaj nowy panel z odpowiednimi queries
3. Restart Grafana lub użyj provisioning

### Przykład Nowego Panelu
```json
{
  "datasource": "Prometheus",
  "targets": [
    {
      "expr": "rate(hft_new_metric_total[5m])",
      "legendFormat": "New Metric Rate"
    }
  ],
  "title": "New Metric Panel",
  "type": "timeseries"
}
```

### Alerty
Skonfiguruj alerty dla krytycznych metryk:
- P&L poniżej progu
- Latencja powyżej 100ms
- Circuit breaker aktywny
- Wysoki error rate

## 🔧 Troubleshooting

### Brak Danych
1. Sprawdź czy HFT Ninja eksportuje metryki na porcie 8080
2. Sprawdź konfigurację Prometheus
3. Sprawdź logi kontenerów

### Dashboardy Nie Ładują Się
1. Sprawdź provisioning configuration
2. Sprawdź uprawnienia do plików
3. Sprawdź logi Grafana

### Problemy z Wydajnością
1. Zwiększ limity w Prometheus
2. Skonfiguruj retention policy
3. Optymalizuj queries

## 📚 Zaawansowane Funkcje

### Variables
Dodaj zmienne do dashboardów dla filtrowania:
- Time range
- Strategy type
- DEX selection

### Annotations
Dodaj adnotacje dla ważnych wydarzeń:
- Deployment events
- Circuit breaker activations
- Major trades

### Alerting
Skonfiguruj unified alerting dla:
- Trading anomalies
- System health issues
- Security events

---

**🥷 HFT Ninja Monitoring** - Kompletne monitorowanie systemu tradingowego na Solana!
