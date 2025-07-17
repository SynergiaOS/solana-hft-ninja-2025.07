#!/bin/bash

# 🚀 MAINNET WALLET SETUP SCRIPT
# Production-grade wallet creation and security setup

set -euo pipefail

echo "🔐 SOLANA MAINNET WALLET SETUP"
echo "================================"

# Check if solana CLI is installed
if ! command -v solana &> /dev/null; then
    echo "❌ Solana CLI not found. Installing..."
    sh -c "$(curl -sSfL https://release.solana.com/v1.18.4/install)"
    export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
fi

# Set to mainnet
echo "🌐 Setting Solana CLI to mainnet-beta..."
solana config set --url https://api.mainnet-beta.solana.com

# Create production wallet directory
WALLET_DIR="$HOME/.solana-mainnet"
mkdir -p "$WALLET_DIR"
chmod 700 "$WALLET_DIR"

# Generate new keypair for production
WALLET_PATH="$WALLET_DIR/trading-wallet.json"

if [ ! -f "$WALLET_PATH" ]; then
    echo "🔑 Generating new production wallet..."
    solana-keygen new --outfile "$WALLET_PATH" --no-bip39-passphrase
    chmod 600 "$WALLET_PATH"
else
    echo "✅ Production wallet already exists at $WALLET_PATH"
fi

# Get wallet address
WALLET_ADDRESS=$(solana-keygen pubkey "$WALLET_PATH")
echo "📍 Wallet Address: $WALLET_ADDRESS"

# Check balance
echo "💰 Checking wallet balance..."
BALANCE=$(solana balance "$WALLET_ADDRESS" --url https://api.mainnet-beta.solana.com)
echo "Current Balance: $BALANCE"

# Create backup
BACKUP_DIR="$HOME/.solana-backups"
mkdir -p "$BACKUP_DIR"
chmod 700 "$BACKUP_DIR"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_PATH="$BACKUP_DIR/trading-wallet-backup-$TIMESTAMP.json"
cp "$WALLET_PATH" "$BACKUP_PATH"
chmod 600 "$BACKUP_PATH"

echo "💾 Backup created: $BACKUP_PATH"

# Create environment file
ENV_FILE=".env.mainnet"
cat > "$ENV_FILE" << EOF
# 🚀 MAINNET PRODUCTION CONFIGURATION
# Generated: $(date)

# Wallet Configuration
SOLANA_NETWORK=mainnet-beta
WALLET_PRIVATE_KEY_PATH=$WALLET_PATH
WALLET_ADDRESS=$WALLET_ADDRESS

# RPC Configuration (UPDATE WITH YOUR KEYS)
HELIUS_API_KEY=your_helius_key_here
QUICKNODE_ENDPOINT=your_quicknode_endpoint_here
SOLANA_RPC_URL=https://mainnet.helius-rpc.com/?api-key=\${HELIUS_API_KEY}
SOLANA_RPC_FALLBACK=https://your-endpoint.solana-mainnet.quiknode.pro/your_key/

# Trading Configuration
TRADING_ENABLED=false
MAX_POSITION_SIZE_SOL=1.0
MAX_DAILY_LOSS_SOL=0.5
STOP_LOSS_PERCENTAGE=5.0

# Risk Management
CIRCUIT_BREAKER_ENABLED=true
MAX_SLIPPAGE_PERCENTAGE=2.0
MIN_LIQUIDITY_USD=10000

# Monitoring
PROMETHEUS_ENABLED=true
GRAFANA_ENABLED=true
ALERT_WEBHOOK_URL=your_webhook_url_here

# Security
RATE_LIMIT_ENABLED=true
IP_WHITELIST_ENABLED=false
API_KEY_REQUIRED=true
EOF

chmod 600 "$ENV_FILE"

echo "⚙️  Environment file created: $ENV_FILE"

# Security recommendations
echo ""
echo "🛡️  SECURITY RECOMMENDATIONS:"
echo "================================"
echo "1. ✅ Wallet private key secured with 600 permissions"
echo "2. ✅ Backup created in $BACKUP_DIR"
echo "3. ⚠️  UPDATE .env.mainnet with your RPC API keys"
echo "4. ⚠️  Store backup in secure, offline location"
echo "5. ⚠️  Never commit private keys to git"
echo "6. ⚠️  Use hardware wallet for large amounts"
echo "7. ⚠️  Enable 2FA on all accounts"
echo ""

# Add to .gitignore
if [ -f ".gitignore" ]; then
    echo "# Mainnet secrets" >> .gitignore
    echo ".env.mainnet" >> .gitignore
    echo "*.json" >> .gitignore
    echo ".solana-*/" >> .gitignore
fi

echo "🎯 NEXT STEPS:"
echo "1. Fund wallet: solana transfer <amount> $WALLET_ADDRESS"
echo "2. Update RPC API keys in .env.mainnet"
echo "3. Run security audit: ./scripts/security-audit.sh"
echo "4. Test with small amounts first"
echo ""
echo "🚀 Wallet setup complete!"
