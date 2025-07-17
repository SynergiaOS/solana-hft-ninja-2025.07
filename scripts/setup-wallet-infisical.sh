#!/bin/bash

# 🔐 Solana HFT Ninja - Infisical Wallet Setup
# Configure wallet through Infisical secrets management

set -e

echo "🔐 Solana HFT Ninja - Infisical Wallet Setup"
echo "============================================"

# Check if infisical CLI is installed
if ! command -v infisical &> /dev/null; then
    echo "❌ Infisical CLI not found. Installing..."
    curl -1sLf 'https://dl.cloudsmith.io/public/infisical/infisical-cli/setup.deb.sh' | sudo -E bash
    sudo apt-get update && sudo apt-get install -y infisical
fi

# Check if logged in to Infisical
if ! infisical user &> /dev/null; then
    echo "🔑 Please login to Infisical first:"
    infisical login
fi

PROJECT_ID="73c2f3cb-c922-4a46-a333-7b96fbc6301a"
ENVIRONMENT="production"

echo ""
echo "📋 Current Infisical Configuration:"
echo "Project ID: $PROJECT_ID"
echo "Environment: $ENVIRONMENT"
echo ""

# List current secrets
echo "🔍 Current secrets in Infisical:"
infisical secrets --projectId="$PROJECT_ID" --env="$ENVIRONMENT"

echo ""
echo "Choose wallet setup method:"
echo "1) Upload existing wallet file to Infisical"
echo "2) Set wallet private key as base58 string"
echo "3) Generate new wallet and upload to Infisical"
echo ""
read -p "Enter choice (1-3): " choice

case $choice in
    1)
        read -p "Enter path to your wallet file: " wallet_file
        if [ -f "$wallet_file" ]; then
            WALLET_CONTENT=$(cat "$wallet_file")
            infisical secrets set WALLET_PRIVATE_KEY="$WALLET_CONTENT" --projectId="$PROJECT_ID" --env="$ENVIRONMENT"
            echo "✅ Wallet file uploaded to Infisical!"
        else
            echo "❌ File not found: $wallet_file"
            exit 1
        fi
        ;;
    2)
        read -p "Enter your base58 private key: " base58_key
        infisical secrets set WALLET_PRIVATE_KEY="[$base58_key]" --projectId="$PROJECT_ID" --env="$ENVIRONMENT"
        echo "✅ Private key set in Infisical!"
        ;;
    3)
        echo "🔑 Generating new wallet..."
        mkdir -p temp
        solana-keygen new --outfile temp/new_wallet.json --no-bip39-passphrase --force
        WALLET_CONTENT=$(cat temp/new_wallet.json)
        PUBKEY=$(solana-keygen pubkey temp/new_wallet.json)
        
        infisical secrets set WALLET_PRIVATE_KEY="$WALLET_CONTENT" --projectId="$PROJECT_ID" --env="$ENVIRONMENT"
        
        echo "✅ New wallet generated and uploaded to Infisical!"
        echo "🔑 Public Key: $PUBKEY"
        echo "💰 Fund this wallet before trading!"
        
        # Cleanup
        rm -rf temp/
        ;;
    *)
        echo "❌ Invalid choice"
        exit 1
        ;;
esac

echo ""
echo "✅ Wallet Configuration Complete!"
echo "================================="
echo "🔐 Wallet private key is now stored securely in Infisical"
echo "🚀 Start HFT Ninja with: ./scripts/run-with-infisical.sh"
echo ""
echo "⚠️  Security Notes:"
echo "• Private key is encrypted in Infisical"
echo "• Access controlled by service tokens"
echo "• Audit logs available in Infisical dashboard"
