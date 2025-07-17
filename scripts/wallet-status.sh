#!/bin/bash

# 🔐 Solana HFT Ninja - Wallet Status Checker
# Check wallet configuration and balance

set -e

echo "🔐 Solana HFT Ninja - Wallet Status"
echo "==================================="

# Check if solana CLI is installed
if ! command -v solana &> /dev/null; then
    echo "❌ Solana CLI not found. Please install first."
    exit 1
fi

# Check wallet files
echo "📁 Checking wallet files..."
if [ -f "config/wallet.json" ]; then
    echo "✅ config/wallet.json exists"
    
    # Get public key
    PUBKEY=$(solana-keygen pubkey config/wallet.json 2>/dev/null || echo "❌ Invalid wallet file")
    
    if [[ $PUBKEY == "❌"* ]]; then
        echo "❌ config/wallet.json is invalid or corrupted"
    else
        echo "🔑 Public Key: $PUBKEY"
        
        # Check balance on different networks
        echo ""
        echo "💰 Balance Check:"
        echo "=================="
        
        echo "📊 Mainnet Balance:"
        MAINNET_BALANCE=$(solana balance $PUBKEY --url mainnet-beta 2>/dev/null || echo "Error")
        echo "   $MAINNET_BALANCE"
        
        echo "📊 Devnet Balance:"
        DEVNET_BALANCE=$(solana balance $PUBKEY --url devnet 2>/dev/null || echo "Error")
        echo "   $DEVNET_BALANCE"
        
        echo "📊 Testnet Balance:"
        TESTNET_BALANCE=$(solana balance $PUBKEY --url testnet 2>/dev/null || echo "Error")
        echo "   $TESTNET_BALANCE"
        
        # Check if wallet is funded
        if [[ $MAINNET_BALANCE == *"0 SOL"* ]]; then
            echo ""
            echo "⚠️  WARNING: Wallet has 0 SOL on mainnet!"
            echo "💰 Fund your wallet before trading:"
            echo "   • Send SOL from another wallet"
            echo "   • Use an exchange to deposit"
            echo "   • For testing, use devnet: solana airdrop 1 $PUBKEY --url devnet"
        fi
    fi
else
    echo "❌ config/wallet.json not found"
    echo ""
    echo "🔧 To create a wallet, run:"
    echo "   ./scripts/generate-wallet.sh    # Generate new wallet"
    echo "   ./scripts/import-wallet.sh      # Import existing wallet"
fi

if [ -f "config/keypair.json" ]; then
    echo "✅ config/keypair.json exists"
else
    echo "❌ config/keypair.json not found"
fi

# Check environment configuration
echo ""
echo "⚙️  Environment Configuration:"
echo "=============================="

if [ -f ".env" ]; then
    echo "✅ .env file exists"
    
    # Check wallet paths in .env
    PRIVATE_KEY_PATH=$(grep "PRIVATE_KEY_PATH" .env | cut -d'=' -f2 || echo "")
    KEYPAIR_PATH=$(grep "KEYPAIR_PATH" .env | cut -d'=' -f2 || echo "")
    
    echo "📁 PRIVATE_KEY_PATH: $PRIVATE_KEY_PATH"
    echo "📁 KEYPAIR_PATH: $KEYPAIR_PATH"
    
    # Check if paths match actual files
    if [[ "$PRIVATE_KEY_PATH" == *"wallet.json"* ]] && [ -f "config/wallet.json" ]; then
        echo "✅ Wallet path configuration matches"
    else
        echo "⚠️  Wallet path configuration mismatch"
    fi
else
    echo "❌ .env file not found"
fi

# Check Infisical configuration
echo ""
echo "🔐 Infisical Configuration:"
echo "=========================="

if [ -f "infisical.json" ]; then
    echo "✅ infisical.json exists"
    PROJECT_ID=$(grep "projectId" infisical.json | cut -d'"' -f4)
    echo "📋 Project ID: $PROJECT_ID"
    
    if command -v infisical &> /dev/null; then
        echo "✅ Infisical CLI installed"
        
        # Check if logged in and can access secrets
        if infisical secrets --projectId="$PROJECT_ID" --env="production" &> /dev/null; then
            echo "✅ Infisical access working"
            echo "🔍 Available secrets:"
            infisical secrets --projectId="$PROJECT_ID" --env="production" | grep -E "(WALLET|PRIVATE)" || echo "   No wallet secrets found"
        else
            echo "❌ Cannot access Infisical secrets (login required?)"
        fi
    else
        echo "❌ Infisical CLI not installed"
    fi
else
    echo "❌ infisical.json not found"
fi

echo ""
echo "🚀 Ready to Trade?"
echo "=================="
if [ -f "config/wallet.json" ] && [[ $PUBKEY != "❌"* ]] && [[ $MAINNET_BALANCE != *"0 SOL"* ]]; then
    echo "✅ Wallet is configured and funded!"
    echo "🚀 Start trading with: ./scripts/start.sh"
else
    echo "⚠️  Wallet setup incomplete:"
    [ ! -f "config/wallet.json" ] && echo "   • Create wallet: ./scripts/generate-wallet.sh"
    [[ $PUBKEY == "❌"* ]] && echo "   • Fix wallet file: ./scripts/import-wallet.sh"
    [[ $MAINNET_BALANCE == *"0 SOL"* ]] && echo "   • Fund wallet with SOL"
fi
