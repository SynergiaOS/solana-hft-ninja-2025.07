#!/bin/bash

# Infisical Setup Script for Solana HFT Ninja
# This script sets up Infisical CLI and fetches secrets

set -e

echo "🔐 Setting up Infisical for Solana HFT Ninja..."

# Install Infisical CLI if not present
if ! command -v infisical &> /dev/null; then
    echo "📦 Installing Infisical CLI..."
    curl -1sLf 'https://dl.cloudsmith.io/public/infisical/infisical-cli/setup.deb.sh' | sudo -E bash
    sudo apt-get update && sudo apt-get install -y infisical
fi

# Check if Infisical is authenticated
if ! infisical whoami &> /dev/null; then
    echo "🔑 Please authenticate with Infisical:"
    echo "Run: infisical login"
    echo "Or set INFISICAL_TOKEN environment variable"
    exit 1
fi

# Project configuration
PROJECT_ID="73c2f3cb-c922-4a46-a333-7b96fbc6301a"
ENVIRONMENT="production"

echo "📋 Project ID: $PROJECT_ID"
echo "🌍 Environment: $ENVIRONMENT"

# Create infisical directory if it doesn't exist
mkdir -p ./infisical

# Fetch secrets and create .env file
echo "🔄 Fetching secrets from Infisical..."
infisical export --projectId="$PROJECT_ID" --env="$ENVIRONMENT" --format=dotenv > ./infisical/.env.infisical

# Backup existing .env if it exists
if [ -f .env ]; then
    echo "💾 Backing up existing .env to .env.backup"
    cp .env .env.backup
fi

# Merge Infisical secrets with local .env
echo "🔗 Merging Infisical secrets with local configuration..."
cat ./infisical/.env.infisical > .env.merged

# Add local overrides if .env.local exists
if [ -f .env.local ]; then
    echo "📝 Adding local overrides from .env.local"
    cat .env.local >> .env.merged
fi

# Replace .env with merged configuration
mv .env.merged .env

echo "✅ Infisical setup complete!"
echo "🔐 Secrets have been fetched and merged into .env"
echo "🚀 You can now run: docker-compose up -d"
