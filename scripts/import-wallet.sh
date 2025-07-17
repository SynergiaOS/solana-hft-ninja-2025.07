#!/bin/bash

# 🔐 Solana HFT Ninja - Wallet Import
# Import existing wallet from seed phrase or private key

set -e

echo "🔐 Solana HFT Ninja - Wallet Import"
echo "==================================="

# Check if solana CLI is installed
if ! command -v solana &> /dev/null; then
    echo "❌ Solana CLI not found. Installing..."
    sh -c "$(curl -sSfL https://release.solana.com/v1.18.4/install)"
    export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
fi

# Create config directory if it doesn't exist
mkdir -p config

echo ""
echo "Choose import method:"
echo "1) Import from seed phrase (12/24 words)"
echo "2) Import from private key file"
echo "3) Import from base58 private key"
echo ""
read -p "Enter choice (1-3): " choice

case $choice in
    1)
        echo "🌱 Importing from seed phrase..."
        echo "⚠️  Enter your 12 or 24 word seed phrase:"
        solana-keygen recover --outfile config/wallet.json
        ;;
    2)
        echo "📁 Importing from private key file..."
        read -p "Enter path to your private key file: " keyfile
        if [ -f "$keyfile" ]; then
            cp "$keyfile" config/wallet.json
            echo "✅ Private key imported successfully!"
        else
            echo "❌ File not found: $keyfile"
            exit 1
        fi
        ;;
    3)
        echo "🔑 Importing from base58 private key..."
        read -p "Enter your base58 private key: " base58_key
        echo "[$base58_key]" > config/wallet.json
        echo "✅ Base58 private key imported successfully!"
        ;;
    *)
        echo "❌ Invalid choice"
        exit 1
        ;;
esac

# Copy to keypair.json for compatibility
cp config/wallet.json config/keypair.json

# Get public key
PUBKEY=$(solana-keygen pubkey config/wallet.json)

echo ""
echo "✅ Wallet Imported Successfully!"
echo "================================"
echo "📁 Private Key: config/wallet.json"
echo "🔑 Public Key: $PUBKEY"
echo ""
echo "🔍 Check balance:"
echo "solana balance $PUBKEY"
echo ""
echo "🚀 Ready to start HFT Ninja!"
