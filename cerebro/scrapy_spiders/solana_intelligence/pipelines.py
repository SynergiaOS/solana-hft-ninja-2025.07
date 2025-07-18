# Define your item pipelines here
#
# Don't forget to add your pipeline to the ITEM_PIPELINES setting
# See: https://docs.scrapy.org/en/latest/topics/item-pipeline.html

import json
import os
import redis
import logging
from datetime import datetime
from itemadapter import ItemAdapter


class SolanaIntelligencePipeline:
    """Base pipeline for processing scraped items"""

    def process_item(self, item, spider):
        adapter = ItemAdapter(item)

        # Add processing timestamp if not present
        if not adapter.get('collected_at'):
            adapter['collected_at'] = datetime.now().isoformat()

        # Validate required fields
        if not adapter.get('type'):
            raise ValueError("Item must have a 'type' field")

        return item


class DataValidationPipeline:
    """Validate and clean scraped data"""

    def process_item(self, item, spider):
        adapter = ItemAdapter(item)

        # Clean text fields
        for field_name, field_value in adapter.items():
            if isinstance(field_value, str):
                # Remove extra whitespace
                adapter[field_name] = field_value.strip()

                # Remove null bytes that can cause issues
                adapter[field_name] = adapter[field_name].replace('\x00', '')

        # Validate specific item types
        item_type = adapter.get('type')

        if item_type == 'news_article':
            self._validate_news_article(adapter)
        elif item_type == 'project_audit':
            self._validate_project_audit(adapter)
        elif item_type == 'trending_tokens':
            self._validate_trending_tokens(adapter)

        return item

    def _validate_news_article(self, adapter):
        """Validate news article data"""
        data = adapter.get('data', {})

        if not data.get('title'):
            raise ValueError("News article must have a title")

        if not data.get('url'):
            raise ValueError("News article must have a URL")

        # Ensure impact_score is numeric
        if 'impact_score' in data:
            try:
                data['impact_score'] = float(data['impact_score'])
            except (ValueError, TypeError):
                data['impact_score'] = 0

    def _validate_project_audit(self, adapter):
        """Validate project audit data"""
        data = adapter.get('data', {})

        if not data.get('project_name'):
            raise ValueError("Project audit must have a project name")

        # Ensure health_score is between 0-100
        if 'health_score' in data:
            try:
                score = float(data['health_score'])
                data['health_score'] = max(0, min(100, score))
            except (ValueError, TypeError):
                data['health_score'] = 0

    def _validate_trending_tokens(self, adapter):
        """Validate trending tokens data"""
        data = adapter.get('data', {})

        if not data.get('tokens'):
            raise ValueError("Trending tokens must have tokens list")


class DuplicateFilterPipeline:
    """Filter out duplicate items"""

    def __init__(self):
        self.seen_items = set()

    def process_item(self, item, spider):
        adapter = ItemAdapter(item)

        # Create a unique identifier for the item
        item_id = self._generate_item_id(adapter)

        if item_id in self.seen_items:
            logging.info(f"Duplicate item filtered: {item_id}")
            raise ValueError(f"Duplicate item: {item_id}")

        self.seen_items.add(item_id)
        return item

    def _generate_item_id(self, adapter):
        """Generate unique ID for item"""
        item_type = adapter.get('type')
        data = adapter.get('data', {})

        if item_type == 'news_article':
            return f"news_{data.get('url', '')}"
        elif item_type == 'project_audit':
            return f"audit_{data.get('project_name', '')}_{data.get('component', '')}"
        elif item_type == 'discord_messages':
            return f"discord_{data.get('server_name', '')}_{data.get('timestamp', '')}"
        else:
            return f"{item_type}_{adapter.get('collected_at', '')}"


class JsonFilesPipeline:
    """Save items to JSON files"""

    def __init__(self, output_dir='scraped_data'):
        self.output_dir = output_dir
        self.files = {}

    @classmethod
    def from_crawler(cls, crawler):
        return cls(
            output_dir=crawler.settings.get('JSON_OUTPUT_DIR', 'scraped_data')
        )

    def open_spider(self, spider):
        """Create output directory"""
        os.makedirs(self.output_dir, exist_ok=True)

    def close_spider(self, spider):
        """Close all open files"""
        for file_handle in self.files.values():
            file_handle.close()

    def process_item(self, item, spider):
        adapter = ItemAdapter(item)
        item_type = adapter.get('type', 'unknown')

        # Get or create file handle for this item type
        if item_type not in self.files:
            filename = f"{item_type}_{datetime.now().strftime('%Y%m%d')}.jsonl"
            filepath = os.path.join(self.output_dir, filename)
            self.files[item_type] = open(filepath, 'a', encoding='utf-8')

        # Write item to file
        line = json.dumps(dict(adapter), ensure_ascii=False) + '\n'
        self.files[item_type].write(line)
        self.files[item_type].flush()

        return item


