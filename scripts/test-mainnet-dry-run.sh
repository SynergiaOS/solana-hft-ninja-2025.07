#!/bin/bash

# 🥷 Test Mainnet Trading (Dry Run)
# Safe testing of mainnet systems without real money

set -e

echo "🥷 Solana HFT Ninja - Mainnet Dry Run Test"
echo "=========================================="
echo ""

# Configuration
WALLET_ADDRESS="uqAmyJmTS34GRbj5JLA4MTofCCcRCcrT99uhkCvyiM6"
CONFIG_FILE="config/mainnet-ultra-safe.toml"

echo "📋 Test Configuration:"
echo "Wallet: $WALLET_ADDRESS"
echo "Config: $CONFIG_FILE"
echo "Mode: DRY RUN (safe)"
echo ""

# Check if binary exists
if [ ! -f "target/release/devnet_trader" ]; then
    echo "🔨 Building mainnet trader..."
    cargo build --release --bin devnet_trader
fi

echo "✅ Binary ready"

# Check wallet balance
echo "💰 Checking wallet balance..."
BALANCE=$(solana balance $WALLET_ADDRESS --url mainnet-beta | cut -d' ' -f1)
echo "💰 Current balance: $BALANCE SOL"

echo ""
echo "🧪 STARTING DRY RUN TESTS"
echo "========================="
echo ""

# Test 1: Arbitrage (5 minutes, dry run)
echo "🎯 Test 1: Arbitrage Strategy (Dry Run)"
echo "Duration: 5 minutes"
echo "Position: 0.005 SOL"
echo ""

./target/release/devnet_trader \
    --config "$CONFIG_FILE" \
    --strategy arbitrage \
    --duration 300 \
    --max-position 0.005 \
    --min-profit 0.001 \
    --dry-run \
    --verbose

echo ""
echo "✅ Test 1 completed!"
echo ""

# Test 2: Jupiter Arbitrage (3 minutes, dry run)
echo "🎯 Test 2: Jupiter Arbitrage Strategy (Dry Run)"
echo "Duration: 3 minutes"
echo "Position: 0.003 SOL"
echo ""

./target/release/devnet_trader \
    --config "$CONFIG_FILE" \
    --strategy jupiter-arb \
    --duration 180 \
    --max-position 0.003 \
    --min-profit 0.001 \
    --dry-run \
    --verbose

echo ""
echo "✅ Test 2 completed!"
echo ""

# Final balance check
FINAL_BALANCE=$(solana balance $WALLET_ADDRESS --url mainnet-beta | cut -d' ' -f1)
echo "💰 Final balance: $FINAL_BALANCE SOL"

if [ "$BALANCE" = "$FINAL_BALANCE" ]; then
    echo "✅ Balance unchanged (expected for dry run)"
else
    echo "⚠️  Balance changed: $BALANCE → $FINAL_BALANCE SOL"
fi

echo ""
echo "🎉 MAINNET DRY RUN TESTS COMPLETED!"
echo "=================================="
echo ""
echo "✅ All systems tested successfully"
echo "✅ Ready for real trading (when funded)"
echo ""
echo "Next steps:"
echo "1. Fund wallet: $WALLET_ADDRESS"
echo "2. Run: ./scripts/mainnet-trading.sh"
echo "3. Start with small amounts!"
echo ""
