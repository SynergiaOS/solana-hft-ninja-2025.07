#!/usr/bin/env python3
"""
Test script for Scrapy integration with Cerebro system
"""

import asyncio
import json
import os
import redis
import requests
import subprocess
import time
from datetime import datetime
from pathlib import Path

class ScrapyIntegrationTester:
    def __init__(self):
        self.base_dir = Path(__file__).parent
        self.scrapy_dir = self.base_dir / "scrapy_spiders"
        self.bff_url = "http://localhost:8002"
        self.redis_client = None
        
    def setup_redis(self):
        """Setup Redis connection"""
        try:
            self.redis_client = redis.Redis(
                host='localhost', 
                port=6379, 
                db=2, 
                decode_responses=True
            )
            self.redis_client.ping()
            print("✅ Redis connection established")
            return True
        except Exception as e:
            print(f"❌ Redis connection failed: {e}")
            return False
    
    def test_spider_execution(self, spider_name, max_items=5):
        """Test individual spider execution"""
        print(f"\n🕷️ Testing {spider_name} spider...")
        
        try:
            # Change to scrapy directory
            original_dir = Path.cwd()
            os.chdir(self.scrapy_dir)
            
            # Run spider with limited items
            cmd = [
                "scrapy", "crawl", spider_name,
                "-s", f"CLOSESPIDER_ITEMCOUNT={max_items}",
                "-s", "DOWNLOAD_DELAY=1",
                "-L", "INFO",
                "-o", f"test_output_{spider_name}.json"
            ]
            
            result = subprocess.run(
                cmd, 
                capture_output=True, 
                text=True, 
                timeout=60
            )
            
            # Change back to original directory
            os.chdir(original_dir)
            
            if result.returncode == 0:
                print(f"✅ {spider_name} spider executed successfully")
                
                # Check if output file was created
                output_file = self.scrapy_dir / f"test_output_{spider_name}.json"
                if output_file.exists():
                    with open(output_file, 'r') as f:
                        data = json.load(f)
                    print(f"📊 {spider_name} collected {len(data)} items")
                    
                    # Store test data in Redis
                    if self.redis_client:
                        key = f"scrapy:{spider_name}:test_{int(time.time())}"
                        self.redis_client.setex(key, 3600, json.dumps(data))
                        print(f"💾 Test data stored in Redis: {key}")
                    
                    # Cleanup
                    output_file.unlink()
                    return True
                else:
                    print(f"⚠️ {spider_name} no output file created")
                    return False
            else:
                print(f"❌ {spider_name} spider failed: {result.stderr}")
                return False
                
        except subprocess.TimeoutExpired:
            print(f"⏰ {spider_name} spider timed out")
            return False
        except Exception as e:
            print(f"❌ {spider_name} spider error: {e}")
            return False
    
    def test_bff_api(self):
        """Test BFF API endpoints"""
        print("\n🔌 Testing BFF API integration...")
        
        endpoints = [
            "/api/scrapy/status",
            "/api/scrapy/metrics",
            "/api/scrapy/alerts/recent"
        ]
        
        results = {}
        
        for endpoint in endpoints:
            try:
                response = requests.get(f"{self.bff_url}{endpoint}", timeout=10)
                if response.status_code == 200:
                    print(f"✅ {endpoint} - OK")
                    results[endpoint] = response.json()
                else:
                    print(f"❌ {endpoint} - HTTP {response.status_code}")
                    results[endpoint] = None
            except Exception as e:
                print(f"❌ {endpoint} - Error: {e}")
                results[endpoint] = None
        
        return results
    
    def test_data_flow(self):
        """Test complete data flow from Scrapy to BFF"""
        print("\n🔄 Testing complete data flow...")
        
        # Generate test alert
        test_alert = {
            "alerts": [
                {
                    "type": "test_alert",
                    "message": "Test alert from integration test",
                    "source": "integration_test",
                    "timestamp": datetime.now().isoformat(),
                    "severity": "low"
                }
            ]
        }
        
        try:
            # Send test alert to BFF
            response = requests.post(
                f"{self.bff_url}/api/scrapy/alerts",
                json=test_alert,
                timeout=10
            )
            
            if response.status_code == 200:
                print("✅ Test alert sent successfully")
                
                # Wait a moment and retrieve alerts
                time.sleep(1)
                
                response = requests.get(
                    f"{self.bff_url}/api/scrapy/alerts/recent",
                    timeout=10
                )
                
                if response.status_code == 200:
                    alerts = response.json()
                    if alerts.get("count", 0) > 0:
                        print("✅ Test alert retrieved successfully")
                        return True
                    else:
                        print("⚠️ No alerts found in response")
                        return False
                else:
                    print(f"❌ Failed to retrieve alerts: HTTP {response.status_code}")
                    return False
            else:
                print(f"❌ Failed to send test alert: HTTP {response.status_code}")
                return False
                
        except Exception as e:
            print(f"❌ Data flow test error: {e}")
            return False
    
    def test_redis_data(self):
        """Test Redis data storage and retrieval"""
        print("\n💾 Testing Redis data operations...")
        
        if not self.redis_client:
            print("❌ Redis client not available")
            return False
        
        try:
            # Check for scrapy keys
            scrapy_keys = self.redis_client.keys("scrapy:*")
            print(f"📊 Found {len(scrapy_keys)} scrapy keys in Redis")
            
            # Check for alert keys
            alert_keys = self.redis_client.keys("alerts:scrapy:*")
            print(f"🚨 Found {len(alert_keys)} alert keys in Redis")
            
            # Test data storage
            test_key = f"scrapy:test:{int(time.time())}"
            test_data = {"test": True, "timestamp": datetime.now().isoformat()}
            
            self.redis_client.setex(test_key, 60, json.dumps(test_data))
            
            # Test data retrieval
            retrieved_data = self.redis_client.get(test_key)
            if retrieved_data:
                parsed_data = json.loads(retrieved_data)
                if parsed_data.get("test") is True:
                    print("✅ Redis data storage/retrieval working")
                    
                    # Cleanup
                    self.redis_client.delete(test_key)
                    return True
                else:
                    print("❌ Redis data corruption detected")
                    return False
            else:
                print("❌ Redis data retrieval failed")
                return False
                
        except Exception as e:
            print(f"❌ Redis test error: {e}")
            return False
    
    def run_full_test(self):
        """Run complete integration test suite"""
        print("🚀 Starting Scrapy Integration Test Suite")
        print("=" * 50)
        
        results = {
            "redis_connection": False,
            "spider_tests": {},
            "bff_api": False,
            "data_flow": False,
            "redis_operations": False
        }
        
        # Test 1: Redis connection
        results["redis_connection"] = self.setup_redis()
        
        # Test 2: Spider execution (limited test)
        spiders = ["news_aggregator", "dex_monitor"]  # Test subset for speed
        
        for spider in spiders:
            results["spider_tests"][spider] = self.test_spider_execution(spider, max_items=3)
        
        # Test 3: BFF API
        bff_results = self.test_bff_api()
        results["bff_api"] = any(bff_results.values())
        
        # Test 4: Data flow
        results["data_flow"] = self.test_data_flow()
        
        # Test 5: Redis operations
        results["redis_operations"] = self.test_redis_data()
        
        # Summary
        print("\n" + "=" * 50)
        print("📋 TEST RESULTS SUMMARY")
        print("=" * 50)
        
        total_tests = 0
        passed_tests = 0
        
        for category, result in results.items():
            if category == "spider_tests":
                for spider, spider_result in result.items():
                    total_tests += 1
                    if spider_result:
                        passed_tests += 1
                    status = "✅ PASS" if spider_result else "❌ FAIL"
                    print(f"{spider}_spider: {status}")
            else:
                total_tests += 1
                if result:
                    passed_tests += 1
                status = "✅ PASS" if result else "❌ FAIL"
                print(f"{category}: {status}")
        
        print("=" * 50)
        print(f"OVERALL: {passed_tests}/{total_tests} tests passed")
        
        if passed_tests == total_tests:
            print("🎉 ALL TESTS PASSED! Scrapy integration is working correctly.")
            return True
        else:
            print("⚠️ Some tests failed. Check the output above for details.")
            return False

def main():
    """Main test function"""
    import os
    
    tester = ScrapyIntegrationTester()
    success = tester.run_full_test()
    
    if success:
        print("\n🎯 Scrapy integration is ready for production!")
        exit(0)
    else:
        print("\n🔧 Please fix the issues before proceeding.")
        exit(1)

if __name__ == "__main__":
    main()
