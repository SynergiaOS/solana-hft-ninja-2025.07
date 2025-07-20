#!/bin/bash

# Run Solana HFT Ninja with Infisical secrets
# This script loads secrets from Infisical and runs the application

set -e

PROJECT_ID="1232ea01-7ff9-4eac-be5a-c66a6cb34c88"
ENVIRONMENT="production"

echo "🔐 Starting Solana HFT Ninja with Infisical secrets..."
echo "📋 Project ID: $PROJECT_ID"
echo "🌍 Environment: $ENVIRONMENT"
echo ""

# Load local environment if available
if [ -f .env.local ]; then
    echo "📁 Loading local environment..."
    source .env.local
fi

# Check if token is available
if [ -z "$INFISICAL_TOKEN" ]; then
    echo "❌ INFISICAL_TOKEN not found"
    echo "   Please set it in .env.local or environment"
    exit 1
fi

echo "✅ Infisical token found"

# Check if application binary exists
if [ ! -f "./target/release/hft_main" ]; then
    echo "🔨 Building application..."
    cargo build --release --bin hft_main
fi

echo "🚀 Starting HFT Ninja with Infisical secrets injection..."
echo ""

# Run with Infisical (DEVNET Testing)
infisical run \
    --projectId="$PROJECT_ID" \
    --env="$ENVIRONMENT" \
    --token="$INFISICAL_TOKEN" \
    -- ./target/release/hft_main \
    --config-path config/config.toml \
    --enable-helius \
    --enable-mev \
    --enable-jito \
    --log-level info

echo ""
echo "✅ HFT Ninja session completed"
