# 🔗 MCP (Machine-readable Cooperative Protocol) Integration

## Overview

Cerebro HFT Ninja integrates with MCP to provide seamless communication between AI assistants (like Claude, Cursor) and our trading ecosystem. This enables natural language interaction with complex trading operations.

## Architecture

```
┌─────────────────┐    ┌──────────────┐    ┌─────────────────┐
│   Claude/Cursor │◄──►│ MCP Protocol │◄──►│ Cerebro BFF API │
└─────────────────┘    └──────────────┘    └─────────────────┘
                                                     │
                                                     ▼
┌─────────────────┐    ┌──────────────┐    ┌─────────────────┐
│   n8n Workflows │◄──►│ HFT Ninja    │◄──►│ External APIs   │
└─────────────────┘    └──────────────┘    └─────────────────┘
```

## Available MCP Servers

### 1. Cerebro Internal Server
**Base URL**: `http://localhost:8000/api/mcp`

**Available Tools**:
- `get_hft_status` - Get current HFT Ninja status
- `get_trading_metrics` - Get real-time trading metrics
- `get_strategy_performance` - Get MEV strategy performance
- `analyze_market_opportunity` - AI-powered market analysis
- `get_portfolio_summary` - Portfolio balance and P&L
- `trigger_emergency_stop` - Emergency trading halt

### 2. n8n Workflows Server
**Base URL**: `http://localhost:5678/api/mcp`

**Available Tools**:
- `trigger_workflow` - Execute n8n workflow
- `get_workflow_status` - Check workflow status

### 3. External Integrations
- **Brave Search**: Web and news search
- **Helius Solana**: Blockchain data and metadata

## Setup Instructions

### 1. Start the MCP-enabled Stack

```bash
# Start all services including n8n
./scripts/start-n8n-cerebro.sh

# Verify MCP endpoints
curl http://localhost:8000/api/mcp/servers
```

### 2. Configure Claude Desktop

Add to your Claude Desktop configuration (`~/.claude/config.json`):

```json
{
  "mcpServers": {
    "cerebro-hft": {
      "command": "curl",
      "args": [
        "-X", "POST",
        "-H", "Content-Type: application/json",
        "-d", "@-",
        "http://localhost:8000/api/mcp/call"
      ]
    }
  }
}
```

### 3. Configure Cursor

Add to Cursor settings (`settings.json`):

```json
{
  "mcp.servers": [
    {
      "name": "cerebro-hft",
      "url": "http://localhost:8000/api/mcp",
      "tools": [
        "get_hft_status",
        "get_trading_metrics",
        "analyze_market_opportunity"
      ]
    }
  ]
}
```

## Usage Examples

### Example 1: Check Trading Status

**Natural Language**: "What's the current status of my HFT bot?"

**MCP Call**:
```bash
curl -X POST http://localhost:8000/api/mcp/call \
  -H "Content-Type: application/json" \
  -d '{
    "server_name": "cerebro_internal",
    "tool_name": "get_hft_status",
    "parameters": {}
  }'
```

### Example 2: Analyze Token Risk

**Natural Language**: "Analyze the risk of token EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"

**MCP Call**:
```bash
curl -X POST http://localhost:8000/api/mcp/call \
  -H "Content-Type: application/json" \
  -d '{
    "server_name": "cerebro_internal",
    "tool_name": "analyze_market_opportunity",
    "parameters": {
      "token_address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
      "analysis_type": "risk_assessment"
    }
  }'
```

### Example 3: Trigger n8n Workflow

**Natural Language**: "Run the status monitoring workflow"

**MCP Call**:
```bash
curl -X POST http://localhost:8000/api/mcp/n8n/trigger/cerebro-status-monitor \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "manual_trigger": true,
      "requested_by": "claude"
    }
  }'
```

### Example 4: Search for Market News

**Natural Language**: "Find recent news about Solana DeFi"

**MCP Call**:
```bash
curl -X GET "http://localhost:8000/api/mcp/search/news?q=Solana%20DeFi&count=5"
```

## Available Workflows

### 1. Cerebro Status Monitor
- **ID**: `cerebro-status-monitor`
- **Frequency**: Every 5 minutes
- **Purpose**: Monitor health of all Cerebro services
- **Triggers**: Automatic + Manual via MCP

### 2. External Data Ingestion
- **ID**: `external-data-ingestion`
- **Frequency**: Every hour
- **Purpose**: Fetch market data from external sources
- **Triggers**: Automatic + Manual via MCP

## Security Considerations

### Authentication
- MCP endpoints use bearer token authentication
- Set `CEREBRO_MCP_TOKEN` environment variable
- Rate limiting: 60 requests/minute per client

### Network Security
- MCP server runs on internal Docker network
- External access only through Cerebro BFF
- All requests logged for audit

### Data Privacy
- No sensitive trading data exposed via MCP
- Portfolio details require explicit authorization
- Emergency stop requires additional confirmation

## Troubleshooting

### Common Issues

1. **MCP Client Not Initialized**
   ```bash
   # Check Cerebro BFF logs
   docker-compose logs cerebro-bff
   
   # Restart BFF service
   docker-compose restart cerebro-bff
   ```

2. **n8n Workflows Not Responding**
   ```bash
   # Check n8n status
   curl http://localhost:5678/healthz
   
   # Restart n8n
   docker-compose restart n8n
   ```

3. **Rate Limiting Issues**
   ```bash
   # Check rate limit status
   curl http://localhost:8000/api/mcp/servers
   
   # Wait 1 minute for reset
   ```

### Debug Mode

Enable debug logging:
```bash
export CEREBRO_MCP_DEBUG=true
docker-compose restart cerebro-bff
```

## Advanced Usage

### Custom MCP Tools

Create custom tools by extending the MCP client:

```python
# In cerebro/bff/mcp_client.py
class CustomMCPTool(MCPTool):
    name = "custom_analysis"
    description = "Custom market analysis"
    # ... implementation
```

### Webhook Integration

Set up webhooks for real-time notifications:

```bash
# Register webhook with external service
curl -X POST https://external-service.com/webhooks \
  -d '{
    "url": "http://localhost:8000/api/mcp/webhook/market-signal",
    "events": ["price_alert", "volume_spike"]
  }'
```

## Performance Metrics

- **Average Response Time**: <200ms
- **Throughput**: 100 requests/minute
- **Uptime**: >99.9%
- **Error Rate**: <0.1%

## Future Enhancements

- [ ] GraphQL MCP interface
- [ ] Real-time WebSocket MCP
- [ ] Multi-tenant MCP servers
- [ ] Advanced authentication (OAuth2)
- [ ] MCP tool marketplace integration
