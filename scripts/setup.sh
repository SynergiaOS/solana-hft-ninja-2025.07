#!/bin/bash

set -e

echo "🚀 Setting up Solana HFT Ninja 2025.07..."

# Create necessary directories
mkdir -p config logs data

# Copy environment file if it doesn't exist
if [ ! -f .env ]; then
    cp .env.example .env
    echo "✅ Created .env file from .env.example"
fi

# Create config directory structure
mkdir -p config/prometheus

# Create Prometheus configuration
cat > config/prometheus.yml << EOF
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'hft-ninja'
    static_configs:
      - targets: ['hft-ninja:8080']
    metrics_path: /metrics
    scrape_interval: 5s

  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']
EOF

# Create wallet placeholder
if [ ! -f config/wallet.json ]; then
    echo "⚠️  Please add your wallet private key to config/wallet.json"
    echo "[]" > config/wallet.json
fi

# Create keypair placeholder
if [ ! -f config/keypair.json ]; then
    echo "⚠️  Please add your keypair to config/keypair.json"
    echo "[]" > config/keypair.json
fi

# Create config file
cat > config/config.toml << EOF
[solana]
rpc_url = "https://api.mainnet-beta.solana.com"
ws_url = "wss://api.mainnet-beta.solana.com"
rpc_timeout_ms = 30000

[wallet]
private_key_path = "./config/wallet.json"
keypair_path = "./config/keypair.json"

[trading]
initial_balance_sol = 1.0
max_position_size_sol = 0.1
max_slippage_bps = 50
min_profit_threshold_bps = 10
risk_limit_bps = 100

[strategy]
strategy_mode = "market_making"
update_interval_ms = 100
order_book_depth = 10
spread_bps = 20

[risk]
stop_loss_bps = 200
take_profit_bps = 300
max_drawdown_bps = 500

[logging]
rust_log = "info,solana_hft_ninja=debug"
log_level = "debug"
log_file_path = "./logs/hft.log"

[monitoring]
metrics_port = 8080
health_check_interval_ms = 5000
enable_ddos_protection = true
rate_limit_rps = 100
EOF

echo "✅ Setup complete!"
echo ""
echo "Next steps:"
echo "1. Add your wallet private key to config/wallet.json"
echo "2. Add your keypair to config/keypair.json"
echo "3. Run: ./scripts/start.sh"