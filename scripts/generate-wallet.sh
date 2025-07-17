#!/bin/bash

# 🔐 Solana HFT Ninja - Wallet Generator
# Generates new wallet for testing purposes

set -e

echo "🔐 Solana HFT Ninja - Wallet Generator"
echo "====================================="

# Check if solana CLI is installed
if ! command -v solana &> /dev/null; then
    echo "❌ Solana CLI not found. Installing..."
    sh -c "$(curl -sSfL https://release.solana.com/v1.18.4/install)"
    export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
fi

# Create config directory if it doesn't exist
mkdir -p config

# Generate new keypair
echo "🔑 Generating new keypair..."
solana-keygen new --outfile config/wallet.json --no-bip39-passphrase --force

# Copy to keypair.json for compatibility
cp config/wallet.json config/keypair.json

# Get public key
PUBKEY=$(solana-keygen pubkey config/wallet.json)

echo ""
echo "✅ Wallet Generated Successfully!"
echo "================================="
echo "📁 Private Key: config/wallet.json"
echo "🔑 Public Key: $PUBKEY"
echo ""
echo "⚠️  IMPORTANT SECURITY NOTES:"
echo "• Keep your private key file secure"
echo "• Never share your private key"
echo "• This wallet has 0 SOL - fund it before trading"
echo ""
echo "💰 To fund your wallet:"
echo "solana airdrop 1 $PUBKEY --url devnet"
echo "# Or send SOL from another wallet"
echo ""
echo "🔍 Check balance:"
echo "solana balance $PUBKEY"
echo ""
echo "🚀 Ready to start HFT Ninja!"
