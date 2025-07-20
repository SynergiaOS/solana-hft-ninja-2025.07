#!/bin/bash

# 🥷 Solana HFT Ninja - Mainnet Trading Script
# REAL MONEY TRADING - USE WITH EXTREME CAUTION

set -e

echo "🥷 Solana HFT Ninja - Mainnet Trading"
echo "====================================="
echo ""

# Configuration
PROJECT_ID="1232ea01-7ff9-4eac-be5a-c66a6cb34c88"
ENVIRONMENT="production"
WALLET_ADDRESS=$(solana-keygen pubkey config/mainnet-wallet-infisical.json)
CONFIG_FILE="config/mainnet-ultra-safe.toml"

# Check if Infisical token is set
if [ -z "$INFISICAL_TOKEN" ]; then
    echo "❌ INFISICAL_TOKEN not set"
    echo "   Please set: export INFISICAL_TOKEN=\"st.b17d925d-9dc9-4f5c-90e2-c350dfe0a7fe.6e50b9f94a8329c537739814c8846f5b.07d6f15e828b3adba654f847687a7c97\""
    exit 1
fi

echo "✅ Infisical token found"

# Check wallet balance
echo "💰 Checking wallet balance..."
BALANCE=$(solana balance $WALLET_ADDRESS --url mainnet-beta | cut -d' ' -f1)
echo "💰 Current balance: $BALANCE SOL"

# Safety check
if (( $(echo "$BALANCE < 0.01" | bc -l) )); then
    echo "❌ Insufficient balance: $BALANCE SOL (minimum: 0.01 SOL)"
    echo "   Please fund wallet: $WALLET_ADDRESS"
    exit 1
fi

echo "✅ Sufficient balance for trading"

# Build mainnet trader if needed
if [ ! -f "target/release/devnet_trader" ]; then
    echo "🔨 Building mainnet trader..."
    cargo build --release --bin devnet_trader
fi

echo ""
echo "🚨 MAINNET TRADING WARNING 🚨"
echo "=============================="
echo "• This will use REAL SOL from wallet: $WALLET_ADDRESS"
echo "• Current balance: $BALANCE SOL"
echo "• Trading with real money involves significant risk"
echo "• You may lose all your funds"
echo ""

# Strategy selection
echo "Select trading strategy:"
echo "1) Arbitrage (Recommended for beginners)"
echo "2) Jupiter Arbitrage (Medium risk)"
echo "3) Sandwich (High risk)"
echo "4) Sniping (Very high risk)"
echo "5) Liquidation (High risk)"
echo "6) All strategies (Expert only)"
echo ""
read -p "Enter choice (1-6): " strategy_choice

case $strategy_choice in
    1) STRATEGY="arbitrage" ;;
    2) STRATEGY="jupiter-arb" ;;
    3) STRATEGY="sandwich" ;;
    4) STRATEGY="sniping" ;;
    5) STRATEGY="liquidation" ;;
    6) STRATEGY="all" ;;
    *) echo "❌ Invalid choice"; exit 1 ;;
esac

# Duration selection
echo ""
echo "Select trading duration:"
echo "1) 5 minutes (Quick test)"
echo "2) 15 minutes (Short session)"
echo "3) 30 minutes (Medium session)"
echo "4) 60 minutes (Long session)"
echo "5) Custom duration"
echo ""
read -p "Enter choice (1-5): " duration_choice

case $duration_choice in
    1) DURATION=300 ;;
    2) DURATION=900 ;;
    3) DURATION=1800 ;;
    4) DURATION=3600 ;;
    5) 
        read -p "Enter duration in seconds: " DURATION
        if ! [[ "$DURATION" =~ ^[0-9]+$ ]]; then
            echo "❌ Invalid duration"
            exit 1
        fi
        ;;
    *) echo "❌ Invalid choice"; exit 1 ;;
esac

# Position size selection
echo ""
echo "Select maximum position size:"
echo "1) 0.005 SOL (Ultra conservative)"
echo "2) 0.01 SOL (Conservative)"
echo "3) 0.02 SOL (Moderate)"
echo "4) 0.03 SOL (Aggressive)"
echo "5) Custom amount"
echo ""
read -p "Enter choice (1-5): " position_choice

case $position_choice in
    1) MAX_POSITION=0.005 ;;
    2) MAX_POSITION=0.01 ;;
    3) MAX_POSITION=0.02 ;;
    4) MAX_POSITION=0.03 ;;
    5) 
        read -p "Enter max position in SOL: " MAX_POSITION
        if ! [[ "$MAX_POSITION" =~ ^[0-9]+\.?[0-9]*$ ]]; then
            echo "❌ Invalid position size"
            exit 1
        fi
        ;;
    *) echo "❌ Invalid choice"; exit 1 ;;
esac

# Dry run option
echo ""
read -p "Start with dry run? (y/N): " dry_run_choice
if [[ $dry_run_choice =~ ^[Yy]$ ]]; then
    DRY_RUN="--dry-run"
    echo "✅ Dry run mode enabled (safe)"
else
    DRY_RUN=""
    echo "🚨 REAL TRADING MODE (dangerous)"
fi

# Final confirmation
echo ""
echo "🎯 TRADING CONFIGURATION"
echo "========================"
echo "Strategy: $STRATEGY"
echo "Duration: $DURATION seconds"
echo "Max Position: $MAX_POSITION SOL"
echo "Wallet: $WALLET_ADDRESS"
echo "Balance: $BALANCE SOL"
echo "Mode: $([ -n "$DRY_RUN" ] && echo "DRY RUN" || echo "REAL TRADING")"
echo ""

if [ -z "$DRY_RUN" ]; then
    echo "🚨 FINAL WARNING: This will trade with REAL MONEY!"
    echo "🚨 You may lose all your funds!"
    echo ""
    read -p "Type 'I UNDERSTAND THE RISKS' to continue: " confirmation
    if [ "$confirmation" != "I UNDERSTAND THE RISKS" ]; then
        echo "❌ Trading cancelled"
        exit 1
    fi
fi

echo ""
echo "🚀 Starting mainnet trading..."
echo "Press Ctrl+C to stop at any time"
echo ""

# Start trading
./target/release/devnet_trader \
    --config "$CONFIG_FILE" \
    --strategy "$STRATEGY" \
    --duration "$DURATION" \
    --max-position "$MAX_POSITION" \
    --min-profit 0.001 \
    --verbose \
    $DRY_RUN

echo ""
echo "🎉 Trading session completed!"
echo ""

# Final balance check
FINAL_BALANCE=$(solana balance $WALLET_ADDRESS --url mainnet-beta | cut -d' ' -f1)
echo "💰 Final balance: $FINAL_BALANCE SOL"

# Calculate P&L
PROFIT_LOSS=$(echo "$FINAL_BALANCE - $BALANCE" | bc -l)
if (( $(echo "$PROFIT_LOSS > 0" | bc -l) )); then
    echo "📈 Profit: +$PROFIT_LOSS SOL"
elif (( $(echo "$PROFIT_LOSS < 0" | bc -l) )); then
    echo "📉 Loss: $PROFIT_LOSS SOL"
else
    echo "➡️ No change: $PROFIT_LOSS SOL"
fi

echo ""
echo "🥷 Mainnet trading session completed!"
