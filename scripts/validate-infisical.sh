#!/bin/bash

# Infisical Validation Script for Solana HFT Ninja
# This script validates Infisical setup and secret access

set -e

PROJECT_ID="73c2f3cb-c922-4a46-a333-7b96fbc6301a"
ENVIRONMENT="production"

echo "🔍 Validating Infisical setup for Solana HFT Ninja..."
echo "📋 Project ID: $PROJECT_ID"
echo "🌍 Environment: $ENVIRONMENT"
echo ""

# Check if Infisical CLI is installed
if ! command -v infisical &> /dev/null; then
    echo "❌ Infisical CLI not found. Please install it first:"
    echo "   curl -1sLf 'https://dl.cloudsmith.io/public/infisical/infisical-cli/setup.deb.sh' | sudo -E bash"
    echo "   sudo apt-get update && sudo apt-get install -y infisical"
    exit 1
fi

echo "✅ Infisical CLI found: $(infisical --version)"

# Check authentication
echo "🔐 Checking authentication..."
if [ -n "$INFISICAL_TOKEN" ]; then
    echo "✅ Service token found in environment"
    # Test token by trying to list secrets
    if infisical secrets --projectId="$PROJECT_ID" --env="$ENVIRONMENT" --token="$INFISICAL_TOKEN" &> /dev/null; then
        echo "✅ Service token is valid and working"
    else
        echo "❌ Service token is invalid or expired"
        exit 1
    fi
elif infisical whoami &> /dev/null; then
    echo "✅ Authenticated as: $(infisical whoami)"
else
    echo "❌ Not authenticated with Infisical"
    echo "   Please run: infisical login"
    echo "   Or set INFISICAL_TOKEN environment variable"
    exit 1
fi

# Test project access
echo "📂 Testing project access..."
# For service tokens, we test access by trying to list secrets
if [ -n "$INFISICAL_TOKEN" ]; then
    if infisical secrets --projectId="$PROJECT_ID" --env="$ENVIRONMENT" --token="$INFISICAL_TOKEN" &> /dev/null; then
        echo "✅ Project access confirmed via service token"
    else
        echo "❌ Cannot access project $PROJECT_ID with service token"
        echo "   Please check your token permissions"
        exit 1
    fi
else
    if infisical projects list | grep -q "$PROJECT_ID"; then
        echo "✅ Project access confirmed"
    else
        echo "❌ Cannot access project $PROJECT_ID"
        echo "   Please check your permissions"
        exit 1
    fi
fi

# Test secret listing
echo "🔑 Testing secret access..."
if [ -n "$INFISICAL_TOKEN" ]; then
    SECRET_COUNT=$(infisical secrets --projectId="$PROJECT_ID" --env="$ENVIRONMENT" --token="$INFISICAL_TOKEN" --plain | wc -l 2>/dev/null || echo "0")
else
    SECRET_COUNT=$(infisical secrets list --projectId="$PROJECT_ID" --env="$ENVIRONMENT" --format=json | jq length 2>/dev/null || echo "0")
fi

if [ "$SECRET_COUNT" -gt 0 ]; then
    echo "✅ Found $SECRET_COUNT secrets in $ENVIRONMENT environment"
else
    echo "⚠️  No secrets found in $ENVIRONMENT environment"
    echo "   Please add secrets to your Infisical project"
fi

# Test critical secrets
echo "🎯 Checking critical secrets..."
CRITICAL_SECRETS=("HELIUS_API_KEY" "QUICKNODE_API_KEY" "GRAFANA_PASSWORD")

for secret in "${CRITICAL_SECRETS[@]}"; do
    if [ -n "$INFISICAL_TOKEN" ]; then
        if infisical secrets get "$secret" --projectId="$PROJECT_ID" --env="$ENVIRONMENT" --token="$INFISICAL_TOKEN" &> /dev/null; then
            echo "✅ $secret: Found"
        else
            echo "⚠️  $secret: Missing"
        fi
    else
        if infisical secrets get "$secret" --projectId="$PROJECT_ID" --env="$ENVIRONMENT" &> /dev/null; then
            echo "✅ $secret: Found"
        else
            echo "⚠️  $secret: Missing"
        fi
    fi
done

# Test export functionality
echo "📤 Testing secret export..."
TEMP_FILE=$(mktemp)
if [ -n "$INFISICAL_TOKEN" ]; then
    if infisical export --projectId="$PROJECT_ID" --env="$ENVIRONMENT" --format=dotenv --token="$INFISICAL_TOKEN" > "$TEMP_FILE" 2>/dev/null; then
        EXPORTED_COUNT=$(grep -c "=" "$TEMP_FILE" || echo "0")
        echo "✅ Successfully exported $EXPORTED_COUNT secrets"
        rm "$TEMP_FILE"
    else
        echo "❌ Failed to export secrets"
        rm "$TEMP_FILE"
        exit 1
    fi
else
    if infisical export --projectId="$PROJECT_ID" --env="$ENVIRONMENT" --format=dotenv > "$TEMP_FILE" 2>/dev/null; then
        EXPORTED_COUNT=$(grep -c "=" "$TEMP_FILE" || echo "0")
        echo "✅ Successfully exported $EXPORTED_COUNT secrets"
        rm "$TEMP_FILE"
    else
        echo "❌ Failed to export secrets"
        rm "$TEMP_FILE"
        exit 1
    fi
fi

# Test Docker integration
echo "🐳 Testing Docker integration..."
if [ -f "docker-compose.yml" ]; then
    if grep -q "INFISICAL_PROJECT_ID" docker-compose.yml; then
        echo "✅ Docker Compose configured for Infisical"
    else
        echo "⚠️  Docker Compose not configured for Infisical"
    fi
else
    echo "⚠️  docker-compose.yml not found"
fi

# Summary
echo ""
echo "📊 Validation Summary:"
echo "   ✅ Infisical CLI: Installed"
echo "   ✅ Authentication: Valid"
echo "   ✅ Project Access: Confirmed"
echo "   ✅ Secret Export: Working"
echo "   📊 Total Secrets: $SECRET_COUNT"
echo ""
echo "🚀 Infisical is ready for Solana HFT Ninja!"
echo ""
echo "Next steps:"
echo "1. Add missing critical secrets to Infisical"
echo "2. Run: ./scripts/infisical-setup.sh"
echo "3. Start with: docker-compose up -d"
