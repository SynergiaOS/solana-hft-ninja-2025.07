# Define here the models for your scraped items
#
# See documentation in:
# https://docs.scrapy.org/en/latest/topics/items.html

import scrapy


class SolanaIntelligenceItem(scrapy.Item):
    """Base item for all Solana intelligence data"""
    type = scrapy.Field()
    data = scrapy.Field()
    source = scrapy.Field()
    collected_at = scrapy.Field()


class DiscordMessageItem(scrapy.Item):
    """Discord message data"""
    server_name = scrapy.Field()
    channel_name = scrapy.Field()
    message_content = scrapy.Field()
    author = scrapy.Field()
    timestamp = scrapy.Field()
    sentiment = scrapy.Field()
    keywords_found = scrapy.Field()
    message_id = scrapy.Field()


class ProjectAuditItem(scrapy.Item):
    """Project audit results"""
    project_name = scrapy.Field()
    component = scrapy.Field()  # website, github, twitter, etc.
    url = scrapy.Field()
    health_score = scrapy.Field()
    issues = scrapy.Field()
    status_code = scrapy.Field()
    timestamp = scrapy.Field()
    risk_level = scrapy.Field()


class NewsArticleItem(scrapy.Item):
    """News article data"""
    title = scrapy.Field()
    url = scrapy.Field()
    source = scrapy.Field()
    content = scrapy.Field()
    author = scrapy.Field()
    published_date = scrapy.Field()
    impact_score = scrapy.Field()
    sentiment = scrapy.Field()
    mentioned_projects = scrapy.Field()
    solana_keywords_found = scrapy.Field()
    impact_keywords_found = scrapy.Field()


class TrendingTokenItem(scrapy.Item):
    """Trending token data from DEX"""
    symbol = scrapy.Field()
    name = scrapy.Field()
    price = scrapy.Field()
    volume_24h = scrapy.Field()
    price_change_24h = scrapy.Field()
    liquidity = scrapy.Field()
    market_cap = scrapy.Field()
    dex = scrapy.Field()
    contract_address = scrapy.Field()
    risk_score = scrapy.Field()


class NewPairItem(scrapy.Item):
    """New trading pair data"""
    symbol = scrapy.Field()
    pair_address = scrapy.Field()
    dex = scrapy.Field()
    age = scrapy.Field()
    liquidity = scrapy.Field()
    volume_24h = scrapy.Field()
    price_change_24h = scrapy.Field()
    risk_score = scrapy.Field()
    created_at = scrapy.Field()


class SolanaOverviewItem(scrapy.Item):
    """Solana ecosystem overview"""
    total_volume_24h = scrapy.Field()
    total_pairs = scrapy.Field()
    top_gainers = scrapy.Field()
    top_losers = scrapy.Field()
    new_listings = scrapy.Field()
    timestamp = scrapy.Field()
