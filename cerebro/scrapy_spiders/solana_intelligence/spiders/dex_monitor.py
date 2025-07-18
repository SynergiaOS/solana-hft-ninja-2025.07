import scrapy
import json
import re
from datetime import datetime
from urllib.parse import urljoin, quote


class DexMonitorSpider(scrapy.Spider):
    name = "dex_monitor"
    allowed_domains = [
        "dexscreener.com", "birdeye.so", "solscan.io",
        "raydium.io", "orca.so", "jup.ag"
    ]

    # DEX platforms to monitor
    dex_sources = {
        'dexscreener': {
            'base_url': 'https://dexscreener.com',
            'solana_url': 'https://dexscreener.com/solana',
            'trending_url': 'https://dexscreener.com/solana?rankBy=trendingScoreH6&order=desc',
            'new_pairs_url': 'https://dexscreener.com/new-pairs/solana',
        },
        'birdeye': {
            'base_url': 'https://birdeye.so',
            'trending_url': 'https://birdeye.so/trending',
            'new_tokens_url': 'https://birdeye.so/new-tokens',
        },
        'raydium': {
            'base_url': 'https://raydium.io',
            'pools_url': 'https://raydium.io/pools/',
        }
    }

    # Metrics to track
    important_metrics = [
        'volume_24h', 'price_change_24h', 'liquidity', 'market_cap',
        'holders', 'transactions', 'fdv', 'price_change_1h'
    ]

    # Risk indicators for new tokens
    risk_indicators = [
        'low_liquidity', 'high_concentration', 'new_token',
        'unverified', 'no_website', 'suspicious_activity'
    ]

    custom_settings = {
        'DOWNLOAD_DELAY': 2,
        'RANDOMIZE_DOWNLOAD_DELAY': True,
        'USER_AGENT': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
        'COOKIES_ENABLED': True,
        'ROBOTSTXT_OBEY': True,
    }

    def start_requests(self):
        """Generate requests for DEX monitoring"""
        # Monitor trending tokens
        yield scrapy.Request(
            url=self.dex_sources['dexscreener']['trending_url'],
            callback=self.parse_trending_tokens,
            meta={'source': 'dexscreener', 'type': 'trending'}
        )

        # Monitor new pairs
        yield scrapy.Request(
            url=self.dex_sources['dexscreener']['new_pairs_url'],
            callback=self.parse_new_pairs,
            meta={'source': 'dexscreener', 'type': 'new_pairs'}
        )

        # Monitor Solana overview
        yield scrapy.Request(
            url=self.dex_sources['dexscreener']['solana_url'],
            callback=self.parse_solana_overview,
            meta={'source': 'dexscreener', 'type': 'overview'}
        )

    def parse_trending_tokens(self, response):
        """Parse trending tokens from DexScreener"""
        source = response.meta['source']

        # Look for token cards/rows
        token_selectors = [
            '.ds-table-row', '.token-row', '.pair-row',
            '[data-testid="token-row"]', '.table-row'
        ]

        tokens_found = False
        for selector in token_selectors:
            token_elements = response.css(selector)
            if token_elements:
                tokens_found = True
                break

        if not tokens_found:
            # Fallback: look for any structured data
            token_elements = response.css('tr, .row, .item')[:20]

        trending_tokens = []

        for token in token_elements[:15]:  # Limit to top 15
            token_data = self.extract_token_data(token, response)
            if token_data and token_data.get('symbol'):
                trending_tokens.append(token_data)

        if trending_tokens:
            yield {
                'type': 'trending_tokens',
                'data': {
                    'tokens': trending_tokens,
                    'source': source,
                    'timestamp': datetime.now().isoformat(),
                    'total_found': len(trending_tokens)
                },
                'source': 'dex_monitor',
                'collected_at': datetime.now().isoformat()
            }

    def parse_new_pairs(self, response):
        """Parse new trading pairs"""
        source = response.meta['source']

        # Similar logic to trending but for new pairs
        pair_elements = response.css('.ds-table-row, .pair-row, tr')[:20]
        new_pairs = []

        for pair in pair_elements:
            pair_data = self.extract_pair_data(pair, response)
            if pair_data:
                # Calculate risk score for new pairs
                pair_data['risk_score'] = self.calculate_risk_score(pair_data)
                new_pairs.append(pair_data)

        if new_pairs:
            yield {
                'type': 'new_pairs',
                'data': {
                    'pairs': new_pairs,
                    'source': source,
                    'timestamp': datetime.now().isoformat(),
                    'high_risk_count': len([p for p in new_pairs if p.get('risk_score', 0) > 70])
                },
                'source': 'dex_monitor',
                'collected_at': datetime.now().isoformat()
            }

    def parse_solana_overview(self, response):
        """Parse Solana ecosystem overview"""
        # Extract overall market metrics
        overview_data = {
            'total_volume_24h': self.extract_metric(response, ['total-volume', 'volume-24h']),
            'total_pairs': self.extract_metric(response, ['total-pairs', 'pairs-count']),
            'top_gainers': self.extract_top_movers(response, 'gainers'),
            'top_losers': self.extract_top_movers(response, 'losers'),
            'new_listings': self.extract_metric(response, ['new-listings', 'new-tokens']),
        }

        yield {
            'type': 'solana_overview',
            'data': overview_data,
            'source': 'dex_monitor',
            'collected_at': datetime.now().isoformat()
        }

    def extract_token_data(self, element, response):
        """Extract token data from HTML element"""
        token_data = {}

        # Try to extract symbol/name
        symbol_selectors = [
            '.token-symbol', '.symbol', '.pair-symbol',
            '[data-testid="symbol"]', '.ticker'
        ]

        for selector in symbol_selectors:
            symbol = element.css(f'{selector}::text').get()
            if symbol:
                token_data['symbol'] = symbol.strip()
                break

        # Extract price
        price_selectors = [
            '.price', '.token-price', '[data-testid="price"]',
            '.current-price', '.last-price'
        ]

        for selector in price_selectors:
            price = element.css(f'{selector}::text').get()
            if price:
                token_data['price'] = self.clean_numeric_value(price)
                break

        # Extract volume
        volume_selectors = [
            '.volume', '.volume-24h', '[data-testid="volume"]'
        ]

        for selector in volume_selectors:
            volume = element.css(f'{selector}::text').get()
            if volume:
                token_data['volume_24h'] = self.clean_numeric_value(volume)
                break

        # Extract price change
        change_selectors = [
            '.change', '.price-change', '.change-24h',
            '[data-testid="change"]', '.percent-change'
        ]

        for selector in change_selectors:
            change = element.css(f'{selector}::text').get()
            if change:
                token_data['price_change_24h'] = self.clean_numeric_value(change)
                break

        # Extract liquidity
        liquidity_selectors = [
            '.liquidity', '.total-liquidity', '[data-testid="liquidity"]'
        ]

        for selector in liquidity_selectors:
            liquidity = element.css(f'{selector}::text').get()
            if liquidity:
                token_data['liquidity'] = self.clean_numeric_value(liquidity)
                break

        return token_data

    def extract_pair_data(self, element, response):
        """Extract trading pair data"""
        pair_data = self.extract_token_data(element, response)

        # Additional pair-specific data
        age_selectors = ['.age', '.created', '.pair-age', '[data-testid="age"]']
        for selector in age_selectors:
            age = element.css(f'{selector}::text').get()
            if age:
                pair_data['age'] = age.strip()
                break

        # Extract DEX information
        dex_selectors = ['.dex', '.exchange', '.platform', '[data-testid="dex"]']
        for selector in dex_selectors:
            dex = element.css(f'{selector}::text').get()
            if dex:
                pair_data['dex'] = dex.strip()
                break

        return pair_data

    def extract_metric(self, response, selectors):
        """Extract numeric metric from page"""
        for selector in selectors:
            value = response.css(f'.{selector}::text, #{selector}::text').get()
            if value:
                return self.clean_numeric_value(value)
        return None

    def extract_top_movers(self, response, type_):
        """Extract top gainers/losers"""
        movers = []

        # Look for sections containing gainers/losers
        section_selectors = [
            f'.{type_}', f'.top-{type_}', f'[data-testid="{type_}"]'
        ]

        for selector in section_selectors:
            section = response.css(selector)
            if section:
                tokens = section.css('.token-row, tr, .item')[:5]
                for token in tokens:
                    token_data = self.extract_token_data(token, response)
                    if token_data:
                        movers.append(token_data)
                break

        return movers

    def clean_numeric_value(self, value):
        """Clean and convert numeric values"""
        if not value:
            return None

        # Remove common formatting
        cleaned = re.sub(r'[,$%\s]', '', value.strip())

        # Handle K, M, B suffixes
        multipliers = {'K': 1000, 'M': 1000000, 'B': 1000000000}

        for suffix, multiplier in multipliers.items():
            if cleaned.upper().endswith(suffix):
                try:
                    number = float(cleaned[:-1])
                    return number * multiplier
                except ValueError:
                    pass

        # Try to convert to float
        try:
            return float(cleaned)
        except ValueError:
            return value  # Return original if can't convert

    def calculate_risk_score(self, pair_data):
        """Calculate risk score for new pairs (0-100, higher = riskier)"""
        risk_score = 0

        # Low liquidity risk
        liquidity = pair_data.get('liquidity', 0)
        if isinstance(liquidity, (int, float)):
            if liquidity < 10000:  # Less than $10k liquidity
                risk_score += 40
            elif liquidity < 50000:  # Less than $50k liquidity
                risk_score += 20

        # Age risk (newer = riskier)
        age = pair_data.get('age', '')
        if 'min' in age.lower() or 'hour' in age.lower():
            risk_score += 30
        elif 'day' in age.lower():
            if '1' in age:
                risk_score += 20
            else:
                risk_score += 10

        # Volume risk
        volume = pair_data.get('volume_24h', 0)
        if isinstance(volume, (int, float)):
            if volume < 1000:  # Very low volume
                risk_score += 25
            elif volume < 10000:  # Low volume
                risk_score += 15

        # Price change volatility
        price_change = pair_data.get('price_change_24h', 0)
        if isinstance(price_change, (int, float)):
            if abs(price_change) > 500:  # Extreme volatility
                risk_score += 30
            elif abs(price_change) > 100:  # High volatility
                risk_score += 15

        return min(risk_score, 100)