class DragonflyDBPipeline:
    """Send high-priority items to DragonflyDB for real-time processing"""

    def __init__(self, redis_host='localhost', redis_port=6379, redis_db=0):
        self.redis_host = redis_host
        self.redis_port = redis_port
        self.redis_db = redis_db
        self.redis_client = None

    @classmethod
    def from_crawler(cls, crawler):
        return cls(
            redis_host=crawler.settings.get('DRAGONFLY_HOST', 'localhost'),
            redis_port=crawler.settings.get('DRAGONFLY_PORT', 6379),
            redis_db=crawler.settings.get('DRAGONFLY_DB', 0)
        )

    def open_spider(self, spider):
        """Connect to DragonflyDB"""
        try:
            self.redis_client = redis.Redis(
                host=self.redis_host,
                port=self.redis_port,
                db=self.redis_db,
                decode_responses=True
            )
            # Test connection
            self.redis_client.ping()
            logging.info("Connected to DragonflyDB")
        except Exception as e:
            logging.error(f"Failed to connect to DragonflyDB: {e}")
            self.redis_client = None

    def close_spider(self, spider):
        """Close DragonflyDB connection"""
        if self.redis_client:
            self.redis_client.close()

    def process_item(self, item, spider):
        if not self.redis_client:
            return item

        adapter = ItemAdapter(item)
        item_type = adapter.get('type')

        # Only send high-priority items to DragonflyDB
        high_priority_types = [
            'project_audit',  # Risk alerts
            'trending_tokens',  # Market opportunities
            'new_pairs'  # New trading opportunities
        ]

        if item_type in high_priority_types:
            try:
                # Store in DragonflyDB with TTL
                key = f"scrapy:{item_type}:{datetime.now().strftime('%Y%m%d_%H%M%S')}"
                value = json.dumps(dict(adapter), ensure_ascii=False)

                # Set TTL based on item type
                ttl = 3600  # 1 hour default
                if item_type == 'project_audit':
                    ttl = 86400  # 24 hours for audit data
                elif item_type in ['trending_tokens', 'new_pairs']:
                    ttl = 1800  # 30 minutes for market data

                self.redis_client.setex(key, ttl, value)

                # Also add to a list for easy retrieval
                list_key = f"scrapy:list:{item_type}"
                self.redis_client.lpush(list_key, key)
                self.redis_client.ltrim(list_key, 0, 99)  # Keep only last 100 items

                logging.info(f"Stored {item_type} in DragonflyDB: {key}")

            except Exception as e:
                logging.error(f"Failed to store item in DragonflyDB: {e}")

        return item


class AlertPipeline:
    """Generate alerts for critical findings"""

    def __init__(self):
        self.alert_thresholds = {
            'project_audit_health_score': 30,  # Alert if health score < 30
            'token_risk_score': 80,  # Alert if risk score > 80
            'news_impact_score': 70,  # Alert if impact score > 70
        }

    def process_item(self, item, spider):
        adapter = ItemAdapter(item)
        item_type = adapter.get('type')
        data = adapter.get('data', {})

        alerts = []

        if item_type == 'project_audit':
            health_score = data.get('health_score', 100)
            if health_score < self.alert_thresholds['project_audit_health_score']:
                alerts.append({
                    'type': 'project_risk',
                    'severity': 'high' if health_score < 10 else 'medium',
                    'message': f"Project {data.get('project_name')} health score: {health_score}",
                    'data': data
                })

        elif item_type == 'new_pairs':
            pairs = data.get('pairs', [])
            for pair in pairs:
                risk_score = pair.get('risk_score', 0)
                if risk_score > self.alert_thresholds['token_risk_score']:
                    alerts.append({
                        'type': 'high_risk_token',
                        'severity': 'high',
                        'message': f"High risk token detected: {pair.get('symbol')} (risk: {risk_score})",
                        'data': pair
                    })

        elif item_type == 'news_article':
            impact_score = data.get('impact_score', 0)
            if impact_score > self.alert_thresholds['news_impact_score']:
                alerts.append({
                    'type': 'high_impact_news',
                    'severity': 'medium',
                    'message': f"High impact news: {data.get('title')} (impact: {impact_score})",
                    'data': data
                })

        # Add alerts to item if any were generated
        if alerts:
            adapter['alerts'] = alerts
            logging.warning(f"Generated {len(alerts)} alerts for {item_type}")

        return item
