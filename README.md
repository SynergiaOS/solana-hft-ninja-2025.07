# 🥷 SOLANA-HFT-NINJA 2025.07

**Zero-cost Solana High-Frequency Trading Engine**

[![Rust](https://img.shields.io/badge/Rust-1.79+-orange.svg)](https://www.rust-lang.org/)
[![Solana](https://img.shields.io/badge/Solana-1.18+-blue.svg)](https://solana.com/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 🎯 Overview

SOLANA-HFT-NINJA 2025.07 is a cutting-edge, zero-cost high-frequency trading engine built specifically for the Solana blockchain. Engineered for maximum performance and minimal latency, it leverages Rust's zero-cost abstractions and Solana's high-throughput architecture to deliver institutional-grade trading capabilities.

### ✨ Key Features

- **Zero-Cost Architecture**: Built with Rust's zero-cost abstractions for maximum performance
- **Ultra-Low Latency**: Sub-millisecond order execution with optimized networking
- **Multi-Strategy Support**: Market making, arbitrage, and custom strategy plugins
- **Advanced Risk Management**: Real-time position monitoring and automated risk controls
- **Comprehensive Monitoring**: Prometheus metrics + Grafana dashboards
- **Docker-Ready**: One-command deployment with Docker Compose
- **Production-Grade**: Battle-tested in live trading environments

## 🚀 Quick Start

### Prerequisites

- **Docker** & **Docker Compose**
- **Rust 1.79+** (for development)
- **Solana CLI** (for wallet management)

### 1. Clone & Setup

```bash
git clone https://github.com/hftninja/solana-hft-ninja-2025.07.git
cd solana-hft-ninja-2025.07
chmod +x scripts/*.sh
./scripts/setup.sh
```

### 2. Configure Secrets (Choose One)

#### Option A: Infisical (Recommended for Production) 🔐

```bash
# Setup Infisical secrets management
./scripts/infisical-setup.sh

# Validate configuration
./scripts/validate-infisical.sh

# Run with Infisical
docker-compose -f docker-compose.yml -f docker-compose.infisical.yml up -d
```

**Project ID**: `73c2f3cb-c922-4a46-a333-7b96fbc6301a`

See [Infisical Setup Guide](docs/INFISICAL_SETUP.md) for detailed instructions.

#### Option B: Local .env File

Edit `.env` file with your settings:

```bash
# Solana RPC Configuration
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_WS_URL=wss://api.mainnet-beta.solana.com

# Wallet Configuration
PRIVATE_KEY_PATH=./config/wallet.json
```

### 3. Deploy

#### Option A: Oracle Cloud (Recommended for Production) 🌐

```bash
# Quick Oracle Cloud deployment to 10.0.0.59
scp -r . opc@10.0.0.59:/opt/solana-hft-ninja/
ssh opc@10.0.0.59
cd /opt/solana-hft-ninja
./scripts/deploy-oracle-cloud.sh

# Access dashboard
# http://10.0.0.59:8080/health
# http://ninja.subnet07161247.vcn07161247.oraclevcn.com:8080
```

See [Oracle Cloud Deployment Guide](docs/ORACLE_CLOUD_DEPLOYMENT.md) for detailed instructions.

#### Option B: Local Development

```bash
# Simple start
./scripts/start.sh

# With Infisical secrets
./scripts/run-with-infisical.sh

# Docker deployment
docker-compose up -d

# Full monitoring stack
docker-compose -f docker-compose.yml -f docker-compose.monitoring.yml up -d
```

## 📊 Architecture

```
solana-hft-ninja-2025.07/
├── src/
│   ├── bin/main.rs          # Entry point
│   ├── config/              # Configuration management
│   ├── core/                # Core trading components
│   ├── engine/              # Trading engine
│   ├── market/              # Market data handling
│   ├── strategy/            # Trading strategies
│   ├── types/               # Type definitions
│   └── utils/               # Utilities
├── scripts/                 # Deployment scripts
├── config/                  # Configuration files
├── tests/                   # Test suites
└── docs/                    # Documentation
```

## 🛠️ Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SOLANA_RPC_URL` | Solana RPC endpoint | `https://api.mainnet-beta.solana.com` |
| `PRIVATE_KEY_PATH` | Wallet private key file | `./config/wallet.json` |
| `STRATEGY_MODE` | Trading strategy mode | `market_making` |
| `MAX_SLIPPAGE_BPS` | Maximum slippage tolerance | `50` |
| `RISK_LIMIT_BPS` | Risk limit in basis points | `100` |

### Strategy Configuration

```toml
[strategy]
strategy_mode = "market_making"
update_interval_ms = 100
order_book_depth = 10
spread_bps = 20
```

## 📈 Monitoring

### Grafana Dashboard
- **URL**: http://localhost:3000
- **Username**: admin
- **Password**: admin

### Prometheus Metrics
- **URL**: http://localhost:9090

### Custom Metrics Endpoint
- **URL**: http://localhost:8080/metrics

## 🔧 Development

### Local Development

```bash
# Install dependencies
cargo build --release

# Run tests
cargo test

# Run with custom config
cargo run -- --config-path ./config
```

### Adding New Strategies

1. Create strategy in `src/strategy/`
2. Implement the `Strategy` trait
3. Register in `src/strategy/mod.rs`

```rust
pub struct MyStrategy {
    config: StrategyConfig,
}

#[async_trait]
impl Strategy for MyStrategy {
    async fn generate_orders(&self, market: &MarketSnapshot) -> Result<Vec<Order>> {
        // Your strategy logic
    }
}
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run benchmarks
cargo bench

# Run integration tests
./scripts/test.sh
```

## 🔒 Security

- **Non-custodial**: Private keys never leave your environment
- **Rate limiting**: Built-in DDoS protection
- **Audit logging**: Comprehensive transaction logging
- **Health checks**: Automated system monitoring

## 📚 Documentation

- [API Documentation](docs/api.md)
- [Strategy Guide](docs/strategies.md)
- [Risk Management](docs/risk.md)
- [Deployment Guide](docs/deployment.md)

## 🤝 Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

**This software is for educational and research purposes only. Use at your own risk. Trading cryptocurrencies involves substantial risk of loss and is not suitable for every investor.**

## 🆘 Support

- **Discord**: [HFT Ninja Community](https://discord.gg/hftninja)
- **Issues**: [GitHub Issues](https://github.com/hftninja/solana-hft-ninja-2025.07/issues)
- **Wiki**: [Project Wiki](https://github.com/hftninja/solana-hft-ninja-2025.07/wiki)

---

**Built with ❤️ by the HFT Ninja Team**