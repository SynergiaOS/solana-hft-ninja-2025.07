# 🕷️ Solana Intelligence Scrapy System

Advanced web scraping system for gathering alternative data sources to enhance Cerebro's intelligence capabilities.

## 🎯 Overview

This Scrapy project provides specialized spiders for monitoring the Solana ecosystem:

- **Discord Monitor**: Track sentiment and activity in Solana Discord servers
- **News Aggregator**: Collect and analyze crypto news from major sources
- **Project Auditor**: Monitor project health and detect rug pull indicators
- **DEX Monitor**: Track new tokens, trending pairs, and market activity

## 🏗️ Architecture

```
cerebro/scrapy_spiders/
├── solana_intelligence/
│   ├── spiders/
│   │   ├── discord_monitor.py      # Discord sentiment analysis
│   │   ├── news_aggregator.py      # Crypto news monitoring
│   │   ├── project_auditor.py      # Project risk assessment
│   │   └── dex_monitor.py          # DEX activity tracking
│   ├── items.py                    # Data models
│   ├── pipelines.py                # Data processing pipelines
│   ├── middlewares.py              # Anti-detection & retry logic
│   └── settings.py                 # Configuration
├── scrapy.cfg                      # Scrapy configuration
└── README.md                       # This file
```

## 🚀 Quick Start

### 1. Setup Environment

```bash
cd cerebro/scrapy_spiders
source ../venv/bin/activate
pip install scrapy redis
```

### 2. Run Individual Spiders

```bash
# Monitor DEX activity
scrapy crawl dex_monitor

# Aggregate crypto news
scrapy crawl news_aggregator

# Audit project health
scrapy crawl project_auditor

# Monitor Discord (requires authentication)
scrapy crawl discord_monitor
```

### 3. Run with Custom Settings

```bash
# Limit items and save to file
scrapy crawl dex_monitor -s CLOSESPIDER_ITEMCOUNT=50 -o output.json

# Enable debug logging
scrapy crawl news_aggregator -L DEBUG

# Custom output format
scrapy crawl project_auditor -o results.csv
```

## 🕷️ Spider Details

### Discord Monitor
- **Purpose**: Track sentiment and activity in Solana Discord servers
- **Data**: Messages, sentiment scores, keyword mentions
- **Frequency**: Real-time (when possible)
- **Note**: Requires Discord API tokens for full functionality

### News Aggregator
- **Purpose**: Monitor crypto news sources for Solana mentions
- **Sources**: Cointelegraph, CoinDesk, The Block, Decrypt
- **Data**: Articles, sentiment, impact scores, mentioned projects
- **Frequency**: Every 4 hours

### Project Auditor
- **Purpose**: Monitor project health and detect risk indicators
- **Checks**: Website status, GitHub activity, social media presence
- **Data**: Health scores, risk indicators, audit results
- **Frequency**: Daily

### DEX Monitor
- **Purpose**: Track DEX activity and new token listings
- **Sources**: DexScreener, Birdeye, Raydium
- **Data**: New pairs, trending tokens, volume, liquidity
- **Frequency**: Hourly during trading hours

## 📊 Data Pipeline

```
Scrapy Spiders → Pipelines → DragonflyDB → Cerebro Agent
                     ↓
                JSON Files → Kestra Processing
```

### Pipeline Components

1. **DataValidationPipeline**: Validates and cleans scraped data
2. **DuplicateFilterPipeline**: Removes duplicate items
3. **AlertPipeline**: Generates alerts for critical findings
4. **DragonflyDBPipeline**: Sends high-priority data to DragonflyDB
5. **JsonFilesPipeline**: Saves data to JSON files

## ⚙️ Configuration

### Key Settings

