# 🔐 Infisical Integration for Solana HFT Ninja

This guide explains how to set up Infisical secrets management for secure handling of API keys, private keys, and other sensitive configuration.

## 📋 Prerequisites

1. **Infisical Account**: Sign up at [infisical.com](https://infisical.com)
2. **Project ID**: `73c2f3cb-c922-4a46-a333-7b96fbc6301a`
3. **Docker & Docker Compose** installed

## 🚀 Quick Setup

### 1. Install Infisical CLI

```bash
# Ubuntu/Debian
curl -1sLf 'https://dl.cloudsmith.io/public/infisical/infisical-cli/setup.deb.sh' | sudo -E bash
sudo apt-get update && sudo apt-get install -y infisical

# macOS
brew install infisical/get-cli/infisical

# Or use our setup script
./scripts/infisical-setup.sh
```

### 2. Authenticate with Infisical

```bash
# Option 1: Interactive login
infisical login

# Option 2: Service token (recommended for production)
export INFISICAL_TOKEN="your_service_token_here"
```

### 3. Configure Secrets in Infisical Dashboard

1. Go to [Infisical Dashboard](https://app.infisical.com)
2. Navigate to project: `73c2f3cb-c922-4a46-a333-7b96fbc6301a`
3. Add secrets from `.env.infisical.template`:

```bash
# Critical secrets to add:
HELIUS_API_KEY=your_actual_helius_key
QUICKNODE_API_KEY=your_quicknode_key
GRAFANA_PASSWORD=secure_password
POSTGRES_PASSWORD=secure_password
INFISICAL_TOKEN=your_service_token
```

### 4. Run with Infisical

```bash
# Option 1: Use setup script
./scripts/infisical-setup.sh
docker-compose up -d

# Option 2: Direct Infisical integration
docker-compose -f docker-compose.yml -f docker-compose.infisical.yml up -d

# Option 3: Local development with Infisical
infisical run --projectId=73c2f3cb-c922-4a46-a333-7b96fbc6301a --env=production -- ./target/release/hft_main --dry-run --enable-helius
```

## 🔧 Configuration

### Environment Structure

```
production/     # Main environment
├── HELIUS_API_KEY
├── QUICKNODE_API_KEY
├── GRAFANA_PASSWORD
└── ...

development/    # Dev environment
├── HELIUS_API_KEY (dev key)
└── ...
```

### Service Token Setup

1. In Infisical Dashboard → Settings → Service Tokens
2. Create new token with:
   - **Environment**: `production`
   - **Path**: `/`
   - **Permissions**: Read
3. Copy token and set as `INFISICAL_TOKEN`

## 🐳 Docker Integration

### Standard Docker Compose

```yaml
services:
  hft-ninja:
    environment:
      - INFISICAL_PROJECT_ID=73c2f3cb-c922-4a46-a333-7b96fbc6301a
      - INFISICAL_TOKEN=${INFISICAL_TOKEN}
    command: |
      sh -c "
        infisical export --projectId=73c2f3cb-c922-4a46-a333-7b96fbc6301a --env=production --format=dotenv > /tmp/.env.infisical
        export $$(cat /tmp/.env.infisical | xargs)
        exec /app/hft_main --dry-run --enable-helius
      "
```

### With Infisical Web UI

```bash
# Run full Infisical stack
docker-compose -f docker-compose.yml -f docker-compose.infisical.yml up -d

# Access Infisical Web UI
open http://localhost:8081
```

## 🔒 Security Best Practices

### 1. Secret Rotation
```bash
# Rotate Helius API key
infisical secrets set HELIUS_API_KEY new_key_value --projectId=73c2f3cb-c922-4a46-a333-7b96fbc6301a --env=production

# Restart services to pick up new secrets
docker-compose restart hft-ninja
```

### 2. Environment Separation
- **Production**: Real API keys, mainnet
- **Development**: Test keys, devnet
- **Staging**: Limited keys, testnet

### 3. Access Control
- Use service tokens for automated deployments
- Limit token permissions to specific paths
- Rotate tokens regularly

## 🛠️ Troubleshooting

### Common Issues

1. **Authentication Failed**
```bash
# Check authentication
infisical whoami

# Re-authenticate
infisical login
```

2. **Project Not Found**
```bash
# Verify project ID
infisical projects list

# Check permissions
infisical secrets list --projectId=73c2f3cb-c922-4a46-a333-7b96fbc6301a
```

3. **Secrets Not Loading**
```bash
# Test secret export
infisical export --projectId=73c2f3cb-c922-4a46-a333-7b96fbc6301a --env=production --format=dotenv

# Check Docker logs
docker-compose logs hft-ninja
```

### Debug Commands

```bash
# List all secrets
infisical secrets list --projectId=73c2f3cb-c922-4a46-a333-7b96fbc6301a --env=production

# Export secrets to file
infisical export --projectId=73c2f3cb-c922-4a46-a333-7b96fbc6301a --env=production --format=dotenv > debug.env

# Run with debug logging
INFISICAL_LOG_LEVEL=debug infisical run --projectId=73c2f3cb-c922-4a46-a333-7b96fbc6301a --env=production -- env
```

## 📚 Additional Resources

- [Infisical Documentation](https://infisical.com/docs)
- [Docker Integration Guide](https://infisical.com/docs/integrations/docker)
- [Service Tokens](https://infisical.com/docs/documentation/platform/service-tokens)
- [CLI Reference](https://infisical.com/docs/cli/overview)

## 🎯 Next Steps

1. Set up secrets in Infisical Dashboard
2. Configure service token for production
3. Test with development environment
4. Deploy with Docker Compose
5. Monitor and rotate secrets regularly
