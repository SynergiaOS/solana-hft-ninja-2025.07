# Define here the models for your spider middleware
#
# See documentation in:
# https://docs.scrapy.org/en/latest/topics/spider-middleware.html

import random
import time
import logging
from scrapy import signals
from scrapy.http import HtmlResponse

# useful for handling different item types with a single interface
from itemadapter import is_item, ItemAdapter


class SolanaIntelligenceSpiderMiddleware:
    """Enhanced spider middleware for Solana intelligence gathering"""

    @classmethod
    def from_crawler(cls, crawler):
        s = cls()
        crawler.signals.connect(s.spider_opened, signal=signals.spider_opened)
        return s

    def process_spider_input(self, response, spider):
        """Process responses before they reach the spider"""
        spider.logger.debug(f"Processing response from {response.url} (status: {response.status})")

        if response.status == 429:
            spider.logger.warning(f"Rate limited by {response.url}")

        return None

    def process_spider_output(self, response, result, spider):
        """Process spider output and add metadata"""
        for item in result:
            if is_item(item):
                adapter = ItemAdapter(item)
                adapter['response_url'] = response.url
                adapter['response_status'] = response.status
                adapter['spider_name'] = spider.name
            yield item

    def process_spider_exception(self, response, exception, spider):
        """Handle spider exceptions"""
        spider.logger.error(f"Spider exception for {response.url}: {exception}")
        return None

    def spider_opened(self, spider):
        spider.logger.info(f"Solana Intelligence Spider opened: {spider.name}")


class SolanaIntelligenceDownloaderMiddleware:
    """Enhanced downloader middleware with anti-detection features"""

    def __init__(self):
        self.request_count = 0
        self.last_request_time = 0

    @classmethod
    def from_crawler(cls, crawler):
        s = cls()
        crawler.signals.connect(s.spider_opened, signal=signals.spider_opened)
        return s

    def process_request(self, request, spider):
        """Add anti-detection measures to requests"""
        self.request_count += 1

        # Add random delay between requests
        current_time = time.time()
        if self.last_request_time > 0:
            time_diff = current_time - self.last_request_time
            if time_diff < 1:  # Minimum 1 second between requests
                time.sleep(1 - time_diff)

        self.last_request_time = time.time()

        # Add browser-like headers
        request.headers.setdefault('Accept', 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8')
        request.headers.setdefault('Accept-Language', 'en-US,en;q=0.5')
        request.headers.setdefault('Accept-Encoding', 'gzip, deflate')
        request.headers.setdefault('Connection', 'keep-alive')

        spider.logger.debug(f"Processing request #{self.request_count} to {request.url}")
        return None

    def process_response(self, request, response, spider):
        """Process responses and handle common issues"""
        spider.logger.debug(f"Response {response.status} from {response.url}")

        # Handle JavaScript redirects
        if response.status == 200 and 'javascript' in response.text.lower():
            if 'window.location' in response.text or 'document.location' in response.text:
                spider.logger.warning(f"JavaScript redirect detected at {response.url}")

        # Handle CloudFlare protection
        if 'cloudflare' in response.text.lower() and response.status in [403, 503]:
            spider.logger.warning(f"CloudFlare protection detected at {response.url}")

        return response

    def process_exception(self, request, exception, spider):
        """Handle download exceptions"""
        spider.logger.error(f"Download exception for {request.url}: {exception}")
        return None

    def spider_opened(self, spider):
        spider.logger.info(f"Downloader middleware activated for: {spider.name}")


class RotateUserAgentMiddleware:
    """Rotate User-Agent headers to avoid detection"""

    def __init__(self, user_agent_list):
        self.user_agent_list = user_agent_list

    @classmethod
    def from_crawler(cls, crawler):
        user_agent_list = crawler.settings.get('USER_AGENT_LIST', [
            'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36'
        ])
        return cls(user_agent_list)

    def process_request(self, request, spider):
        """Randomly select and set User-Agent"""
        if self.user_agent_list:
            user_agent = random.choice(self.user_agent_list)
            request.headers['User-Agent'] = user_agent
            spider.logger.debug(f"Set User-Agent: {user_agent[:50]}...")
        return None