import scrapy
import json
import re
from datetime import datetime
from urllib.parse import urljoin


class DiscordMonitorSpider(scrapy.Spider):
    name = "discord_monitor"
    allowed_domains = ["discord.com", "discordapp.com"]

    # Discord servers to monitor (invite links or server IDs)
    target_servers = [
        # Solana ecosystem servers
        "https://discord.gg/solana",
        "https://discord.gg/jito",
        "https://discord.gg/helius",
        "https://discord.gg/raydium",
        "https://discord.gg/orca",
        "https://discord.gg/jupiter",
        # Add more servers as needed
    ]

    # Keywords to monitor for sentiment analysis
    keywords = [
        "airdrop", "launch", "token", "rug", "scam", "pump", "dump",
        "bullish", "bearish", "moon", "crash", "hack", "exploit",
        "listing", "dex", "volume", "liquidity", "whale", "bot"
    ]

    custom_settings = {
        'DOWNLOAD_DELAY': 2,
        'RANDOMIZE_DOWNLOAD_DELAY': True,
        'USER_AGENT': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36',
        'COOKIES_ENABLED': True,
        'ROBOTSTXT_OBEY': False,  # Discord doesn't allow bots via robots.txt
    }

    def start_requests(self):
        """
        Note: This is a simplified example. Real Discord scraping requires:
        1. Authentication tokens
        2. WebSocket connections
        3. Proper rate limiting
        4. Compliance with Discord ToS

        For production, consider using Discord API or discord.py library
        """
        for url in self.target_servers:
            yield scrapy.Request(
                url=url,
                callback=self.parse_server_info,
                meta={'server_url': url}
            )

    def parse_server_info(self, response):
        """Parse Discord server information and extract relevant data"""
        server_url = response.meta['server_url']

        # Extract server metadata
        server_data = {
            'server_url': server_url,
            'timestamp': datetime.now().isoformat(),
            'status': 'accessible' if response.status == 200 else 'error',
            'response_code': response.status,
        }

        # Look for server information in page content
        title = response.css('title::text').get()
        if title:
            server_data['title'] = title.strip()

        # Extract any visible server information
        description = response.css('meta[name="description"]::attr(content)').get()
        if description:
            server_data['description'] = description.strip()

        # Look for member count or activity indicators
        member_info = response.css('.member-count, .online-count').getall()
        if member_info:
            server_data['member_info'] = member_info

        # Check for any error messages or access restrictions
        error_messages = response.css('.error-message, .access-denied').getall()
        if error_messages:
            server_data['access_issues'] = error_messages

        yield {
            'type': 'discord_server_status',
            'data': server_data,
            'source': 'discord_monitor',
            'collected_at': datetime.now().isoformat()
        }

    def parse_messages(self, response):
        """
        Parse Discord messages (placeholder for future implementation)

        Note: Real message parsing would require:
        - Discord API access
        - Proper authentication
        - WebSocket connection for real-time messages
        """
        messages = []

        # This is a placeholder - real implementation would use Discord API
        message_elements = response.css('.message-content')

        for message in message_elements:
            content = message.css('::text').get()
            if content and any(keyword in content.lower() for keyword in self.keywords):
                message_data = {
                    'content': content.strip(),
                    'timestamp': datetime.now().isoformat(),
                    'contains_keywords': [kw for kw in self.keywords if kw in content.lower()],
                    'sentiment_score': self.calculate_sentiment(content),
                }
                messages.append(message_data)

        if messages:
            yield {
                'type': 'discord_messages',
                'data': messages,
                'source': 'discord_monitor',
                'collected_at': datetime.now().isoformat()
            }

    def calculate_sentiment(self, text):
        """Simple sentiment analysis based on keywords"""
        positive_words = ['bullish', 'moon', 'pump', 'good', 'great', 'amazing', 'launch']
        negative_words = ['bearish', 'dump', 'crash', 'rug', 'scam', 'hack', 'exploit']

        text_lower = text.lower()
        positive_count = sum(1 for word in positive_words if word in text_lower)
        negative_count = sum(1 for word in negative_words if word in text_lower)

        if positive_count > negative_count:
            return 'positive'
        elif negative_count > positive_count:
            return 'negative'
        else:
            return 'neutral'
