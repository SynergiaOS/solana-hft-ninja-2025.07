#!/usr/bin/env python3
"""
DragonflyDB Test Suite for Project Cerebro
Tests basic CRUD operations and vector search capabilities
"""

import pytest
import redis
import numpy as np
import json
import time
import os
from typing import List, Dict, Any
import asyncio
import aioredis
from dotenv import load_dotenv

# Load environment variables
load_dotenv()


class DragonflyDBTester:
    """Test suite for DragonflyDB operations"""

    def __init__(self, host: str = None, port: int = None, password: str = None):
        # Get connection details from environment or use defaults
        self.host = host or os.getenv('DRAGONFLY_HOST', 'pj1augq7v.dragonflydb.cloud')
        self.port = port or int(os.getenv('DRAGONFLY_PORT', '6385'))
        self.password = password or os.getenv('DRAGONFLY_PASSWORD', '57q5c8g81u6q')
        self.use_ssl = True  # DragonflyDB Cloud uses SSL
        self.client = None
        self.async_client = None

    def connect(self):
        """Connect to DragonflyDB Cloud"""
        try:
            print(f"🔗 Connecting to DragonflyDB Cloud:")
            print(f"   Host: {self.host}")
            print(f"   Port: {self.port}")
            print(f"   SSL: {self.use_ssl}")

            self.client = redis.Redis(
                host=self.host,
                port=self.port,
                password=self.password,
                ssl=self.use_ssl,
                ssl_cert_reqs=None,
                decode_responses=True
            )
            # Test connection
            response = self.client.ping()
            print(f"✅ Connected to DragonflyDB Cloud: {response}")

            # Get server info
            info = self.client.info()
            print(f"✅ DragonflyDB version: {info.get('dragonfly_version', 'Unknown')}")
            print(f"✅ Memory usage: {info.get('used_memory_human', 'Unknown')}")

            return True
        except Exception as e:
            print(f"❌ Failed to connect to DragonflyDB Cloud: {e}")
            return False

    async def connect_async(self):
        """Connect to DragonflyDB asynchronously"""
        try:
            self.async_client = await aioredis.from_url(
                f"redis://:{self.password}@{self.host}:{self.port}",
                decode_responses=True
            )
            # Test connection
            await self.async_client.ping()
            print(f"✅ Async connected to DragonflyDB at {self.host}:{self.port}")
            return True
        except Exception as e:
            print(f"❌ Failed to async connect to DragonflyDB: {e}")
            return False

    def test_basic_crud(self) -> bool:
        """Test basic CRUD operations"""
        print("\n🧪 Testing Basic CRUD Operations...")

        try:
            # CREATE
            test_key = "cerebro:test:basic"
            test_value = {"message": "Hello Cerebro!", "timestamp": time.time()}
            self.client.set(test_key, json.dumps(test_value))
            print("✅ CREATE: Successfully stored test data")

            # READ
            retrieved = json.loads(self.client.get(test_key))
            assert retrieved["message"] == test_value["message"]
            print("✅ READ: Successfully retrieved test data")

            # UPDATE
            test_value["updated"] = True
            self.client.set(test_key, json.dumps(test_value))
            updated = json.loads(self.client.get(test_key))
            assert updated["updated"] == True
            print("✅ UPDATE: Successfully updated test data")

            # DELETE
            self.client.delete(test_key)
            assert self.client.get(test_key) is None
            print("✅ DELETE: Successfully deleted test data")

            return True

        except Exception as e:
            print(f"❌ CRUD test failed: {e}")
            return False

    def test_vector_operations(self) -> bool:
        """Test vector storage and similarity search"""
        print("\n🧪 Testing Vector Operations...")

        try:
            # Create test vectors (simulating embeddings)
            vectors = {
                "cerebro:vector:trading_loss": np.random.rand(384).tolist(),
                "cerebro:vector:market_analysis": np.random.rand(384).tolist(),
                "cerebro:vector:strategy_optimization": np.random.rand(384).tolist(),
            }

            # Store vectors with metadata
            for key, vector in vectors.items():
                context_data = {
                    "vector": vector,
                    "content": f"Test content for {key}",
                    "type": "test_insight",
                    "timestamp": time.time(),
                    "source": "test_suite"
                }
                self.client.set(key, json.dumps(context_data))

            print("✅ VECTOR STORAGE: Successfully stored test vectors")

            # Test vector retrieval
            retrieved_vector = json.loads(self.client.get("cerebro:vector:trading_loss"))
            assert len(retrieved_vector["vector"]) == 384
            print("✅ VECTOR RETRIEVAL: Successfully retrieved vector data")

            # Test pattern matching (simulating similarity search)
            pattern_keys = self.client.keys("cerebro:vector:*")
            assert len(pattern_keys) == 3
            print("✅ PATTERN SEARCH: Successfully found vector keys")

            # Cleanup
            for key in vectors.keys():
                self.client.delete(key)

            return True

        except Exception as e:
            print(f"❌ Vector test failed: {e}")
            return False

    def test_performance(self) -> bool:
        """Test performance with bulk operations"""
        print("\n🧪 Testing Performance...")

        try:
            # Bulk write test
            start_time = time.time()
            test_data = {}

            for i in range(1000):
                key = f"cerebro:perf:test_{i}"
                value = {
                    "id": i,
                    "data": f"Performance test data {i}",
                    "timestamp": time.time()
                }
                test_data[key] = json.dumps(value)

            # Use pipeline for bulk operations
            pipe = self.client.pipeline()
            for key, value in test_data.items():
                pipe.set(key, value)
            pipe.execute()

            write_time = time.time() - start_time
            print(f"✅ BULK WRITE: 1000 records in {write_time:.3f}s ({1000/write_time:.0f} ops/sec)")

            # Bulk read test
            start_time = time.time()
            pipe = self.client.pipeline()
            for key in test_data.keys():
                pipe.get(key)
            results = pipe.execute()

            read_time = time.time() - start_time
            print(f"✅ BULK READ: 1000 records in {read_time:.3f}s ({1000/read_time:.0f} ops/sec)")

            # Cleanup
            self.client.delete(*test_data.keys())

            return True

        except Exception as e:
            print(f"❌ Performance test failed: {e}")
            return False

    def run_all_tests(self) -> bool:
        """Run all DragonflyDB tests"""
        print("🚀 Starting DragonflyDB Test Suite for Project Cerebro")
        print("=" * 60)

        if not self.connect():
            return False

        tests = [
            self.test_basic_crud,
            self.test_vector_operations,
            self.test_performance
        ]

        passed = 0
        total = len(tests)

        for test in tests:
            if test():
                passed += 1
            else:
                print(f"❌ Test {test.__name__} failed")

        print("\n" + "=" * 60)
        print(f"📊 Test Results: {passed}/{total} tests passed")

        if passed == total:
            print("🎉 All DragonflyDB tests passed! Ready for Cerebro integration.")
            return True
        else:
            print("⚠️  Some tests failed. Please check DragonflyDB configuration.")
            return False


def main():
    """Main test runner"""
    tester = DragonflyDBTester()
    success = tester.run_all_tests()
    return 0 if success else 1


if __name__ == "__main__":
    exit(main())