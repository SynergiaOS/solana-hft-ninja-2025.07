import scrapy
import json
import re
from datetime import datetime, timedelta
from urllib.parse import urljoin


class NewsAggregatorSpider(scrapy.Spider):
    name = "news_aggregator"
    allowed_domains = [
        "cointelegraph.com", "coindesk.com", "decrypt.co",
        "theblock.co", "cryptonews.com", "solana.com",
        "dune.com", "flipside.crypto", "defillama.com"
    ]

    # News sources with their specific selectors
    news_sources = {
        'cointelegraph': {
            'base_url': 'https://cointelegraph.com',
            'search_url': 'https://cointelegraph.com/tags/solana',
            'article_selector': '.post-card-inline',
            'title_selector': '.post-card-inline__title a',
            'link_selector': '.post-card-inline__title a::attr(href)',
            'date_selector': '.post-card-inline__date',
            'summary_selector': '.post-card-inline__text'
        },
        'coindesk': {
            'base_url': 'https://www.coindesk.com',
            'search_url': 'https://www.coindesk.com/tag/solana/',
            'article_selector': '.articleTextSection',
            'title_selector': 'h3 a, h4 a',
            'link_selector': 'h3 a::attr(href), h4 a::attr(href)',
            'date_selector': '.typography__StyledTypography-sc-owin6q-0',
            'summary_selector': '.box__StyledBox-sc-1bsd7ul-0'
        },
        'theblock': {
            'base_url': 'https://www.theblock.co',
            'search_url': 'https://www.theblock.co/search?query=solana',
            'article_selector': '.article-card',
            'title_selector': '.article-card__title',
            'link_selector': '.article-card__title a::attr(href)',
            'date_selector': '.article-card__date',
            'summary_selector': '.article-card__excerpt'
        }
    }

    # Keywords for Solana ecosystem monitoring
    solana_keywords = [
        'solana', 'sol', 'spl', 'phantom', 'raydium', 'orca', 'jupiter',
        'jito', 'helius', 'magic eden', 'tensor', 'marinade', 'lido',
        'serum', 'mango', 'drift', 'kamino', 'marginfi', 'solend'
    ]

    # Market impact keywords
    impact_keywords = [
        'partnership', 'integration', 'launch', 'listing', 'delisting',
        'hack', 'exploit', 'upgrade', 'mainnet', 'testnet', 'airdrop',
        'funding', 'investment', 'acquisition', 'regulation', 'sec'
    ]

    custom_settings = {
        'DOWNLOAD_DELAY': 1,
        'RANDOMIZE_DOWNLOAD_DELAY': True,
        'USER_AGENT': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
        'COOKIES_ENABLED': True,
        'ROBOTSTXT_OBEY': True,
    }

    def start_requests(self):
        """Generate requests for all news sources"""
        for source_name, config in self.news_sources.items():
            yield scrapy.Request(
                url=config['search_url'],
                callback=self.parse_news_list,
                meta={'source': source_name, 'config': config}
            )

    def parse_news_list(self, response):
        """Parse news listing pages"""
        source = response.meta['source']
        config = response.meta['config']

        # Extract article links
        articles = response.css(config['article_selector'])

        for article in articles[:10]:  # Limit to recent articles
            title_element = article.css(config['title_selector'])
            link_element = article.css(config['link_selector'])

            if title_element and link_element:
                title = title_element.get()
                link = link_element.get()

                # Clean title
                if title:
                    title = re.sub(r'<[^>]+>', '', title).strip()

                # Make absolute URL
                if link and not link.startswith('http'):
                    link = urljoin(config['base_url'], link)

                # Check if article is relevant to Solana
                if self.is_solana_relevant(title):
                    yield scrapy.Request(
                        url=link,
                        callback=self.parse_article,
                        meta={
                            'source': source,
                            'title': title,
                            'article_url': link
                        }
                    )

    def parse_article(self, response):
        """Parse individual news articles"""
        source = response.meta['source']
        title = response.meta['title']
        article_url = response.meta['article_url']

        # Extract article content
        content_selectors = [
            '.article-content', '.post-content', '.entry-content',
            '.article-body', '.story-body', '.content-body',
            'article', '.article', '[role="article"]'
        ]

        content = ""
        for selector in content_selectors:
            content_elements = response.css(f'{selector} p::text').getall()
            if content_elements:
                content = ' '.join(content_elements)
                break

        # Extract publication date
        date_selectors = [
            'time::attr(datetime)', '[datetime]::attr(datetime)',
            '.date::text', '.published::text', '.timestamp::text'
        ]

        pub_date = None
        for selector in date_selectors:
            date_element = response.css(selector).get()
            if date_element:
                pub_date = date_element
                break

        # Extract author
        author_selectors = [
            '.author::text', '.byline::text', '[rel="author"]::text',
            '.writer::text', '.journalist::text'
        ]

        author = None
        for selector in author_selectors:
            author_element = response.css(selector).get()
            if author_element:
                author = author_element.strip()
                break

        # Analyze content for market impact
        impact_score = self.calculate_impact_score(title, content)
        sentiment = self.analyze_sentiment(title, content)

        # Extract mentioned tokens/projects
        mentioned_projects = self.extract_mentioned_projects(title + ' ' + content)

        article_data = {
            'title': title,
            'url': article_url,
            'source': source,
            'content': content[:1000],  # Limit content length
            'author': author,
            'published_date': pub_date,
            'impact_score': impact_score,
            'sentiment': sentiment,
            'mentioned_projects': mentioned_projects,
            'solana_keywords_found': [kw for kw in self.solana_keywords if kw.lower() in (title + ' ' + content).lower()],
            'impact_keywords_found': [kw for kw in self.impact_keywords if kw.lower() in (title + ' ' + content).lower()],
        }

        yield {
            'type': 'news_article',
            'data': article_data,
            'source': 'news_aggregator',
            'collected_at': datetime.now().isoformat()
        }

    def is_solana_relevant(self, text):
        """Check if text is relevant to Solana ecosystem"""
        if not text:
            return False

        text_lower = text.lower()
        return any(keyword in text_lower for keyword in self.solana_keywords)

    def calculate_impact_score(self, title, content):
        """Calculate potential market impact score (0-100)"""
        text = (title + ' ' + content).lower()

        # High impact keywords
        high_impact = ['hack', 'exploit', 'sec', 'regulation', 'ban', 'partnership', 'acquisition']
        medium_impact = ['listing', 'integration', 'upgrade', 'launch', 'funding']
        low_impact = ['update', 'announcement', 'event', 'conference']

        score = 0
        for keyword in high_impact:
            if keyword in text:
                score += 30

        for keyword in medium_impact:
            if keyword in text:
                score += 20

        for keyword in low_impact:
            if keyword in text:
                score += 10

        return min(score, 100)

    def analyze_sentiment(self, title, content):
        """Simple sentiment analysis"""
        text = (title + ' ' + content).lower()

        positive_words = ['bullish', 'growth', 'surge', 'rally', 'partnership', 'adoption', 'breakthrough']
        negative_words = ['bearish', 'crash', 'hack', 'exploit', 'ban', 'regulation', 'decline']

        positive_count = sum(1 for word in positive_words if word in text)
        negative_count = sum(1 for word in negative_words if word in text)

        if positive_count > negative_count:
            return 'positive'
        elif negative_count > positive_count:
            return 'negative'
        else:
            return 'neutral'

    def extract_mentioned_projects(self, text):
        """Extract mentioned Solana projects"""
        projects = [
            'phantom', 'raydium', 'orca', 'jupiter', 'jito', 'helius',
            'magic eden', 'tensor', 'marinade', 'lido', 'serum', 'mango',
            'drift', 'kamino', 'marginfi', 'solend', 'step finance'
        ]

        text_lower = text.lower()
        mentioned = []

        for project in projects:
            if project in text_lower:
                mentioned.append(project)

        return mentioned