```python
# Respectful scraping
DOWNLOAD_DELAY = 2
RANDOMIZE_DOWNLOAD_DELAY = True
CONCURRENT_REQUESTS_PER_DOMAIN = 8

# Anti-detection
USER_AGENT_LIST = [...]  # Rotating user agents
COOKIES_ENABLED = True
AUTOTHROTTLE_ENABLED = True

# Data processing
ITEM_PIPELINES = {
    'DataValidationPipeline': 200,
    'DuplicateFilterPipeline': 300,
    'AlertPipeline': 400,
    'DragonflyDBPipeline': 500,
    'JsonFilesPipeline': 600,
}
```

### Environment Variables

```bash
# DragonflyDB connection
DRAGONFLY_HOST=localhost
DRAGONFLY_PORT=6379
DRAGONFLY_DB=0

# Output directory
JSON_OUTPUT_DIR=scraped_data
```

## 🔧 Integration with Kestra

The system integrates with Kestra for automated scheduling:

```yaml
# Daily intelligence gathering
triggers:
  - id: daily_intelligence_gathering
    type: Schedule
    cron: "0 6 * * *"
    
  - id: hourly_dex_monitoring
    type: Schedule
    cron: "0 8-22 * * *"
```

## 📈 Monitoring & Alerts

### Alert Types

- **Project Risk**: Health score < 30
- **High Risk Token**: Risk score > 80
- **High Impact News**: Impact score > 70

### Metrics Tracked

- Items scraped per spider
- Success/failure rates
- Response times
- Alert counts

## 🛡️ Anti-Detection Features

- **User Agent Rotation**: Multiple browser user agents
- **Request Delays**: Randomized delays between requests
- **Retry Logic**: Intelligent backoff for failed requests
- **CloudFlare Detection**: Automatic detection and handling
- **Rate Limit Handling**: Automatic throttling on 429 responses

## 🔍 Data Quality

### Validation Rules

- Required fields validation
- Data type checking
- Range validation for scores
- URL format validation
- Duplicate detection

### Error Handling

- Graceful failure handling
- Comprehensive logging
- Automatic retries
- Circuit breaker patterns

## 📝 Usage Examples

### Custom Spider Run

```python
from scrapy.crawler import CrawlerProcess
from solana_intelligence.spiders.dex_monitor import DexMonitorSpider

process = CrawlerProcess({
    'CLOSESPIDER_ITEMCOUNT': 100,
    'LOG_LEVEL': 'INFO'
})

process.crawl(DexMonitorSpider)
process.start()
```

### Data Processing

```python
import json
from solana_intelligence.pipelines import AlertPipeline

# Process scraped data
with open('dex_data.json', 'r') as f:
    data = json.load(f)

# Generate alerts
pipeline = AlertPipeline()
for item in data:
    processed_item = pipeline.process_item(item, spider)
```

## 🚨 Important Notes

### Legal & Ethical Considerations

- Respects robots.txt files
- Implements rate limiting
- Uses public data only
- Follows website terms of service

### Discord Monitoring

- Requires proper authentication
- Must comply with Discord ToS
- Consider using Discord API instead of scraping

### Rate Limiting

- Automatic throttling enabled
- Respects server response times
- Implements exponential backoff

## 🔧 Troubleshooting

### Common Issues

1. **CloudFlare Protection**: Implement proxy rotation
2. **Rate Limiting**: Increase delays, reduce concurrency
3. **JavaScript Content**: Consider using Splash or Selenium
4. **Authentication**: Implement proper session handling

### Debug Commands

```bash
# Enable debug logging
scrapy crawl spider_name -L DEBUG

# Test spider without running
scrapy parse --spider=spider_name url

# Check spider syntax
scrapy check spider_name
```

## 📚 Resources

- [Scrapy Documentation](https://docs.scrapy.org/)
- [Solana Ecosystem](https://solana.com/ecosystem)
- [DexScreener API](https://docs.dexscreener.com/)
- [Discord Developer Portal](https://discord.com/developers/docs)

---

**⚠️ Disclaimer**: This tool is for educational and research purposes. Always respect website terms of service and implement appropriate rate limiting.
