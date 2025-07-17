# 🧠 Project Cerebro - AI Assistant for Solana HFT Ninja

**Autonomiczny system wsparcia decyzji i optymalizacji strategii**

## 🎯 Przegląd

Project Cerebro to zaawansowany system AI, który przekształca Solana HFT Ninja z prostego bota wykonawczego w inteligentnego, adaptacyjnego agenta tradingowego. System wykorzystuje pamięć kontekstową, analizę danych i interakcję w języku naturalnym.

## 🏗️ Architektura

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   React UI      │    │   FastAPI BFF   │    │  DragonflyDB    │
│   Dashboard     │◄──►│   (Port 8000)   │◄──►│  (Port 6379)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Kestra        │◄──►│  LangChain      │◄──►│   Jina AI       │
│   (Port 8081)   │    │   Agent         │    │  (Port 8002)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
                    ┌─────────────────┐
                    │  HFT Ninja API  │
                    │  (Port 8080)    │
                    └─────────────────┘
```

## 🚀 Quick Start

### 1. Uruchomienie całego stacku

```bash
# Uruchom wszystkie serwisy
docker-compose up -d

# Sprawdź status
docker-compose ps

# Sprawdź logi
docker-compose logs -f
```

### 2. Dostęp do serwisów

- **BFF API**: http://localhost:8000
- **API Docs**: http://localhost:8000/docs
- **Kestra UI**: http://localhost:8081
- **Redis Insight**: http://localhost:8001
- **Jina Gateway**: http://localhost:8002

### 3. Test połączenia

```bash
# Test health check
curl http://localhost:8000/health

# Test dashboard data
curl http://localhost:8000/api/dashboard

# Test prompt (wymaga działającego Kestra)
curl -X POST http://localhost:8000/api/prompt \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Jak działa moja strategia arbitrażu?"}'
```