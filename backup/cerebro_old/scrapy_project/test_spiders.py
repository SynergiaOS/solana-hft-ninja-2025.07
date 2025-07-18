#!/usr/bin/env python3
"""
Test script for Scrapy spiders
"""
import subprocess
import sys
import os

def test_spider(spider_name):
    """Test individual spider"""
    print(f"🕷️ Testing {spider_name} spider...")
    
    cmd = [
        'scrapy', 'crawl', spider_name,
        '-s', 'CLOSESPIDER_PAGECOUNT=5',  # Limit pages for testing
        '-L', 'INFO'
    ]
    
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
        if result.returncode == 0:
            print(f"✅ {spider_name} spider test passed")
            return True
        else:
            print(f"❌ {spider_name} spider test failed: {result.stderr}")
            return False
    except subprocess.TimeoutExpired:
        print(f"⏰ {spider_name} spider test timed out")
        return False

def main():
    """Run all spider tests"""
    os.chdir('/app/cerebro/scrapy_project')
    
    spiders = ['discord_monitor', 'project_auditor', 'news_aggregator', 'dex_monitor']
    
    for spider in spiders:
        test_spider(spider)

if __name__ == "__main__":
    main()