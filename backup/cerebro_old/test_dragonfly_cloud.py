#!/usr/bin/env python3
"""
Simple test for DragonflyDB Cloud connection
"""

import redis
import json
import time
import os
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

def test_dragonflydb_cloud():
    """Test connection to DragonflyDB Cloud"""
    try:
        # Get connection details from environment
        host = os.getenv('DRAGONFLY_HOST', 'pj1augq7v.dragonflydb.cloud')
        port = int(os.getenv('DRAGONFLY_PORT', '6385'))
        password = os.getenv('DRAGONFLY_PASSWORD', '57q5c8g81u6q')
        
        print("🐉 Testing DragonflyDB Cloud Connection")
        print("=" * 50)
        print(f"🔗 Host: {host}")
        print(f"🔗 Port: {port}")
        print(f"🔗 SSL: True")
        print("=" * 50)
        
        # Connect to DragonflyDB Cloud
        r = redis.Redis(
            host=host,
            port=port,
            password=password,
            ssl=True,
            ssl_cert_reqs=None,
            decode_responses=True
        )
        
        # Test 1: Ping
        print("🧪 Test 1: Connection Ping")
        response = r.ping()
        print(f"✅ Ping response: {response}")
        
        # Test 2: Server Info
        print("\n🧪 Test 2: Server Information")
        info = r.info()
        print(f"✅ DragonflyDB version: {info.get('dragonfly_version', 'Unknown')}")
        print(f"✅ Memory usage: {info.get('used_memory_human', 'Unknown')}")
        print(f"✅ Connected clients: {info.get('connected_clients', 'Unknown')}")
        
        # Test 3: Basic Operations
        print("\n🧪 Test 3: Basic CRUD Operations")
        test_key = "cerebro:test:connection"
        test_data = {
            "message": "Hello from Project Cerebro!",
            "timestamp": time.time(),
            "version": "1.0.0"
        }
        
        # SET
        r.set(test_key, json.dumps(test_data))
        print("✅ SET: Data stored successfully")
        
        # GET
        retrieved = json.loads(r.get(test_key))
        print(f"✅ GET: Retrieved data: {retrieved['message']}")
        
        # DELETE
        r.delete(test_key)
        print("✅ DELETE: Data deleted successfully")
        
        # Test 4: Trading Data Simulation
        print("\n🧪 Test 4: Trading Data Operations")
        
        # Store multiple trading records
        trading_data = []
        for i in range(10):
            trade = {
                "id": f"trade_{i}",
                "symbol": "SOL/USDC",
                "strategy": "sandwich" if i % 2 == 0 else "arbitrage",
                "profit_sol": round(0.001 * (i + 1), 6),
                "timestamp": time.time() + i,
                "status": "completed"
            }
            trading_data.append(trade)
            r.set(f"cerebro:trade:{i}", json.dumps(trade))
        
        print("✅ Stored 10 trading records")
        
        # Retrieve and verify
        retrieved_trades = []
        for i in range(10):
            trade_json = r.get(f"cerebro:trade:{i}")
            if trade_json:
                retrieved_trades.append(json.loads(trade_json))
        
        print(f"✅ Retrieved {len(retrieved_trades)} trading records")
        
        # Calculate total profit
        total_profit = sum(trade['profit_sol'] for trade in retrieved_trades)
        print(f"✅ Total simulated profit: {total_profit:.6f} SOL")
        
        # Test 5: Vector Storage (for AI embeddings)
        print("\n🧪 Test 5: Vector Storage Operations")
        
        vector_data = {
            "embedding": [0.1, 0.2, 0.3, 0.4, 0.5] * 20,  # 100-dim vector
            "text": "Solana sandwich strategy analysis",
            "metadata": {
                "type": "strategy_analysis",
                "confidence": 0.95,
                "market": "SOL/USDC"
            }
        }
        
        r.hset("cerebro:vector:strategy_1", mapping={
            "embedding": json.dumps(vector_data["embedding"]),
            "text": vector_data["text"],
            "metadata": json.dumps(vector_data["metadata"])
        })
        
        print("✅ Stored vector embedding")
        
        # Retrieve vector
        stored_vector = r.hgetall("cerebro:vector:strategy_1")
        embedding = json.loads(stored_vector["embedding"])
        metadata = json.loads(stored_vector["metadata"])
        
        print(f"✅ Retrieved vector: {len(embedding)} dimensions")
        print(f"✅ Vector metadata: {metadata['type']}")
        
        # Test 6: Performance Test
        print("\n🧪 Test 6: Performance Test")
        
        start_time = time.time()
        
        # Bulk operations using pipeline
        pipe = r.pipeline()
        for i in range(100):
            key = f"cerebro:perf:{i}"
            value = json.dumps({
                "id": i,
                "data": f"Performance test data {i}",
                "timestamp": time.time()
            })
            pipe.set(key, value)
        
        pipe.execute()
        end_time = time.time()
        
        duration = end_time - start_time
        ops_per_sec = 100 / duration
        
        print(f"✅ Bulk write: 100 operations in {duration:.3f}s ({ops_per_sec:.1f} ops/sec)")
        
        # Cleanup all test data
        print("\n🧹 Cleaning up test data...")
        
        # Delete trading records
        for i in range(10):
            r.delete(f"cerebro:trade:{i}")
        
        # Delete performance test data
        for i in range(100):
            r.delete(f"cerebro:perf:{i}")
        
        # Delete vector data
        r.delete("cerebro:vector:strategy_1")
        
        print("✅ Cleanup completed")
        
        print("\n" + "=" * 50)
        print("🎉 ALL TESTS PASSED!")
        print("🚀 DragonflyDB Cloud is ready for Project Cerebro!")
        print("=" * 50)
        
        return True
        
    except Exception as e:
        print(f"\n❌ Test failed: {e}")
        print("Please check your DragonflyDB Cloud configuration in .env file")
        return False

if __name__ == "__main__":
    success = test_dragonflydb_cloud()
    exit(0 if success else 1)
